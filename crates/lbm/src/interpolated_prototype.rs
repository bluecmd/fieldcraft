//! Local numerical experiment for #798; deliberately unavailable to production callers.
//! Linear BFL: Bouzidi, Firdaouss & Lallemand (2001), DOI 10.1063/1.1399290;
//! equations checked against Marson et al., arXiv:2009.04604v3, eqs. (13a,b).
//! Momentum exchange: Mei et al. (2002), DOI 10.1103/PhysRevE.65.041203.
//! Optional rest-population compensation: Sanjeevi, Zarghami & Padding (2018),
//! DOI 10.1103/PhysRevE.97.043305, section IV B case 1. Their pressure-artifact
//! warning is why both variants are measured. Neither is accepted here yet.
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

#[path = "../tests/common/mod.rs"]
mod common;

use super::*;
use common::{domain, model, obstacle, sphere, step};
use fieldcraft_mesh::predicates::{Point3, orient3d};
use fieldcraft_mesh::surface::BoundarySurface;
use fieldcraft_model::BoxFace;

#[derive(Clone, Copy)]
struct Cut {
    cell: usize,
    direction: usize,
    group: usize,
    fraction: f64,
    second: usize,
}

/// Predicate decisions are exact; floating point is used only to locate a certified crossing.
/// Exhaustive bounded search is sufficient for this experiment; production needs the BVH.
fn fraction(surface: &BoundarySurface, start: [f64; 3], end: [f64; 3]) -> Option<f64> {
    let p = Point3::new(start[0], start[1], start[2]).unwrap();
    let s = Point3::new(end[0], end[1], end[2]).unwrap();
    let mut best: Option<f64> = None;
    for tri in surface.triangles() {
        let v = tri.map(|v| surface.point3(v).unwrap());
        let [a, b, c] = v.map(|v| v.coords());
        if (0..3).any(|k| start[k].min(end[k]) > a[k].max(b[k]).max(c[k]) || start[k].max(end[k]) < a[k].min(b[k]).min(c[k])) {
            continue;
        }
        let ps = orient3d(&v[0], &v[1], &v[2], &p);
        let ss = orient3d(&v[0], &v[1], &v[2], &s);
        if ps == ss {
            continue;
        }
        let edges = [orient3d(&p, &s, &v[0], &v[1]), orient3d(&p, &s, &v[1], &v[2]), orient3d(&p, &s, &v[2], &v[0])];
        let mut nonzero = edges.into_iter().filter(|s| !s.is_zero());
        if let Some(first) = nonzero.next()
            && nonzero.any(|s| s != first)
        {
            continue;
        }
        let ab = [0, 1, 2].map(|k| b[k] - a[k]);
        let ac = [0, 1, 2].map(|k| c[k] - a[k]);
        let normal = [ab[1] * ac[2] - ab[2] * ac[1], ab[2] * ac[0] - ab[0] * ac[2], ab[0] * ac[1] - ab[1] * ac[0]];
        let dot = |x: [f64; 3]| (0..3).map(|k| x[k] * normal[k]).sum::<f64>();
        let t = dot([0, 1, 2].map(|k| a[k] - start[k])) / dot([0, 1, 2].map(|k| end[k] - start[k]));
        assert!(t.is_finite() && (-1e-12..=1.0 + 1e-12).contains(&t), "certified crossing {t}");
        let t = t.clamp(0.0, 1.0);
        if best.is_none_or(|b| t < b) {
            best = Some(t);
        }
    }
    best
}

fn cuts(flow: &Flow, surface: &BoundarySurface, body: &str) -> Vec<Cut> {
    flow.links
        .iter()
        .filter_map(|link| {
            let Rule::Bounce { group, du } = link.rule else { return None };
            if flow.groups[group as usize].body.as_deref() != Some(body) {
                return None;
            }
            assert_eq!(du, 0.0);
            let (cell, q) = (link.cell as usize, link.q as usize);
            let start = flow.grid.centre(cell);
            let end = [0, 1, 2].map(|a| start[a] - c(q)[a] * flow.grid.spacing);
            let fraction = fraction(surface, start, end).expect("fluid-to-solid link intersects the surface");
            let ijk = flow.grid.coords(cell);
            let next = [0, 1, 2].map(|a| {
                let i = ijk[a] as i64 + i64::from(C[q][a]);
                assert!(flow.periodic[a] || (0..flow.grid.cells[a] as i64).contains(&i));
                i.rem_euclid(flow.grid.cells[a] as i64) as usize
            });
            let second = flow.grid.index(next);
            assert!(fraction >= 0.5 || flow.is_fluid(second), "narrow gap needs an explicit policy");
            Some(Cut { cell, direction: q, group: group as usize, fraction, second })
        })
        .collect()
}

