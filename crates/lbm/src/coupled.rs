//! Two CPU fluid lattices sharing one finite-volume wall. The CHT link is the same as in
//! `flow`/`conduction` (Patankar 1980; Li, Mei & Klausner, JCP 237:366, 2013). Each circuit's
//! affine link heat is converted through joules before addition to the single wall balance.
//! The wall advances once with physical capacity/source; its effective temperature sets both
//! circuits' incoming populations. No temperature or flux is copied between fluid lattices.

use std::collections::BTreeMap;

use crate::conduction::{Face, Interface, SolidHeat};
use crate::{Backend, Error, Fields, Flow, Limits, Memory, Obstacle, Result, Setup};

/// Heat rates in W over one common physical time step. Positive transfers heat the wall.
#[derive(Debug, Clone, Default, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct Exchange {
    pub time: f64,
    pub into_wall: [f64; 2],
    pub wall_storage: f64,
    pub wall_power: f64,
    pub wall_residual: f64,
    /// Net energy inserted by open-boundary population resets, divided by dt. This discrete
    /// boundary exchange includes advective and conductive effects; it is not a port integral.
    pub open: [f64; 2],
    /// Heat leaving each fluid through its other walls (positive is an external loss).
    pub external: [f64; 2],
}

struct Side {
    cells: Vec<u32>,
    map: Vec<u32>,
    links: Vec<Interface>,
    heat_scale: f64,
    zero: f64,
    temperature: Vec<f64>,
}

/// Conservative combined host/output bound, including the temporary second wall during
/// preparation. The persistent thermal wall has one owner; view maps are counted separately.
#[derive(Debug, Clone, PartialEq, serde::Serialize)]
pub struct CoupledMemory {
    pub cells: u64,
    pub preparation_bytes: u64,
    pub frame_bytes: u64,
    pub coupling_bytes: u64,
    pub bound_bytes: u64,
}

/// CPU + CPU, equal aligned spacing, identical complete separator occupancy and common dt.
/// Other solids, thermal contacts, held/flux separator ends and accelerated capacities are
/// refused. Momentum populations and fluid properties stay independent.
pub struct CoupledFlow {
    pub(crate) flows: [Flow; 2],
    sides: [Side; 2],
    wall: SolidHeat,
    dt: f64,
    initial_wall_energy: f64,
}

fn invalid(message: impl Into<String>) -> Error {
    Error::Invalid(format!("coupled exchanger: {}", message.into()))
}

fn key(grid: &crate::voxel::Grid, cell: u32, origin: [f64; 3]) -> Result<[i64; 3]> {
    if cell as usize >= grid.len() {
        return Err(invalid("wall cell outside grid"));
    }
    let xyz = grid.coords(cell as usize);
    let mut key = [0; 3];
    for axis in 0..3 {
        let offset = (grid.origin[axis] - origin[axis]) / grid.spacing;
        if !offset.is_finite() || offset.abs() > i32::MAX as f64 || (offset - offset.round()).abs() > 1e-8 {
            return Err(invalid("grid origins must align at the common spacing"));
        }
        key[axis] = offset.round() as i64 + xyz[axis] as i64;
    }
    Ok(key)
}

