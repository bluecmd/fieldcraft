//! Local numerical experiment for #798; deliberately unavailable to production callers.
//! Linear BFL: Bouzidi, Firdaouss & Lallemand (2001), DOI 10.1063/1.1399290;
//! equations checked against Marson et al., arXiv:2009.04604v3, eqs. (13a,b).
//! Momentum exchange: Mei et al. (2002), DOI 10.1103/PhysRevE.65.041203.
//! Optional rest-population compensation: Sanjeevi, Zarghami & Padding (2018),
//! DOI 10.1103/PhysRevE.97.043305, section IV B case 1. Their pressure-artifact
//! warning is why both variants are measured. Neither is accepted here yet.
//! Fixed TRT magic does not remove linear BFL's viscosity-dependent wall error:
//! d'Humieres & Ginzburg (2009), DOI 10.1016/j.camwa.2009.02.008, section 3.1.3.
//! The planar and sphere controls keep this separate from geometry and mass correction.
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
    let _ = advance_with_open_impulse(flow, cuts, conserve, false);
}

fn advance_with_open_impulse(flow: &mut Flow, cuts: &[Cut], conserve: bool, measure: bool) -> Option<[f64; 3]> {
    // The prototype supports only constant-viscosity TRT, static obstacles, no heat/SGS.
    assert_eq!(flow.collision, Collision::Trt);
    assert!(flow.thermal.is_none() && flow.surface.is_none() && flow.walls.is_none());
    flow.targets().unwrap();
    let u2 = flow.collide();
    assert!(u2 <= MAX_LATTICE_SPEED * MAX_LATTICE_SPEED, "step {} unstable: {u2}", flow.steps_done);
    // Open links initially reflect their outgoing populations as placeholders.
    // Include that flux and the later Zou-He change in the actual open-boundary impulse.
    let placeholder = measure.then(|| {
        let mut impulse = [0.0; 3];
        for link in &flow.links {
            if matches!(link.rule, Rule::Open) {
                let q = link.q as usize;
                let out = flow.f[opposite(q) * flow.stride + link.cell as usize];
                for a in 0..3 {
                    impulse[a] += 2.0 * c(q)[a] * out;
                }
            }
        }
        impulse
    });
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
    let open_momentum = |flow: &Flow| {
        let mut sum = [0.0; 3];
        for open in &flow.open {
            for q in 1..Q {
                let population = flow.g[q * flow.stride + open.cell as usize];
                for a in 0..3 {
                    sum[a] += c(q)[a] * population;
                }
            }
        }
        sum
    };
    let open_before = measure.then(|| open_momentum(flow));
    flow.open_boundaries();
    let impulse = placeholder.zip(open_before).map(|(p, before)| {
        let after = open_momentum(flow);
        [0, 1, 2].map(|a| p[a] + after[a] - before[a])
    });
    std::mem::swap(&mut flow.f, &mut flow.g);
    flow.steps_done += 1;
    impulse
}

fn sphere_case(radius: usize, phase: [f64; 3], steps: u64) -> (Flow, Vec<Cut>, f64) {
    sphere_case_with(radius, phase, steps, 5.0 / 6.0, 4)
}