fn reflected(q: f64, out: f64, second: f64, same: f64) -> f64 {
    if q < 0.5 { 2.0 * q * out + (1.0 - 2.0 * q) * second } else { (out + (2.0 * q - 1.0) * same) / (2.0 * q) }
}

fn advance(flow: &mut Flow, cuts: &[Cut], conserve: bool) {
    // The prototype supports only constant-viscosity TRT, static obstacles, no heat/SGS.
    assert_eq!(flow.collision, Collision::Trt);
    assert!(flow.thermal.is_none() && flow.surface.is_none() && flow.walls.is_none());
    flow.targets().unwrap();
    let u2 = flow.collide();
    assert!(u2 <= MAX_LATTICE_SPEED * MAX_LATTICE_SPEED, "step {} unstable: {u2}", flow.steps_done);
    flow.stream();
    for cut in cuts {
        let Cut { cell, direction: q, group, fraction, second } = *cut;
        let out = flow.f[opposite(q) * flow.stride + cell];
        let back = reflected(fraction, out, flow.f[opposite(q) * flow.stride + second], flow.f[q * flow.stride + cell]);
        let delta = back - out;
        flow.g[q * flow.stride + cell] = back;
        if conserve {
            flow.g[cell] -= delta;
        }
        for a in 0..3 {
            flow.forces[group][a] -= c(q)[a] * delta;
        }
    }
    flow.open_boundaries();
    std::mem::swap(&mut flow.f, &mut flow.g);
    flow.steps_done += 1;
}

fn sphere_case(radius: usize, phase: [f64; 3], steps: u64) -> (Flow, Vec<Cut>, f64) {
    let side = (16 * radius / 3) as f64;
    let mut d = domain([0.0; 3], [side; 3], 1.0);
    for f in BoxFace::ALL {
        *d.faces.get_mut(f) = FaceCondition::Periodic {};
    }
    d.acceleration = [1e-6 * (3.0 / radius as f64).powi(3), 0.0, 0.0];
    d.obstacles = vec!["Ball".into()];
    let ball = obstacle("Ball", sphere(phase.map(|p| side / 2.0 + p), radius as f64, 4), &["skin"]);
    let m = model(1.0, 5.0 / 6.0, d, step(steps as f64, 1.0, Some(3.0 / 16.0)));
    let limits = crate::Limits { max_cell_updates: 20_000_000_000, ..Default::default() };
    let setup = Setup::new(&m, None, std::slice::from_ref(&ball), &limits).unwrap();
    let mut flow = Flow::new(&setup, std::slice::from_ref(&ball)).unwrap();
    let cuts = cuts(&flow, &ball.surface, "Ball");
    let mass0 = mass(&flow);
    // Caller decides the study duration; this helper only prepares the state.
    assert!(!cuts.is_empty());
    flow.targets().unwrap();
    (flow, cuts, mass0)
}

fn momentum(flow: &Flow) -> [f64; 3] {
    let mut p = [0.0; 3];
    for cell in 0..flow.n {
        if let Some((rho, u)) = flow.moments(cell) {
            for a in 0..3 {
                p[a] += rho * (u[a] - 0.5 * flow.acceleration[a]);
            }
        }
    }
    p
}

// The production diagnostic sums O(19N) terms naively. Use an independent
// compensated sum for a roundoff-scale conservation test.
fn mass(flow: &Flow) -> f64 {
    let (mut sum, mut correction) = (0.0, 0.0);
    for cell in 0..flow.n {
        if !flow.is_fluid(cell) {
            continue;
        }
        for q in 0..Q {
            let term = flow.f[q * flow.stride + cell] - correction;
            let next = sum + term;
            correction = (next - sum) - term;
            sum = next;
        }
    }
    sum
}

#[test]
fn halfway_and_small_fraction_limits() {
    assert_eq!(reflected(0.5, 0.17, 0.23, 0.31), 0.17);
    for q in [0.0, 1e-12, 0.25, 0.5, 0.75, 1.0] {
        assert!((reflected(q, 0.2, 0.2, 0.2) - 0.2).abs() < 1e-16);
        assert!(reflected(q, 0.17, 0.23, 0.31).is_finite());
    }
}