impl CoupledFlow {
    /// Validate both preparations and their combined memory/work caps before allocating flows.
    pub fn estimate(setups: [&Setup; 2], wall: &str, initial: f64, limits: &Limits) -> Result<CoupledMemory> {
        if !initial.is_finite() || initial < 0.0 {
            return Err(invalid("initial wall temperature must be finite kelvin above or equal to zero"));
        }
        let [a, b] = setups;
        if a.step == b.step {
            return Err(invalid("choose two distinct fluid steps"));
        }
        if a.grid.spacing != b.grid.spacing || a.time_step != b.time_step || a.steps != b.steps || a.frames != b.frames {
            return Err(invalid(format!("steps `{}` and `{}` need equal spacing, time step, length and frame count", a.step, b.step)));
        }
        key(&b.grid, 0, a.grid.origin)?;
        for face in fieldcraft_model::BoxFace::ALL {
            let first = matches!(a.domain.faces.get(face), fieldcraft_model::FaceCondition::Periodic {});
            let second = matches!(b.domain.faces.get(face), fieldcraft_model::FaceCondition::Periodic {});
            if (first || second)
                && (first != second
                    || a.domain.min[face.axis()] != b.domain.min[face.axis()]
                    || a.domain.max[face.axis()] != b.domain.max[face.axis()])
            {
                return Err(invalid("periodic wall directions need matching conditions and extents in both domains"));
            }
        }
        let cells = (a.grid.len() as u64).saturating_add(b.grid.len() as u64);
        limits.check_cells([cells, 1, 1])?;
        if cells.saturating_mul(a.steps) > limits.max_cell_updates {
            return Err(Error::Limit("both circuits together exceed max_cell_updates".into()));
        }
        let mut bytes = 0u64;
        let mut frames = 0u64;
        for setup in setups {
            if setup.steady.is_some() {
                return Err(invalid("per-circuit steady/accelerated capacity is unsupported; use bounded transient steps"));
            }
            let thermal = setup.thermal.as_ref().ok_or_else(|| invalid(format!("step `{}` needs thermal settings", setup.step)))?;
            if thermal.solids.len() != 1
                || thermal.solids.first().is_none_or(|s| s.body != wall || s.capacity_scale != 1.0)
                || !thermal.contacts.is_empty()
            {
                return Err(invalid(format!(
                    "step `{}` must conduct only the complete shared wall `{wall}`, with physical capacity and no contacts",
                    setup.step
                )));
            }
            let memory = Memory::of(setup, crate::Choice::Cpu)?;
            bytes = bytes.saturating_add(memory.total());
            frames = frames.saturating_add(memory.frame_bytes);
        }
        // Preparation temporarily holds two SolidHeat copies. The second is dropped below;
        // additional cell maps, interface metadata and scratch are bounded conservatively.
        let count = u64::from(a.frames).saturating_add(1);
        let groups: u64 = setups.iter().map(|s| 6 + s.obstacles.iter().map(|o| o.names.len() as u64).sum::<u64>()).sum();
        let coupling_bytes = cells.saturating_mul(256).saturating_add(count.saturating_mul(4096 + 32 * groups));
        bytes = bytes.saturating_add(coupling_bytes);
        if bytes > limits.max_memory || frames > limits.max_frame_bytes {
            return Err(Error::Limit(format!(
                "coupled preparation needs at most {bytes} bytes and {frames} frame bytes; above the combined memory cap"
            )));
        }
        let ca = a.domain.solids().next().ok_or_else(|| invalid("missing first wall definition"))?;
        let cb = b.domain.solids().next().ok_or_else(|| invalid("missing second wall definition"))?;
        if ca != cb {
            return Err(invalid(format!("wall `{wall}` needs the same material, power and limit in both domains")));
        }
        Ok(CoupledMemory { cells, preparation_bytes: bytes, frame_bytes: frames, coupling_bytes, bound_bytes: limits.max_memory })
    }

    pub fn new(setups: [&Setup; 2], obstacles: [&[Obstacle]; 2], wall: &str, initial: f64, limits: &Limits) -> Result<Self> {
        Self::estimate(setups, wall, initial, limits)?;
        let [a, b] = setups;
        let mut flows = [Flow::new(a, obstacles[0])?, Flow::new(b, obstacles[1])?];
        for cell in 0..a.grid.len() {
            if flows[0].is_fluid(cell) && b.grid.cell_at(a.grid.centre(cell)).is_some_and(|other| flows[1].is_fluid(other)) {
                return Err(invalid(format!("fluid occupancy overlaps between `{}` and `{}`", a.step, b.step)));
            }
        }
        let mut walls = [flows[0].take_wall()?, flows[1].take_wall()?];
        if walls[0].cells.is_empty() || walls[0].cells.len() != walls[1].cells.len() {
            return Err(invalid("both domains must contain the same nonempty complete separator"));
        }
        let mut indices = BTreeMap::new();
        for (index, cell) in walls[0].cells.iter().enumerate() {
            indices.insert(key(&a.grid, *cell, a.grid.origin)?, index as u32);
        }
        let mut maps: [Vec<u32>; 2] = [Vec::new(), Vec::new()];
        for side in 0..2 {
            for cell in &walls[side].cells {
                maps[side].push(
                    *indices
                        .get(&key(&setups[side].grid, *cell, a.grid.origin)?)
                        .ok_or_else(|| invalid("separator cell sets differ between domains"))?,
                );
            }
            if walls[side].interface.is_empty() {
                return Err(invalid(format!("step `{}` has no fluid link into the wall", setups[side].step)));
            }
            if walls[side].cell.iter().flat_map(|c| &c.faces).any(|f| matches!(f, Face::Held { .. } | Face::Flux { .. })) {
                return Err(invalid("separator end faces must be insulated"));
            }
        }
        let thermal = [a.thermal.as_ref().ok_or_else(|| invalid("missing heat"))?, b.thermal.as_ref().ok_or_else(|| invalid("missing heat"))?];
        let sides = std::array::from_fn(|side| Side {
            cells: walls[side].cells.clone(),
            map: std::mem::take(&mut maps[side]),
            links: std::mem::take(&mut walls[side].interface),
            heat_scale: thermal[side].heat_scale,
            zero: thermal[side].zero,
            temperature: vec![0.0; walls[side].cells.len()],
        });
        // Discard the second thermal owner, including all of its capacities and sources.
        let [mut wall, _discarded] = walls;
        wall.temperature.fill(fieldcraft_model::units::kelvin_above(initial, sides[0].zero));
        let initial_wall_energy = wall.heat_content() * sides[0].heat_scale * a.time_step;
        if !initial_wall_energy.is_finite() {
            return Err(invalid("initial wall energy cannot be represented"));
        }
        let mut result = Self { flows, sides, wall, dt: a.time_step, initial_wall_energy };
        result.publish()?;
        Ok(result)
    }

