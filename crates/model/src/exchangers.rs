//! An explicit pair of fluid steps exchanging heat through one named conducting body.
use crate::{Error, Model, Result, check_name, finite, insert_new, remove_existing, replace_existing};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

/// The first supported coupled system: two CPU fluid steps and one shared separator.
/// Steps retain their circuit names and independent materials. Common stepping, occupancy
/// and stability are validated by the coupled solver before constructing its two lattices.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct HeatExchanger {
    pub first: String,
    pub second: String,
    pub wall: String,
    /// One physical wall temperature, in kelvin; fluid temperatures belong to each domain.
    pub initial_temperature: f64,
}

impl HeatExchanger {
    pub fn steps(&self) -> [&str; 2] {
        [&self.first, &self.second]
    }
}

impl Model {
    pub fn heat_exchangers(&self) -> &BTreeMap<String, HeatExchanger> {
        &self.heat_exchangers
    }
    pub fn heat_exchanger(&self, name: &str) -> Option<&HeatExchanger> {
        self.heat_exchangers.get(name)
    }
    pub fn exchanger_of_step(&self, step: &str) -> Option<(&str, &HeatExchanger)> {
        self.heat_exchangers.iter().find(|(_, e)| e.steps().contains(&step)).map(|(n, e)| (n.as_str(), e))
    }
    fn check_heat_exchanger(&self, name: &str, e: &HeatExchanger) -> Result<()> {
        check_name("heat exchanger", name)?;
        check_name("wall body", &e.wall)?;
        finite("initial wall temperature", e.initial_temperature)?;
        if e.initial_temperature < 0.0 || e.first == e.second {
            return Err(Error::Invalid("a heat exchanger needs two distinct fluid steps and a nonnegative kelvin wall temperature".into()));
        }
        for step in e.steps() {
            let domain = self
                .step(step)
                .and_then(|s| s.procedure.fluid_domain())
                .and_then(|n| self.fluid_domain(n))
                .ok_or_else(|| Error::Invalid(format!("heat exchanger `{name}`: `{step}` must name an existing fluid step")))?;
            if !domain.solids().any(|solid| solid.body == e.wall) {
                return Err(Error::Invalid(format!("heat exchanger `{name}`: step `{step}` must conduct wall `{}`", e.wall)));
            }
            if let Some((other, _)) = self.heat_exchangers.iter().find(|(n, h)| n.as_str() != name && h.steps().contains(&step)) {
                return Err(Error::InUse(format!("step `{step}` already belongs to heat exchanger `{other}`")));
            }
        }
        Ok(())
    }
    pub fn add_heat_exchanger(&mut self, name: &str, e: HeatExchanger) -> Result<()> {
        self.check_heat_exchanger(name, &e)?;
        insert_new(&mut self.heat_exchangers, "heat exchanger", name, e)
    }
    pub fn replace_heat_exchanger(&mut self, name: &str, e: HeatExchanger) -> Result<()> {
        self.check_heat_exchanger(name, &e)?;
        replace_existing(&mut self.heat_exchangers, "heat exchanger", name, e)
    }
    pub fn remove_heat_exchanger(&mut self, name: &str) -> Result<()> {
        remove_existing(&mut self.heat_exchangers, "heat exchanger", name).map(|_| ())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{ConductingSolid, Directional, Fluid, FluidDomain, FluidThermal, Material, Procedure, Step, Thermal, rename::Kind};
    use serde_json::json;

    fn model() -> Model {
        let mut m = Model::default();
        m.add_material(
            "Fluid",
            Material {
                density: Some(1.0),
                fluid: Some(Fluid::Newtonian { viscosity: 0.1 }),
                thermal: Some(Thermal { conductivity: Some(Directional::Isotropic(0.1)), specific_heat: Some(1.0), ..Default::default() }),
                ..Default::default()
            },
        )
        .unwrap();
        for (name, min, max) in [("A", [0.0; 3], [4.0, 3.0, 2.0]), ("B", [0.0, 2.0, 0.0], [4.0, 5.0, 2.0])] {
            let domain: FluidDomain = serde_json::from_value(json!({"material":"Fluid","min":min,"max":max,"spacing":1})).unwrap();
            let domain = FluidDomain {
                obstacles: vec!["Wall".into()],
                thermal: Some(FluidThermal {
                    solids: vec![ConductingSolid { body: "Wall".into(), material: "Fluid".into(), power: 0.0, limit: None }],
                    ..Default::default()
                }),
                ..domain
            };
            m.add_fluid_domain(name, domain).unwrap();
            m.add_step(
                Step::new(
                    name,
                    Procedure::Fluid {
                        domain: name.into(),
                        steps: Some(20),
                        time_step: Some(1.0),
                        end_time: None,
                        frames: 2,
                        magic: None,
                        collision: None,
                        steady: None,
                    },
                ),
                None,
            )
            .unwrap();
        }
        m
    }

    #[test]
    fn exchanger_dependencies_roundtrip_and_track_both_step_inputs() {
        let mut m = model();
        let e = HeatExchanger { first: "A".into(), second: "B".into(), wall: "Wall".into(), initial_temperature: 300.0 };
        m.add_heat_exchanger("HX", e.clone()).unwrap();
        let before = serde_json::to_value(m.step_inputs("A").unwrap()).unwrap();
        let mut b = m.step("B").unwrap().clone();
        if let Procedure::Fluid { steps, .. } = &mut b.procedure {
            *steps = Some(30);
        }
        m.replace_step(b).unwrap();
        assert_ne!(before, serde_json::to_value(m.step_inputs("A").unwrap()).unwrap());
        assert!(m.remove_step("B").unwrap_err().to_string().contains("HX"));
        let mut b = m.step("B").unwrap().clone();
        b.procedure = Procedure::Static {};
        assert!(m.replace_step(b).is_err());
        m.rename(Kind::Step, "B", "Coolant").unwrap();
        assert_eq!(m.heat_exchanger("HX").unwrap().second, "Coolant");
        let json = serde_json::to_value(m.without_mesh()).unwrap();
        let opened: Model = serde_json::from_value(json).unwrap();
        assert_eq!(opened, m);
        assert!(m.add_heat_exchanger("Other", e).is_err());
        m.remove_heat_exchanger("HX").unwrap();
        m.remove_step("Coolant").unwrap();
    }

    #[test]
    fn bad_temperatures_names_and_references_are_refused() {
        let mut m = model();
        for e in [
            HeatExchanger { first: "A".into(), second: "A".into(), wall: "Wall".into(), initial_temperature: 300.0 },
            HeatExchanger { first: "A".into(), second: "Missing".into(), wall: "Wall".into(), initial_temperature: 300.0 },
            HeatExchanger { first: "A".into(), second: "B".into(), wall: "Other".into(), initial_temperature: 300.0 },
            HeatExchanger { first: "A".into(), second: "B".into(), wall: "Wall".into(), initial_temperature: f64::NAN },
            HeatExchanger { first: "A".into(), second: "B".into(), wall: "Wall".into(), initial_temperature: -1.0 },
        ] {
            assert!(m.add_heat_exchanger("HX", e).is_err());
        }
    }
}