fn sphere_case_with(radius: usize, phase: [f64; 3], steps: u64, nu: f64, levels: u32) -> (Flow, Vec<Cut>, f64) {
    let side = (16 * radius / 3) as f64;
    let mut d = domain([0.0; 3], [side; 3], 1.0);
    for f in BoxFace::ALL {
        *d.faces.get_mut(f) = FaceCondition::Periodic {};
    }
    // Keep the continuum Reynolds number fixed in the viscosity diagnostic.
    d.acceleration = [1e-6 * (3.0 / radius as f64).powi(3) * (nu / (5.0 / 6.0)).powi(2), 0.0, 0.0];
    d.obstacles = vec!["Ball".into()];
    let ball = obstacle("Ball", sphere(phase.map(|p| side / 2.0 + p), radius as f64, levels), &["skin"]);
    let m = model(1.0, nu, d, step(steps as f64, 1.0, Some(3.0 / 16.0)));
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
    let reference = run(1);
    for threads in [4, 8] {
        assert_eq!(reference, run(threads));
    }
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
fn open_boundary_actual_impulse_closes_each_step() {
    let mut dom = domain([0.0; 3], [8.0, 8.0, 1.0], 1.0);
    dom.faces.x_min = FaceCondition::Velocity { velocity: [0.02, 0.0, 0.0], profile: Profile::Parabolic };
    dom.faces.x_max = FaceCondition::Pressure { pressure: 0.0, outflow: Outflow::Uniform };
    dom.faces.z_min = FaceCondition::Periodic {};
    dom.faces.z_max = FaceCondition::Periodic {};
    let m = model(1.0, 0.1, dom, step(16.0, 1.0, Some(3.0 / 16.0)));
    let setup = Setup::new(&m, None, &[], &crate::Limits::default()).unwrap();
    let mut flow = Flow::new(&setup, &[]).unwrap();
    for _ in 0..16 {
        let before = momentum(&flow);
        let open = advance_with_open_impulse(&mut flow, &[], true, true).unwrap();
        let after = momentum(&flow);
        for a in 0..3 {
            let residual = after[a] - before[a] + flow.forces.iter().map(|f| f[a]).sum::<f64>() - open[a];
            assert!(residual.abs() < 1e-12, "open momentum residual {residual:e}");
        }
    }
}

#[test]
#[ignore = "heavy: local #798 design experiment, not a production validation"]
fn sphere_study() {
    let radius: usize = std::env::var("FIELDCRAFT_PROTOTYPE_RADIUS").unwrap_or("3".into()).parse().unwrap();
    assert!([3, 6, 9, 12].contains(&radius));
    let conserve = std::env::var("FIELDCRAFT_PROTOTYPE_CONSERVE").unwrap_or("1".into()) == "1";
    let tau: f64 = std::env::var("FIELDCRAFT_PROTOTYPE_TAU").unwrap_or("3".into()).parse().unwrap();
    assert!((0.8..=3.0).contains(&tau));
    let nu = (tau - 0.5) / 3.0;
    let levels: u32 = std::env::var("FIELDCRAFT_PROTOTYPE_LEVELS").unwrap_or("4".into()).parse().unwrap();
    assert!((3..=5).contains(&levels));
    let selected: Option<usize> = std::env::var("FIELDCRAFT_PROTOTYPE_PHASE").ok().map(|v| v.parse().unwrap());
    assert!(selected.is_none_or(|p| p < 4));
    let wall = std::env::var("FIELDCRAFT_PROTOTYPE_WALL").unwrap_or("interpolated".into());
    assert!(["interpolated", "halfway"].contains(&wall.as_str()));
    let steps = (3000.max(1500 * (radius as u64 / 3).pow(2)) as f64 * (5.0 / 6.0) / nu).ceil() as u64;
    for (phase_id, phase) in [[0.0; 3], [0.25; 3], [0.5; 3], [0.125, 0.375, 0.25]].into_iter().enumerate() {
        if selected.is_some_and(|p| p != phase_id) {
            continue;
        }
        let (mut flow, mut cuts, mass0) = sphere_case_with(radius, phase, steps, nu, levels);
        println!(
            "sphere setup R={radius} phase_id={phase_id} tau={tau} levels={levels} wall={wall} steps={steps} links={} threads={}",
            cuts.len(),
            parallel::default_threads()
        );
        let centre = phase.map(|p| (16 * radius / 3) as f64 / 2.0 + p);
        let radius_deficit = cuts
            .iter()
            .map(|cut| {
                let start = flow.grid.centre(cut.cell);
                let distance2 = (0..3).map(|a| (start[a] - c(cut.direction)[a] * cut.fraction - centre[a]).powi(2)).sum::<f64>();
                1.0 - distance2.sqrt() / radius as f64
            })
            .fold(0.0_f64, f64::max);
        println!("sphere geometry max relative radial deficit={radius_deficit:.6e}");
        if wall == "halfway" {
            cuts.clear();
        }
        let mut previous_k = 0.0;
        for i in 1..=steps {
            let sample = [steps / 4, steps / 2, 3 * steps / 4, steps].contains(&i);
            let before = sample.then(|| (momentum(&flow), mass(&flow)));
            advance(&mut flow, &cuts, conserve);
            if let Some((before_p, before_mass)) = before {
                let after_p = momentum(&flow);
                let residual = (0..3)
                    .map(|a| (after_p[a] - before_p[a] - before_mass * flow.acceleration[a] + flow.forces.iter().map(|f| f[a]).sum::<f64>()).abs())
                    .fold(0.0_f64, f64::max);
                let scale = before_p.into_iter().chain(after_p).map(f64::abs).fold(1.0_f64, f64::max);
                assert!(residual <= 1e-12 * scale, "momentum residual {residual:e} scale {scale:e}");
                println!("sphere sampled momentum residual={residual:.3e} scale={scale:.3e}");
                let mass = mass(&flow);
                let rho = mass / flow.fluid_cells as f64;
                let (mut sum_u, mut max_u2, mut max_density_error) = (0.0, 0.0_f64, 0.0_f64);
                for (density, velocity) in (0..flow.n).filter_map(|c| flow.moments(c)) {
                    assert!(density.is_finite() && density > 0.0 && velocity.iter().all(|v| v.is_finite()));
                    sum_u += velocity[0];
                    max_u2 = max_u2.max(velocity.iter().map(|v| v * v).sum::<f64>());
                    max_density_error = max_density_error.max((density - rho).abs());
                }
                let u = sum_u / flow.n as f64;
                let re = u * 2.0 * radius as f64 / nu;
                assert!(re > 0.0 && re < 0.001);
                println!("sphere Re={re:.6e} max_Ma={:.6e} max_density_deviation={max_density_error:.6e}", (3.0 * max_u2).sqrt());
                let force: f64 = flow.forces.iter().map(|f| f[0]).sum();
                let k = force * flow.n as f64 / flow.fluid_cells as f64 / (6.0 * std::f64::consts::PI * nu * radius as f64 * u * rho);
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

/// Isolate wall-location/relaxation error with an analytic quadratic profile.
/// Linear BFL is not viscosity-independent under fixed TRT magic:
/// d'Humieres & Ginzburg (2009), DOI 10.1016/j.camwa.2009.02.008, section 3.1.3.
#[test]
#[ignore = "heavy: local #798 design experiment, not a production validation"]
fn planar_study() {
    for q in [0.25, 0.5, 0.75] {
        for tau in [1.0, 3.0] {
            let nu: f64 = (tau - 0.5) / 3.0;
            let acceleration = 1e-6 * (nu / (5.0 / 6.0)).powi(2);
            let (bottom, top) = (0.5 - q, 15.5 + q);
            let height: f64 = top - bottom;
            let mut dom = domain([0.0, -2.0, 0.0], [8.0, 18.0, 1.0], 1.0);
            dom.faces.x_min = FaceCondition::Periodic {};
            dom.faces.x_max = FaceCondition::Periodic {};
            dom.faces.z_min = FaceCondition::Periodic {};
            dom.faces.z_max = FaceCondition::Periodic {};
            dom.acceleration = [acceleration, 0.0, 0.0];
            dom.obstacles = vec!["Bottom".into(), "Top".into()];
            let walls = [
                obstacle("Bottom", common::cuboid([-1.0, -3.0, -1.0], [9.0, bottom, 2.0]), &common::CUBOID_FACES),
                obstacle("Top", common::cuboid([-1.0, top, -1.0], [9.0, 19.0, 2.0]), &common::CUBOID_FACES),
            ];
            let steps = (1600.0 * (5.0 / 6.0) / nu).ceil() as u64;
            let m = model(1.0, nu, dom, step(steps as f64, 1.0, Some(3.0 / 16.0)));
            let setup = Setup::new(&m, None, &walls, &crate::Limits::default()).unwrap();
            let mut flow = Flow::new(&setup, &walls).unwrap();
            let links: Vec<Cut> = walls.iter().flat_map(|w| cuts(&flow, &w.surface, &w.body)).collect();
            assert!(links.iter().all(|c| (c.fraction - q).abs() < 1e-14));
            let mass0 = mass(&flow);
            for _ in 0..steps {
                advance(&mut flow, &links, true);
            }
            let peak = acceleration * height.powi(2) / (8.0 * nu);
            let errors: Vec<f64> = (0..flow.n)
                .filter_map(|cell| {
                    flow.moments(cell).map(|(_, u)| {
                        let y = flow.grid.centre(cell)[1] - bottom;
                        (u[0] - acceleration * y * (height - y) / (2.0 * nu)) / peak
                    })
                })
                .collect();
            let low = errors.iter().copied().fold(f64::INFINITY, f64::min);
            let high = errors.iter().copied().fold(f64::NEG_INFINITY, f64::max);
            let balance = flow.forces.iter().map(|f| f[0]).sum::<f64>() / (mass(&flow) * acceleration) - 1.0;
            assert!((mass(&flow) / mass0 - 1.0).abs() < 1e-10);
            assert!(balance.abs() < 1e-8);
            assert!(high - low < 1e-7, "the error must be a uniform slip, not wrong curvature");
            if q == 0.5 {
                assert!(low.abs().max(high.abs()) < 1e-7);
            }
            println!("planar q={q} tau={tau} normalised_slip_min={low:.9} max={high:.9} force_balance={balance:.3e}");
        }
    }
}

/// Published benchmark, Schafer & Turek (1996), DOI 10.1007/978-3-322-89849-4_39.
/// Baseline only: phase comparisons must also move the physical channel walls relative
/// to the grid, which this test intentionally does not pretend to do.
#[test]
#[ignore = "heavy: local #798 design experiment, not a production validation"]
fn cylinder_study() {
    let cells: f64 = std::env::var("FIELDCRAFT_PROTOTYPE_CELLS").unwrap_or("20".into()).parse().unwrap();
    assert!([20.0, 30.0, 40.0].contains(&cells));
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
    let limits = crate::Limits { max_cell_updates: 24_000_000_000, ..Default::default() };
    let setup = Setup::new(&m, None, std::slice::from_ref(&cyl), &limits).unwrap();
    let mut flow = Flow::new(&setup, std::slice::from_ref(&cyl)).unwrap();
    let cuts = cuts(&flow, &cyl.surface, "Cylinder");
    let group = flow.groups.iter().position(|g| g.body.as_deref() == Some("Cylinder") && g.name == "side").unwrap();
    let conserve = std::env::var("FIELDCRAFT_PROTOTYPE_CONSERVE").unwrap_or("1".into()) == "1";
    println!("cylinder setup D/dx={cells} dt={dt} steps={} cells={} links={}", setup.steps, flow.n, cuts.len());
    // Force unit rho dx^4 / dt^2 divided by rho U^2 D span / 2, U=D=1.
    let coefficient = 2.0 * dx.powi(3) / dt.powi(2);
    let mut history = Vec::new();
    let mut cd_max = f64::NEG_INFINITY;
    let mut max_mach = 0.0_f64;
    let mut max_density_error = 0.0_f64;
    for i in 1..=setup.steps {
        let sample = i % (setup.steps / 4) == 0;
        let before = sample.then(|| momentum(&flow));
        let impulse = advance_with_open_impulse(&mut flow, &cuts, conserve, sample);
        let time = i as f64 * dt;
        if time >= 80.0 {
            cd_max = cd_max.max(coefficient * flow.forces[group][0]);
        }
        if time >= 80.0 && i % 60 == 0 {
            history.push((time, coefficient * flow.forces[group][0], coefficient * flow.forces[group][1]));
            for cell in 0..flow.n {
                if let Some((rho, u)) = flow.moments(cell) {
                    assert!(rho.is_finite() && rho > 0.0 && u.iter().all(|v| v.is_finite()));
                    max_mach = max_mach.max((3.0 * u.iter().map(|v| v * v).sum::<f64>()).sqrt());
                    max_density_error = max_density_error.max((rho - 1.0).abs());
                }
            }
        }
        if let Some((before, open)) = before.zip(impulse) {
            let after = momentum(&flow);
            let residual =
                (0..3).map(|a| (after[a] - before[a] + flow.forces.iter().map(|f| f[a]).sum::<f64>() - open[a]).abs()).fold(0.0_f64, f64::max);
            let scale = before.into_iter().chain(after).map(f64::abs).fold(1.0_f64, f64::max);
            assert!(residual <= 1e-12 * scale, "open momentum residual {residual:e} scale {scale:e}");
            println!("cylinder sampled momentum residual={residual:.3e} scale={scale:.3e}");
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
    for (start, end) in [(80.0, 100.0), (100.0, 120.0)] {
        let samples: Vec<_> = history.iter().filter(|v| (start..end).contains(&v.0)).collect();
        let mean = samples.iter().map(|v| v.1).sum::<f64>() / samples.len() as f64;
        let peak = samples.iter().map(|v| v.1).fold(f64::NEG_INFINITY, f64::max);
        let cycles: Vec<_> = crossings.iter().copied().filter(|t| (start..end).contains(t)).collect();
        assert!(cycles.len() >= 4);
        let frequency = (cycles.len() - 1) as f64 / (cycles.last().unwrap() - cycles[0]);
        println!("cylinder window={start}..{end} Cd_mean={mean:.9} Cd_max_sampled={peak:.9} St={frequency:.9}");
    }
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
