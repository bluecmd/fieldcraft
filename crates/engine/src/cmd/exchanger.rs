//! Explicit coupled-system definitions, CPU solve, and immutable saved budget queries.
use super::{name, opt};
use crate::requirement::Requirement;
use crate::{CommandKind, CommandSpec, EngineError, Result, Session, StepResult};
use serde_json::{Value, json};

pub const CREATE: CommandSpec = CommandSpec {
    id: "model.exchanger.create",
    label: "New Heat Exchanger",
    params: r#"{"name":"HX","first":"AirFlow","second":"CoolantFlow","wall":"Separator","initial_temperature":20} — pair two existing fluid steps through one conducting wall; temperatures follow display units or explicit units; CPU + CPU and physical transient capacities initially"#,
    form: Some(super::forms::EXCHANGER),
    kind: CommandKind::Model,
    requires: &[Requirement::Step],
    run: |s, p| {
        let n = name(p)?;
        s.model.add_heat_exchanger(n, definition(p)?)?;
        Ok(json!({"name":n}))
    },
};
pub const EDIT: CommandSpec = CommandSpec {
    id: "model.exchanger.edit",
    label: "Edit Heat Exchanger",
    params: CREATE.params,
    form: Some(super::forms::EXCHANGER),
    kind: CommandKind::Model,
    requires: &[Requirement::Step],
    run: |s, p| {
        let n = name(p)?;
        s.model.replace_heat_exchanger(n, definition(p)?)?;
        Ok(json!({"name":n}))
    },
};
pub const DELETE: CommandSpec = CommandSpec {
    id: "model.exchanger.delete",
    label: "Delete Heat Exchanger",
    params: r#"{"name":"HX"} — removes the coupled-system definition and releases its two steps"#,
    form: Some(super::forms::EXCHANGER_NAME),
    kind: CommandKind::Model,
    requires: &[Requirement::Step],
    run: |s, p| {
        s.model.remove_heat_exchanger(name(p)?)?;
        Ok(Value::Null)
    },
};
pub const SOLVE: CommandSpec = CommandSpec {
    id: "solve.exchanger",
    label: "Solve Heat Exchanger",
    params: r#"{"name":"HX" (the only one by default),"backend":"cpu","threads":4} — advances both fluid states through one shared conducting wall, with physical capacity and power counted once; identical spacing, aligned grids, common dt, length and frames; retains both named circuit results and common average-rate energy budgets; steady acceleration and GPU/mixed backends are refused"#,
    form: Some(super::forms::EXCHANGER_SOLVE),
    kind: CommandKind::Model,
    requires: &[Requirement::FluidDomain, Requirement::Step],
    run: solve,
};
pub const RESULTS: CommandSpec = CommandSpec {
    id: "results.exchanger",
    label: "Heat Exchanger Budgets",
    params: r#"{"name":"HX" (the displayed circuit's system by default),"frame":10 (the displayed frame by default)} — returns both saved circuit ports and one shared-wall budget: rates averaged since the previous frame, fluid/wall storage, wall power, interface transfers, external losses and net discrete open-boundary exchange, with signed residuals; independent directional advective port duties remain separate; save/reopen needs no solve"#,
    form: Some(super::forms::EXCHANGER_RESULTS),
    kind: CommandKind::Session,
    requires: &[Requirement::Result],
    run: results,
};

fn definition(p: &Value) -> Result<fieldcraft_model::HeatExchanger> {
    let mut fields = p.as_object().cloned().ok_or_else(|| EngineError::Params("exchanger parameters must be an object".into()))?;
    fields.remove("name");
    serde_json::from_value(Value::Object(fields)).map_err(|e| EngineError::Params(format!("heat exchanger: {e}")))
}

fn system(s: &Session, p: &Value) -> Result<String> {
    if let Some(name) = opt::<String>(p, "name")? {
        return Ok(name);
    }
    let mut names = s.model.heat_exchangers().keys();
    let first = names.next().ok_or_else(|| EngineError::Params("create a heat exchanger with model.exchanger.create".into()))?;
    if names.next().is_some() {
        return Err(EngineError::Params("choose a heat exchanger by name".into()));
    }
    Ok(first.clone())
}

