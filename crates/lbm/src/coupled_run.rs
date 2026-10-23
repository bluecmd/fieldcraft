//! Bounded transient orchestration and saved common budgets for the two-circuit CPU path.
use crate::coupled::{CoupledFlow, Exchange};
use crate::{
    Backend, Error, FluidResult, FluidStats, ForceHistory, HeatHistory, Limits, Obstacle, PortProperties, Result, Setup, SolidResult, ZoneHistory,
};
use fieldcraft_model::Model;

/// Average rates (W) over the interval since the previous saved frame. The first is zero.
#[derive(Debug, Clone, Default, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct CoupledBudget {
    pub time: f64,
    pub into_wall: [f64; 2],
    pub fluid_storage: [f64; 2],
    pub wall_storage: f64,
    pub wall_power: f64,
    pub open: [f64; 2],
    pub external: [f64; 2],
    pub residual: f64,
    pub interface_residual: f64,
}

/// One common solve revision; each circuit's saved result carries the identical descriptor.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct CoupledReport {
    pub system: String,
    pub steps: [String; 2],
    pub wall: String,
    pub wall_cells: usize,
    pub initial_temperature: f64,
    /// Only the first step owns the wall capacity and source. The other displays its state.
    pub history: Vec<CoupledBudget>,
}

fn snapshot(result: &mut FluidResult, setup: &Setup, flow: &mut crate::Flow, k: u64) -> Result<()> {
    let frame = crate::solve::frame(setup, flow, k)?;
    result.history_times.push(frame.time);
    result.frames.push(frame);
    for (h, f) in result.forces.iter_mut().zip(Backend::forces(flow)?) {
        h.force.push(f.map(|v| v * setup.force_scale()));
    }
    for (h, q) in result.heat.iter_mut().zip(Backend::heat(flow)?) {
        h.power.push(q * setup.thermal.as_ref().map_or(0.0, |t| t.heat_scale));
    }
    for (h, z) in result.zones.iter_mut().zip(flow.zone_samples()) {
        h.flow.push(z.flow * setup.volume_flow_scale());
        h.pressure.push(z.pressure * setup.pressure_scale());
        if h.fan {
            h.fan_rise.push(z.fan_rise * setup.pressure_scale());
        }
    }
    result.stats.steps = k;
    result.stats.end_time = k as f64 * setup.time_step;
    result.stats.cell_updates = (flow.fluid_cells() as u64).saturating_mul(k);
    Ok(())
}

fn empty(setup: &Setup, flow: &crate::Flow, side: usize) -> FluidResult {
    let n = setup.grid.len();
    let heat_scale = setup.thermal.as_ref().map_or(0.0, |t| t.heat_scale);
    FluidResult {
        coupling: None,
        port_properties: Some(PortProperties {
            density: setup.density,
            specific_heat: setup.thermal.as_ref().map(|t| t.heat_scale * setup.time_step / (setup.density * setup.grid.spacing.powi(3))),
        }),
        step: setup.step.clone(),
        grid: setup.grid,
        solid: (0..n).map(|c| !flow.is_fluid(c)).collect(),
        open_faces: fieldcraft_model::BoxFace::ALL.map(|f| {
            matches!(setup.domain.faces.get(f), fieldcraft_model::FaceCondition::Velocity { .. } | fieldcraft_model::FaceCondition::Pressure { .. })
        }),
        frames: Vec::new(),
        history_times: Vec::new(),
        forces: flow.groups().iter().map(|g| ForceHistory { name: g.name.clone(), body: g.body.clone(), force: Vec::new() }).collect(),
        heat: flow.groups().iter().map(|g| HeatHistory { name: g.name.clone(), body: g.body.clone(), power: Vec::new() }).collect(),
        zones: setup
            .zones
            .iter()
            .map(|z| ZoneHistory { name: z.name.clone(), fan: z.is_fan(), flow: Vec::new(), pressure: Vec::new(), fan_rise: Vec::new() })
            .collect(),
        stats: FluidStats {
            cells: n,
            fluid_cells: flow.fluid_cells(),
            steps: 0,
            time_step: setup.time_step,
            end_time: 0.0,
            tau_plus: setup.tau_plus,
            tau_minus: setup.tau_minus,
            magic: setup.magic,
            collision: setup.collision,
            velocity: setup.velocity,
            lattice_velocity: setup.lattice_velocity,
            mach: setup.mach,
            cell_updates: 0,
            backend: "cpu".into(),
            tau_thermal: setup.thermal.as_ref().map(|t| t.tau),
        },
        warnings: setup.warnings.clone(),
        steady: None,
        temperature_reference: setup.thermal.as_ref().map_or(0.0, |t| t.zero),
        solids: setup
            .thermal
            .iter()
            .flat_map(|t| &t.solids)
            .enumerate()
            .map(|(k, s)| SolidResult {
                body: s.body.clone(),
                cells: flow.conducting().get(k).cloned().unwrap_or_default(),
                power: if side == 0 { s.power * heat_scale } else { 0.0 },
                capacity_scale: 1.0,
                limit: s.limit,
            })
            .collect(),
    }
}