#[test]
fn intersections_include_shared_edges_and_endpoints() {
    let cube = common::cuboid([0.0; 3], [1.0; 3]);
    for (start, end, expected) in [
        ([-0.25, 0.3, 0.7], [0.75, 0.3, 0.7], 0.25),
        ([-0.25, 0.5, 0.5], [0.75, 0.5, 0.5], 0.25),
        ([0.0, 0.3, 0.7], [1.0, 0.3, 0.7], 0.0),
        ([-1.0, 0.3, 0.7], [0.0, 0.3, 0.7], 1.0),
    ] {
        assert_eq!(fraction(&cube, start, end), Some(expected));
    }
    assert_eq!(fraction(&cube, [-1.0, 2.0, 0.5], [1.0, 2.0, 0.5]), None);
}

#[test]
fn thread_count_does_not_change_interpolated_updates() {
    let (flow, cuts, _) = sphere_case(3, [0.125, 0.375, 0.25], 16);
    let run = |threads| {
        parallel::with_threads(threads, || {
            let mut f = flow.clone();
            for _ in 0..16 {
                advance(&mut f, &cuts, true);
            }
            (f.f, f.forces)
        })
    };
    assert_eq!(run(1), run(4));
}

#[test]
fn mass_and_actual_impulse_close_each_step() {
    let (initial, cuts, mass0) = sphere_case(3, [0.25, 0.125, 0.375], 16);
    for conserve in [false, true] {
        let mut flow = initial.clone();
        let (mut max_residual, mut max_mass_drift) = (0.0_f64, 0.0_f64);
        for _ in 0..16 {
            let before = momentum(&flow);
            let before_mass = mass(&flow);
            advance(&mut flow, &cuts, conserve);
            let after = momentum(&flow);
            for a in 0..3 {
                let force: f64 = flow.forces.iter().map(|f| f[a]).sum();
                let residual = after[a] - before[a] - before_mass * flow.acceleration[a] + force;
                max_residual = max_residual.max(residual.abs());
                assert!(residual.abs() < 1e-12, "budget residual {residual:e}");
            }
            if conserve {
                assert!(
                    (mass(&flow) / mass0 - 1.0).abs() < 1e-12,
                    "mass drift {:.3e}; naive {:.3e}",
                    mass(&flow) / mass0 - 1.0,
                    flow.mass() / mass0 - 1.0
                );
            }
            max_mass_drift = max_mass_drift.max((mass(&flow) / mass0 - 1.0).abs());
        }
        println!(
            "conserve={conserve}: max momentum residual={max_residual:.3e}, max relative mass drift={max_mass_drift:.3e}, naive final mass drift={:.3e}",
            flow.mass() / mass0 - 1.0
        );
    }
}

#[test]
#[ignore = "heavy: local #798 design experiment, not a production validation"]
fn sphere_study() {
    let radius: usize = std::env::var("FIELDCRAFT_PROTOTYPE_RADIUS").unwrap_or("3".into()).parse().unwrap();
    assert!([3, 6, 9, 12].contains(&radius));
    let conserve = std::env::var("FIELDCRAFT_PROTOTYPE_CONSERVE").unwrap_or("1".into()) == "1";
    let steps = 3000.max(1500 * (radius as u64 / 3).pow(2));
    for phase in [[0.0; 3], [0.25; 3], [0.5; 3], [0.125, 0.375, 0.25]] {
        let (mut flow, cuts, mass0) = sphere_case(radius, phase, steps);
        let mut previous_k = 0.0;
        for i in 1..=steps {
            advance(&mut flow, &cuts, conserve);
            if i % (steps / 4) == 0 {
                let mass = mass(&flow);
                let rho = mass / flow.fluid_cells as f64;
                let u = (0..flow.n).filter_map(|c| flow.moments(c)).map(|(_, u)| u[0]).sum::<f64>() / flow.n as f64;
                let force: f64 = flow.forces.iter().map(|f| f[0]).sum();
                let k = force * flow.n as f64 / flow.fluid_cells as f64 / (6.0 * std::f64::consts::PI * (5.0 / 6.0) * radius as f64 * u * rho);
                let analytic_fluid = flow.n as f64 - 4.0 / 3.0 * std::f64::consts::PI * (radius as f64).powi(3);
                let analytic_k = k * flow.fluid_cells as f64 / analytic_fluid;
                let balance = force / (mass * flow.acceleration[0]) - 1.0;
                println!(
                    "sphere R={radius} phase={phase:?} conserve={conserve} step={i} K_voxel={k:.9} K_analytic={analytic_k:.9} mass={:.3e} force_balance={balance:.3e} K_drift={:.3e}",
                    mass / mass0 - 1.0,
                    k / previous_k - 1.0
                );
                if i == steps {
                    assert!(balance.abs() < 1e-6, "not steady");
                    assert!((k / previous_k - 1.0).abs() < 1e-6, "K is still drifting");
                    if conserve {
                        assert!((mass / mass0 - 1.0).abs() < 1e-8, "mass conservation failed");
                    }
                    // This is a diagnostic. Accuracy acceptance is deliberately not asserted yet.
                }
                previous_k = k;
            }
        }
    }
}