    fn publish(&mut self) -> Result<()> {
        let first_zero = self.sides[0].zero;
        for side in 0..2 {
            let data = &mut self.sides[side];
            for (slot, k) in data.temperature.iter_mut().zip(&data.map) {
                *slot = self.wall.temperature.get(*k as usize).copied().ok_or_else(|| invalid("missing shared wall temperature"))?;
            }
            self.flows[side].set_shared_temperature(&data.cells, &data.temperature, first_zero - data.zero)?;
        }
        Ok(())
    }

    pub fn step(&mut self) -> Result<Exchange> {
        for flow in &mut self.flows {
            flow.begin_step(true)?;
        }
        let before = self.wall.heat_content();
        for side in 0..2 {
            let scale = self.sides[side].heat_scale / self.sides[0].heat_scale;
            let shift = self.sides[0].zero - self.sides[side].zero;
            for link in &self.sides[side].links {
                let [_, a, b] = self.flows[side].shared_link(link)?;
                let cell = *self.sides[side].map.get(link.solid as usize).ok_or_else(|| invalid("missing shared wall mapping"))?;
                self.wall.link(cell, scale * (a - b * shift), scale * b);
            }
        }
        if !self.wall.step() {
            return Err(Error::Unstable("shared wall temperature is not finite".into()));
        }
        let mut out = Exchange {
            wall_storage: (self.wall.heat_content() - before) * self.sides[0].heat_scale,
            wall_power: self.wall.cell.iter().map(|c| c.source).sum::<f64>() * self.sides[0].heat_scale,
            ..Exchange::default()
        };
        for side in 0..2 {
            let shift = self.sides[0].zero - self.sides[side].zero;
            for link in &self.sides[side].links {
                let [pop, a, b] = self.flows[side].shared_link(link)?;
                let cell = *self.sides[side].map.get(link.solid as usize).ok_or_else(|| invalid("missing shared wall mapping"))?;
                let temperature = self.wall.effective.get(cell as usize).copied().ok_or_else(|| invalid("missing shared wall temperature"))? + shift;
                let heat = a - b * temperature;
                self.flows[side].apply_shared_link(link, pop, heat)?;
                out.into_wall[side] += heat * self.sides[side].heat_scale;
            }
            self.flows[side].finish_step()?;
            out.open[side] = self.flows[side].open_energy() * self.sides[side].heat_scale;
            out.external[side] = self.flows[side].heat().iter().sum::<f64>() * self.sides[side].heat_scale - out.into_wall[side];
        }
        self.publish()?;
        out.time = self.flows[0].steps_done() as f64 * self.dt;
        out.wall_residual = out.wall_storage - out.wall_power - out.into_wall.iter().sum::<f64>();
        Ok(out)
    }

    /// Stored energy above the fluid lattices' zeros, plus wall change from its initial state.
    /// Energy changes (not this arbitrary reference) enter the combined budget.
    pub fn energy(&self) -> f64 {
        self.energies().iter().sum()
    }

    pub(crate) fn energies(&self) -> [f64; 3] {
        [
            self.flows[0].scalar_content() * self.sides[0].heat_scale * self.dt,
            self.flows[1].scalar_content() * self.sides[1].heat_scale * self.dt,
            self.wall.heat_content() * self.sides[0].heat_scale * self.dt - self.initial_wall_energy,
        ]
    }

    pub fn fields(&mut self, circuit: usize) -> Result<Fields> {
        self.flows.get_mut(circuit).ok_or_else(|| invalid("circuit index must be 0 or 1"))?.fields()
    }

    pub fn wall_cells(&self) -> usize {
        self.wall.cells.len()
    }
}