pub(super) fn solve(s: &mut Session, p: &Value) -> Result<Value> {
    let name = system(s, p)?;
    let e = s.model.heat_exchanger(&name).cloned().ok_or_else(|| EngineError::Params(format!("no heat exchanger `{name}`")))?;
    if opt::<String>(p, "backend")?.is_some_and(|b| b != "cpu") {
        return Err(EngineError::Params(format!("heat exchanger `{name}` supports CPU + CPU only; GPU and mixed backends are not verified")));
    }
    let mut lp = p.clone();
    lp["step"] = json!(e.first);
    lp["backend"] = json!("cpu");
    let (limits, _) = super::solve::fluid_limits(s, &lp)?;
    let (_, first) = fieldcraft_lbm::fluid_step(&s.model, Some(&e.first))?;
    let (_, second) = fieldcraft_lbm::fluid_step(&s.model, Some(&e.second))?;
    let a = super::solve::obstacles(s, first)?;
    let b = super::solve::obstacles(s, second)?;
    let setups = fieldcraft_lbm::prepare_exchanger(&s.model, &name, [&a, &b], &limits)?;
    let memory = fieldcraft_lbm::coupled::CoupledFlow::estimate([&setups[0], &setups[1]], &e.wall, e.initial_temperature, &limits)?;
    if opt::<bool>(p, "estimate")? == Some(true) {
        return Ok(json!({"memory":memory,"system":name,"backend":"cpu"}));
    }
    let solved = super::solve::with_threads(super::solve::threads(s, p)?, || fieldcraft_lbm::solve_exchanger(&s.model, &name, [&a, &b], &limits))?;
    let display = s.units().clone();
    let circuits: Vec<Value> = solved
        .iter()
        .map(|r| {
            let mut summary = super::solve::fluid_summary(r, &display);
            crate::units::convert_reply("solve.fluid", &mut summary, crate::units::si(), &display).map(|()| summary)
        })
        .collect::<Result<_>>()?;
    let last = solved.first().map_or(0, |r| r.frames.len().saturating_sub(1));
    for result in solved {
        s.store_result(StepResult::Fluid(result));
    }
    let mut out = results(s, &json!({"name":name,"frame":last}))?;
    out["circuits"] = json!(circuits);
    out["memory"] = json!(memory);
    Ok(out)
}

fn results(s: &mut Session, p: &Value) -> Result<Value> {
    let wanted = opt::<String>(p, "name")?;
    let shown = s.results_view.as_ref();
    let owner = shown.and_then(|v| s.latest_result(&v.step)).map(|(r, _)| r.as_ref());
    let from_view = owner.and_then(|r| match r {
        StepResult::Fluid(f) => f.coupling.as_ref(),
        _ => None,
    });
    let report = if let Some(name) = wanted.as_deref() {
        s.result_steps().filter_map(|(step, _)| s.latest_result(step)).find_map(|(r, _)| match r.as_ref() {
            StepResult::Fluid(f) => f.coupling.as_ref().filter(|c| c.system == name),
            _ => None,
        })
    } else {
        from_view
    }
    .cloned()
    .ok_or_else(|| EngineError::Params("no saved coupled exchanger result for that name".into()))?;
    let frame = opt::<usize>(p, "frame")?.or_else(|| shown.and_then(|v| v.frame)).unwrap_or_else(|| report.history.len().saturating_sub(1));
    let budget = report.history.get(frame).ok_or_else(|| EngineError::Params(format!("no coupled frame {frame}")))?;
    let mut circuits = Vec::new();
    let mut stale = false;
    for name in &report.steps {
        let (result, current) = s.latest_result(name).ok_or_else(|| EngineError::Params(format!("no saved circuit `{name}`")))?;
        stale |= !current;
        let StepResult::Fluid(flow) = result.as_ref() else {
            return Err(EngineError::Params("coupled circuit has no fluid result".into()));
        };
        if flow.coupling.as_ref() != Some(&report) {
            return Err(EngineError::Params("the circuit results are from different coupled solves; solve the exchanger again".into()));
        }
        circuits.push(json!({"step":name,"ports":crate::results::fluid_ports(flow,frame,s.units())}));
    }
    Ok(json!({"system":report.system,"steps":report.steps,"wall":report.wall,"wall_cells":report.wall_cells,
        "capacity_owner":report.steps[0],"mode":"physical_transient","frame":frame,"stale":stale,"budget":budget,"circuits":circuits,
        "open_exchange_convention":"net scalar energy inserted by discrete open-boundary resets; includes advection and conduction; port diagnostics report directional advective transport separately",
        "budget_interval":"average rates since previous saved frame; zero at the initial frame"}))
}