/// Published benchmark, Schafer & Turek (1996), DOI 10.1007/978-3-322-89849-4_39.
/// Baseline only: phase comparisons must also move the physical channel walls relative
/// to the grid, which this test intentionally does not pretend to do.
#[test]
#[ignore = "heavy: local #798 design experiment, not a production validation"]
fn cylinder_study() {
    let cells = 20.0;
    let dx = 1.0 / cells;
    let dt = 0.05 * dx / 1.5;
    let mut dom = domain([0.0; 3], [22.0, 4.1, dx], dx);
    dom.faces.x_min = FaceCondition::Velocity { velocity: [1.5, 0.0, 0.0], profile: Profile::Parabolic };
    dom.faces.x_max = FaceCondition::Pressure { pressure: 0.0, outflow: Outflow::Uniform };
    dom.faces.z_min = FaceCondition::Periodic {};
    dom.faces.z_max = FaceCondition::Periodic {};
    dom.initial_velocity = [1.0, 0.0, 0.0];
    dom.obstacles = vec!["Cylinder".into()];
    let cyl = obstacle("Cylinder", common::cylinder([2.0, 2.0], 0.5, -dx, 2.0 * dx, 1024), &["side", "bottom", "top"]);
    let m = model(1.0, 0.01, dom, step(120.0, dt, Some(3.0 / 16.0)));
    let limits = crate::Limits { max_cell_updates: 4_000_000_000, ..Default::default() };
    let setup = Setup::new(&m, None, std::slice::from_ref(&cyl), &limits).unwrap();
    let mut flow = Flow::new(&setup, std::slice::from_ref(&cyl)).unwrap();
    let cuts = cuts(&flow, &cyl.surface, "Cylinder");
    let group = flow.groups.iter().position(|g| g.body.as_deref() == Some("Cylinder") && g.name == "side").unwrap();
    let conserve = std::env::var("FIELDCRAFT_PROTOTYPE_CONSERVE").unwrap_or("1".into()) == "1";
    // Force unit rho dx^4 / dt^2 divided by rho U^2 D span / 2, U=D=1.
    let coefficient = 2.0 * dx.powi(3) / dt.powi(2);
    let mut history = Vec::new();
    let mut cd_max = f64::NEG_INFINITY;
    let mut max_mach = 0.0_f64;
    let mut max_density_error = 0.0_f64;
    for i in 1..=setup.steps {
        advance(&mut flow, &cuts, conserve);
        let time = i as f64 * dt;
        if time >= 80.0 {
            cd_max = cd_max.max(coefficient * flow.forces[group][0]);
        }
        if time >= 80.0 && i % 60 == 0 {
            history.push((time, coefficient * flow.forces[group][0], coefficient * flow.forces[group][1]));
            for cell in 0..flow.n {
                if let Some((rho, u)) = flow.moments(cell) {
                    max_mach = max_mach.max((3.0 * u.iter().map(|v| v * v).sum::<f64>()).sqrt());
                    max_density_error = max_density_error.max((rho - 1.0).abs());
                }
            }
        }
        if i % (setup.steps / 4) == 0 {
            println!("cylinder conserve={conserve} time={time:.3}");
        }
    }
    let mean_lift = history.iter().map(|v| v.2).sum::<f64>() / history.len() as f64;
    let crossings: Vec<f64> = history
        .windows(2)
        .filter(|w| w[0].2 < mean_lift && w[1].2 >= mean_lift)
        .map(|w| w[0].0 + (w[1].0 - w[0].0) * (mean_lift - w[0].2) / (w[1].2 - w[0].2))
        .collect();
    assert!(crossings.len() >= 8, "not enough shedding cycles");
    let st = (crossings.len() - 1) as f64 / (crossings.last().unwrap() - crossings[0]);
    let cd_mean = history.iter().map(|v| v.1).sum::<f64>() / history.len() as f64;
    println!(
        "cylinder D/dx={cells} conserve={conserve} Cd_max={cd_max:.9} Cd_mean={cd_mean:.9} St={st:.9} max_Ma={max_mach:.6} max_density_error={max_density_error:.6} cycles={}",
        crossings.len() - 1
    );
    // Completion is not acceptance: report this candidate without widening published bounds.
    println!(
        "cylinder accuracy gates: drag={} frequency={}; phase study still outstanding",
        (3.22 * 0.98..=3.24 * 1.02).contains(&cd_max),
        (0.295..=0.305).contains(&st)
    );
}