/// Prepare both named steps, adding the system and circuit names to stability/cap refusals.
pub fn prepare_exchanger(model: &Model, name: &str, obstacles: [&[Obstacle]; 2], limits: &Limits) -> Result<[Setup; 2]> {
    let definition = model.heat_exchanger(name).ok_or_else(|| Error::Invalid(format!("no heat exchanger `{name}`")))?;
    let steps = definition.steps();
    let prepare = |side: usize| {
        Setup::new(model, Some(steps[side]), obstacles[side], limits).map_err(|e| {
            let context = |message: String| format!("heat exchanger `{name}` ({} + {}), circuit `{}`: {message}", steps[0], steps[1], steps[side]);
            match e {
                Error::Invalid(message) => Error::Invalid(context(message)),
                Error::Limit(message) => Error::Limit(context(message)),
                other => other,
            }
        })
    };
    Ok([prepare(0)?, prepare(1)?])
}

/// Solve an explicit exchanger definition. The caller supplies original obstacle surfaces for
/// each circuit, as for solve_fluid. CPU only, physical capacities and common output times.
pub fn solve_exchanger(model: &Model, name: &str, obstacles: [&[Obstacle]; 2], limits: &Limits) -> Result<[FluidResult; 2]> {
    let definition = model.heat_exchanger(name).ok_or_else(|| Error::Invalid(format!("no heat exchanger `{name}`")))?;
    let setups = prepare_exchanger(model, name, obstacles, limits)?;
    let mut coupled = CoupledFlow::new([&setups[0], &setups[1]], obstacles, &definition.wall, definition.initial_temperature, limits)?;
    let mut results = std::array::from_fn(|side| empty(&setups[side], &coupled.flows[side], side));
    for side in 0..2 {
        snapshot(&mut results[side], &setups[side], &mut coupled.flows[side], 0)?;
    }
    let mut report = CoupledReport {
        system: name.into(),
        steps: [definition.first.clone(), definition.second.clone()],
        wall: definition.wall.clone(),
        wall_cells: coupled.wall_cells(),
        initial_temperature: definition.initial_temperature,
        history: vec![CoupledBudget::default()],
    };
    let total = setups[0].steps;
    let frames = u64::from(setups[0].frames).min(total).max(1);
    let mut next = 1u64;
    let mut previous = coupled.energies();
    if previous.iter().any(|v| !v.is_finite()) {
        return Err(Error::Unstable("initial coupled energy cannot be represented".into()));
    }
    let mut previous_time = 0.0;
    let mut sum = Exchange::default();
    let mut count = 0u64;
    fieldcraft_model::progress::stage("Coupled fluid steps", Some(total as f64), "");
    for k in 1..=total {
        if k.is_multiple_of(64) || k == 1 {
            fieldcraft_model::progress::check()?;
            fieldcraft_model::progress::count(k as f64);
        }
        let exchange = coupled.step()?;
        count += 1;
        sum.wall_power += exchange.wall_power;
        sum.wall_residual += exchange.wall_residual;
        for side in 0..2 {
            sum.into_wall[side] += exchange.into_wall[side];
            sum.open[side] += exchange.open[side];
            sum.external[side] += exchange.external[side];
        }
        if k == (next * total + frames / 2) / frames {
            let now = coupled.energies();
            let time = k as f64 * setups[0].time_step;
            let duration = time - previous_time;
            let storage = std::array::from_fn::<_, 3, _>(|i| (now[i] - previous[i]) / duration);
            let n = count as f64;
            let residual = storage.iter().sum::<f64>() - (sum.wall_power + sum.open.iter().sum::<f64>() - sum.external.iter().sum::<f64>()) / n;
            if !residual.is_finite() || !sum.wall_residual.is_finite() {
                return Err(Error::Unstable("coupled energy budget cannot be represented".into()));
            }
            report.history.push(CoupledBudget {
                time,
                into_wall: sum.into_wall.map(|v| v / n),
                fluid_storage: [storage[0], storage[1]],
                wall_storage: storage[2],
                wall_power: sum.wall_power / n,
                open: sum.open.map(|v| v / n),
                external: sum.external.map(|v| v / n),
                residual,
                interface_residual: sum.wall_residual / n,
            });
            for side in 0..2 {
                snapshot(&mut results[side], &setups[side], &mut coupled.flows[side], k)?;
            }
            previous = now;
            previous_time = time;
            sum = Exchange::default();
            count = 0;
            next += 1;
        }
    }
    for (side, result) in results.iter_mut().enumerate() {
        let fields = coupled.flows[side].fields()?;
        let spread = fields
            .density
            .iter()
            .enumerate()
            .filter(|(cell, _)| coupled.flows[side].is_fluid(*cell))
            .map(|(_, rho)| (rho - 1.0).abs())
            .fold(0.0, f64::max);
        if spread > crate::solve::COMPRESSIBILITY_WARNING {
            result.warnings.push(format!(
                "the density varies by {:.1}% in circuit `{}`: weak compressibility is visible; reduce the common time step or refine the grids",
                spread * 100.0,
                result.step
            ));
        }
        for name in coupled.flows[side].empty_obstacles() {
            result.warnings.push(format!("obstacle `{name}` covers no cell centre in circuit `{}`", result.step));
        }
        result.coupling = Some(report.clone());
    }
    Ok(results)
}
