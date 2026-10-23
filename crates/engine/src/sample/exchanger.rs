//! Original short parallel-plate exchanger. Realistic prescribed constant properties, laminar
//! inlets, physical capacity. A transient demonstration, not the refined #700 acceptance case.
use fieldcraft_geometry::Shape;
use serde_json::{Value, json};

pub const DESIGN: &str = r#"{"commands":[
    {"command":"sketch.create","params":{"plane":"XY","name":"Plate"}},
    {"command":"sketch.line","params":{"points":[[0,2],[10,2],[10,3],[0,3]],"closed":true}},
    {"command":"sketch.finish","params":{}},
    {"command":"solid.extrude","params":{"distance":2,"body_name":"Separator"}}
]}"#;

pub(super) fn script(_: &[Shape]) -> Value {
    json!([
        {"command":"model.units","params":{"system":"mm-t-s-N-MPa"}},
        {"command":"model.material.create","params":{"name":"Air","density":"1.2 kg/m³","fluid":{"type":"newtonian","viscosity":"0.000018 Pa·s"},"thermal":{"conductivity":"0.024 W/(m K)","specific_heat":"1000 J/(kg K)"}}},
        {"command":"model.material.create","params":{"name":"Coolant","density":"1000 kg/m³","fluid":{"type":"newtonian","viscosity":"0.001 Pa·s"},"thermal":{"conductivity":"0.6 W/(m K)","specific_heat":"4200 J/(kg K)"}}},
        {"command":"model.material.create","params":{"name":"Steel","density":"8000 kg/m³","thermal":{"conductivity":"15 W/(m K)","specific_heat":"500 J/(kg K)"}}},
        {"command":"model.fluid_domain.create","params":{"name":"AirPassage","material":"Air","min":[0,0,0],"max":[10,3,2],"spacing":0.25,
            "faces":{"x_min":{"type":"velocity","velocity":[1,0,0]},"x_max":{"type":"pressure","pressure":0},"z_min":{"type":"periodic"},"z_max":{"type":"periodic"}},
            "obstacles":["Separator"],"thermal":{"initial_temperature":20,"faces":{"x_min":{"type":"temperature","value":20}},"solids":[{"body":"Separator","material":"Steel"}]}}},
        {"command":"model.fluid_domain.create","params":{"name":"CoolantPassage","material":"Coolant","min":[0,2,0],"max":[10,5,2],"spacing":0.25,
            "faces":{"x_max":{"type":"velocity","velocity":[-0.5,0,0]},"x_min":{"type":"pressure","pressure":0},"z_min":{"type":"periodic"},"z_max":{"type":"periodic"}},
            "obstacles":["Separator"],"thermal":{"initial_temperature":50,"faces":{"x_max":{"type":"temperature","value":50}},"solids":[{"body":"Separator","material":"Steel"}]}}},
        {"command":"model.step.create","params":{"name":"AirFlow","procedure":{"type":"fluid","domain":"AirPassage","end_time":5,"time_step":0.000625,"frames":10,"collision":"regularized"}}},
        {"command":"model.step.create","params":{"name":"CoolantFlow","procedure":{"type":"fluid","domain":"CoolantPassage","end_time":5,"time_step":0.000625,"frames":10,"collision":"regularized"}}},
        {"command":"model.exchanger.create","params":{"name":"Counterflow","first":"AirFlow","second":"CoolantFlow","wall":"Separator","initial_temperature":35}}
    ])
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Session, StepResult};

    #[test]
    fn coupled_commands_keep_both_sides_and_one_saved_budget() {
        let mut s = Session::default();
        s.execute("file.sample", &json!({"name":"heat-exchanger"})).unwrap();
        for (name, domain) in [("AirFlow", "AirPassage"), ("CoolantFlow", "CoolantPassage")] {
            s.execute("model.step.edit",&json!({"name":name,"procedure":{"type":"fluid","domain":domain,"steps":40,"time_step":0.000625,"frames":2,"collision":"regularized"}})).unwrap();
        }
        assert!(s.execute("solve.fluid", &json!({"step":"AirFlow"})).unwrap_err().to_string().contains("solve.exchanger"));
        assert!(s.execute("solve.exchanger", &json!({"backend":"gpu"})).unwrap_err().to_string().contains("CPU + CPU"));
        let r = s.execute("solve.exchanger", &json!({})).unwrap();
        assert_eq!(r["system"], "Counterflow");
        assert_eq!(r["wall_cells"], 1280);
        assert!(r["budget"]["residual"].as_f64().unwrap().abs() < 1e-6, "{r}");
        assert_eq!(r["circuits"].as_array().unwrap().len(), 2);
        let mut results = Vec::new();
        for name in ["AirFlow", "CoolantFlow"] {
            let (result, current) = s.latest_result(name).unwrap();
            assert!(current);
            let StepResult::Fluid(f) = result.as_ref() else { panic!() };
            assert_eq!(f.frames.len(), 3);
            assert!(f.coupling.is_some());
            results.push(fieldcraft_io::results::Stored { definition: [0; 32], result: fieldcraft_io::results::Solved::Fluid(f.clone()) });
        }
        let refs: Vec<_> = results
            .iter()
            .map(|r| (r.result.step(), fieldcraft_io::results::StoredRef { definition: r.definition, result: r.result.as_ref() }))
            .collect();
        let bytes = fieldcraft_io::results::encode(&refs).unwrap();
        assert!(fieldcraft_io::results::encode(&refs[..1]).unwrap_err().to_string().contains("both circuit results"));
        let fieldcraft_io::results::Decoded::Results(decoded) = fieldcraft_io::results::decode(&bytes).unwrap() else { panic!() };
        assert_eq!(decoded, results);
        s.execute("solve.exchanger", &json!({"threads":1})).unwrap();
        let first = s.latest_result("AirFlow").unwrap().0.clone();
        s.execute("solve.exchanger", &json!({"threads":4})).unwrap();
        assert_eq!(first, s.latest_result("AirFlow").unwrap().0.clone(), "coupled reductions must be independent of CPU thread count");
        s.execute("results.surface", &json!({"step":"AirFlow","field":"temperature","show":true})).unwrap();
        let result = s.latest_result("AirFlow").unwrap().0;
        let base = crate::view::display(&s).0;
        let bodies: Vec<_> = s.design.as_ref().unwrap().bodies.iter().map(|b| b.name.as_str()).collect();
        let plot = crate::flow::plot_with_surfaces(s.results_view.as_ref().unwrap(), result, &s.traced, &s, &base, &bodies).unwrap();
        let temperatures: Vec<f64> = plot.wall_values.iter().filter_map(|(_, value)| *value).collect();
        assert!(!temperatures.is_empty());
        let StepResult::Fluid(flow) = result.as_ref() else { panic!() };
        let frame = flow.frames.last().unwrap();
        let recorded: Vec<_> = flow.solids[0].cells.iter().map(|c| f64::from(frame.temperature[*c as usize]) + flow.temperature_reference).collect();
        let low = recorded.iter().copied().fold(f64::INFINITY, f64::min);
        let high = recorded.iter().copied().fold(f64::NEG_INFINITY, f64::max);
        assert!(
            temperatures.iter().all(|t| *t >= low - 1e-10 && *t <= high + 1e-10),
            "surface colours must come from the saved conducting cells: {temperatures:?}, range {low}..{high}"
        );
        s.execute("results.frame", &json!({"step":"CoolantFlow","frame":1})).unwrap();
        let report = s.execute("results.exchanger", &json!({})).unwrap();
        assert_eq!(report["frame"], 1);
        assert!(!report["stale"].as_bool().unwrap());
        s.execute(
            "model.material.edit",
            &json!({"name":"Steel","density":"8100 kg/m³","thermal":{"conductivity":"15 W/(m K)","specific_heat":"500 J/(kg K)"}}),
        )
        .unwrap();
        assert!(s.execute("results.exchanger", &json!({})).unwrap()["stale"].as_bool().unwrap());
        s.execute("model.exchanger.delete", &json!({"name":"Counterflow"})).unwrap();
        s.execute("solve.fluid", &json!({"step":"AirFlow"})).unwrap();
        assert!(s.latest_result("CoolantFlow").is_none(), "replacing one released circuit must retire the old coupled group");
        fieldcraft_io::results::encode(&s.stored_results()).unwrap();
    }
}
