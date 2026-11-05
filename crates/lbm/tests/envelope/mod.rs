//! Test-only bounded measurements. Theory: linear isothermal Navier–Stokes;
//! d'Humières et al. (2002), doi:10.1098/rsta.2001.0955, §3 and appendix A
//! (MRT bulk viscosity); Taylor & Green (1937), doi:10.1098/rspa.1937.0036.
//! Original initial conditions, no external solver code or decks.

use std::f64::consts::TAU;

use fieldcraft_lbm::{Flow, Limits, Setup};
use fieldcraft_lbm::{lattice::Q, turbulence};
use fieldcraft_model::{BoxFace, Collision, FaceCondition, HrrSigma, Procedure, Turbulence};
use serde::Serialize;

use super::common::{collide, domain, model};

#[derive(Clone, Copy, Debug, Serialize)]
pub struct Config {
    pub collision: Collision,
    pub smagorinsky: bool,
    pub sigma: f64,
    pub dynamic: bool,
}

impl Config {
    pub fn new(collision: Collision, smagorinsky: bool) -> Self {
        Self { collision, smagorinsky, sigma: 0.99, dynamic: false }
    }
}

pub fn configurations() -> Vec<Config> {
    let mut cases = Vec::new();
    for collision in [Collision::Trt, Collision::Mrt, Collision::Regularized, Collision::Hrr] {
        for sgs in [false, true] {
            cases.push(Config::new(collision, sgs));
        }
    }
    cases.push(Config { sigma: 0.9, ..Config::new(Collision::Hrr, true) });
    cases.push(Config { dynamic: true, ..Config::new(Collision::Hrr, true) });
    cases
}

/// Units dx=dt=rho0=1. The low-tau no-SGS controls deliberately bypass only the
/// *setup* floor by setting up with C_s=.1 then clearing it before Flow::new.
/// They are diagnostics, not configurations accepted by the public command.
fn flow(cells: [usize; 3], tau: f64, config: Config, steps: usize) -> Result<Flow, String> {
    let mut d = domain([0.0; 3], cells.map(|n| n as f64), 1.0);
    for face in BoxFace::ALL {
        *d.faces.get_mut(face) = FaceCondition::Periodic {};
    }
    let mut p = collide(steps as f64, 1.0, None, config.collision);
    if let Procedure::Fluid { turbulence, sigma, .. } = &mut p {
        *turbulence = Some(Turbulence::Smagorinsky { constant: 0.1, prandtl: 0.9, van_driest: None, wall_model: None });
        if config.collision == Collision::Hrr {
            *sigma = Some(if config.dynamic { HrrSigma::Dynamic } else { HrrSigma::Value(config.sigma) });
        }
    }
    let limits = Limits { max_cells: 64 * 64 * 64, max_steps: 32768, max_cell_updates: 400_000_000, ..Limits::default() };
    let mut setup = Setup::new(&model(1.0, (tau - 0.5) / 3.0, d, p), None, &[], &limits).map_err(|e| e.to_string())?;
    if !config.smagorinsky {
        setup.smagorinsky = 0.0;
    }
    Flow::new(&setup, &[]).map_err(|e| e.to_string())
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize)]
pub enum Kind {
    Shear,
    Acoustic,
    TaylorGreen,
}

#[derive(Clone, Copy, Debug, Serialize)]
pub struct WaveCase {
    pub kind: Kind,
    pub n: usize,
    pub tau: f64,
    pub config: Config,
    pub direction: [i32; 3],
    pub drift: f64,
    pub amplitude: f64,
    pub max_steps: usize,
    pub fit_start: usize,
}

impl Default for WaveCase {
    fn default() -> Self {
        Self {
            kind: Kind::Shear,
            n: 64,
            tau: 0.8,
            config: Config::new(Collision::Trt, false),
            direction: [1, 0, 0],
            drift: 0.0,
            amplitude: 1e-6,
            max_steps: 8192,
            fit_start: 128,
        }
    }
}

#[derive(Debug, Serialize)]
pub struct Fit {
    pub rate: f64,
    pub log_rms_residual: f64,
    pub half_window_relative_difference: f64,
}

fn slope(points: &[(f64, f64)]) -> Option<(f64, f64)> {
    if points.len() < 3 || points.iter().any(|(t, a)| !t.is_finite() || !a.is_finite() || *a <= 1e-13) {
        return None;
    }
    let count = points.len() as f64;
    let tmean = points.iter().map(|p| p.0).sum::<f64>() / count;
    let ymean = points.iter().map(|p| p.1.ln()).sum::<f64>() / count;
    let den = points.iter().map(|p| (p.0 - tmean).powi(2)).sum::<f64>();
    if den <= 0.0 {
        return None;
    }
    let slope = points.iter().map(|p| (p.0 - tmean) * (p.1.ln() - ymean)).sum::<f64>() / den;
    let residual = (points.iter().map(|p| (p.1.ln() - ymean - slope * (p.0 - tmean)).powi(2)).sum::<f64>() / count).sqrt();
    Some((-slope, residual))
}

pub fn fit_decay(points: &[(f64, f64)]) -> Option<Fit> {
    let (rate, residual) = slope(points)?;
    let (first, _) = slope(points.get(..points.len() / 2)?)?;
    let (last, _) = slope(points.get(points.len() / 2..)?)?;
    Some(Fit { rate, log_rms_residual: residual, half_window_relative_difference: (first - last).abs() / rate.abs().max(1e-30) })
}

#[derive(Debug, Serialize, Default)]
struct Health {
    mass_drift: f64,
    density_departure: f64,
    mach: f64,
    eddy_ratio_max: f64,
    sigma_min: Option<f64>,
}

impl Health {
    fn sample(&mut self, f: &Flow, mass: f64) -> Result<(), String> {
        self.mass_drift = self.mass_drift.max((f.mass() / mass - 1.0).abs());
        self.eddy_ratio_max = self.eddy_ratio_max.max(f.eddy_viscosity_ratio());
        if let Some((min, _)) = f.dynamic_sigma() {
            self.sigma_min = Some(self.sigma_min.unwrap_or(1.0).min(min));
        }
        for c in 0..f.grid().len() {
            let (rho, u) = f.moments(c).ok_or("missing fluid moments")?;
            if !rho.is_finite() || u.iter().any(|v| !v.is_finite()) {
                return Err("nonfinite field".into());
            }
            self.density_departure = self.density_departure.max((rho - 1.0).abs());
            self.mach = self.mach.max((3.0 * u.iter().map(|v| v * v).sum::<f64>()).sqrt());
        }
        Ok(())
    }
}

#[derive(Debug, Serialize)]
pub struct WaveRow {
    pub case: WaveCase,
    pub diagnostic_sgs_off_below_setup_floor: bool,
    pub completed: bool,
    pub error: Option<String>,
    pub steps: usize,
    pub fit_start: usize,
    pub sample_stride: usize,
    pub reference_rate: f64,
    pub fit: Option<Fit>,
    pub relative_rate_error: Option<f64>,
    pub effective_viscosity: Option<f64>,
    pub viscosity_pass: bool,
    pub decay_pass: bool,
    pub max_amplitude_ratio: f64,
    health: Health,
}

fn dot(a: [f64; 3], b: [f64; 3]) -> f64 {
    a.into_iter().zip(b).map(|(x, y)| x * y).sum()
}

pub fn wave(case: WaveCase) -> WaveRow {
    let k = TAU / case.n as f64;
    let direction = case.direction.map(f64::from);
    let norm = dot(direction, direction).sqrt();
    let normal = direction.map(|v| v / norm);
    let transverse = [-normal[1], normal[0], 0.0];
    let k2 = if case.kind == Kind::TaylorGreen { 2.0 * k * k } else { k * k * norm * norm };
    let nu = (case.tau - 0.5) / 3.0;
    // Linear isothermal sound: Gamma = (4nu/3 + zeta)/2. MRT's trace
    // relaxes at s_e=1.19; all other collisions have zeta=2nu/3 at k->0.
    let attenuation =
        if case.kind == Kind::Acoustic && case.config.collision == Collision::Mrt { 2.0 * nu / 3.0 + (1.0 / 1.19 - 0.5) / 9.0 } else { nu };
    let reference_rate = attenuation * k2;
    let steps = ((1.5 / reference_rate).ceil() as usize).clamp(case.fit_start + 384, case.max_steps);
    let burn = case.fit_start;
    let stride = 8;
    let mut row = WaveRow {
        case,
        diagnostic_sgs_off_below_setup_floor: !case.config.smagorinsky && case.tau < 0.505,
        completed: false,
        error: None,
        steps: 0,
        fit_start: burn,
        sample_stride: stride,
        reference_rate,
        fit: None,
        relative_rate_error: None,
        effective_viscosity: None,
        viscosity_pass: false,
        decay_pass: false,
        max_amplitude_ratio: 1.0,
        health: Health::default(),
    };
    let mut run = || -> Result<Vec<(f64, f64)>, String> {
        let cells = if case.kind == Kind::TaylorGreen { [case.n, case.n, 1] } else { case.direction.map(|d| if d == 0 { 1 } else { case.n }) };
        let mut f = flow(cells, case.tau, case.config, steps)?;
        let grid = *f.grid();
        let phase = |c| dot(grid.centre(c), direction) * k;
        let tg = |c| {
            let [x, y, _] = grid.centre(c).map(|x| k * x);
            [x.sin() * y.cos(), -x.cos() * y.sin(), 0.0]
        };
        f.set_equilibrium(|c| {
            let wave = case.amplitude * phase(c).cos();
            let base = normal.map(|v| case.drift * v);
            match case.kind {
                Kind::Shear => (1.0, std::array::from_fn(|a| base[a] + wave * transverse[a])),
                Kind::Acoustic => (1.0 + 3.0_f64.sqrt() * wave, normal.map(|v| v * (case.drift + wave))),
                Kind::TaylorGreen => {
                    let [x, y, _] = grid.centre(c).map(|x| k * x);
                    let pressure = case.amplitude.powi(2) * ((2.0 * x).cos() + (2.0 * y).cos()) / 4.0;
                    (1.0 + 3.0 * pressure, tg(c).map(|v| v * case.amplitude))
                }
            }
        });
        let amplitude = |f: &Flow| -> f64 {
            let mut re = 0.0;
            let mut im = 0.0;
            let mut den = 0.0;
            for c in 0..grid.len() {
                let (rho, u) = f.moments(c).unwrap();
                if case.kind == Kind::TaylorGreen {
                    let v = tg(c);
                    re += dot(u, v);
                    den += dot(v, v);
                } else {
                    let value = if case.kind == Kind::Acoustic {
                        // Right-going acoustic characteristic; complex spatial projection
                        // removes travel-phase oscillations and largely rejects the left mode.
                        0.5 * (dot(u, normal) - case.drift + (rho - 1.0) / 3.0_f64.sqrt())
                    } else {
                        dot(u, transverse)
                    };
                    re += value * phase(c).cos();
                    im += value * phase(c).sin();
                    den += 0.5;
                }
            }
            re.hypot(im) / den
        };
        let mass = f.mass();
        row.health.sample(&f, mass)?;
        let mut samples = Vec::new();
        for step in 1..=steps {
            f.step().map_err(|e| e.to_string())?;
            row.steps = step;
            if step % stride == 0 {
                row.health.sample(&f, mass)?;
                let a = amplitude(&f);
                row.max_amplitude_ratio = row.max_amplitude_ratio.max(a / case.amplitude);
                if step >= burn {
                    samples.push((step as f64, a));
                }
                if a > 10.0 * case.amplitude {
                    return Err("mode amplified more than tenfold".into());
                }
            }
        }
        Ok(samples)
    };
    match run() {
        Ok(samples) => {
            row.completed = true;
            row.fit = fit_decay(&samples);
            if let Some(fit) = &row.fit {
                let err = fit.rate / reference_rate - 1.0;
                row.relative_rate_error = Some(err);
                if case.kind != Kind::Acoustic {
                    row.effective_viscosity = Some(fit.rate / k2);
                }
                let reliable =
                    fit.rate > 0.0 && fit.half_window_relative_difference < 0.01 && fit.log_rms_residual < 0.001 && row.health.mass_drift < 1e-10;
                row.decay_pass = reliable && err.abs() <= 0.02;
                row.viscosity_pass = case.kind != Kind::Acoustic && reliable && err.abs() <= 0.01;
            }
        }
        Err(error) => row.error = Some(error),
    }
    row
}

#[derive(Debug, Serialize)]
pub struct TgvRow {
    config: Config,
    n: usize,
    reynolds: f64,
    tau: f64,
    completed: bool,
    error: Option<String>,
    sample_stride: usize,
    /// t, E_x, E_y, E_z, <u_x u_y>, sigma_min, sigma_mean, max(nu_t/nu),
    /// molecular 2nu<S:S>, modelled 2<nu_t S:S>, explicit SGS contribution.
    /// The last is zero for dynamic HRR: its SGS estimate sets the filter instead.
    history: Vec<[f64; 11]>,
    peak_dissipation: f64,
    peak_time: f64,
    peak_relative_error: Option<f64>,
    peak_time_relative_error: Option<f64>,
    /// Peak checks alone do not establish spatial convergence or validate LES.
    peak_checks_pass: bool,
    health: Health,
}

pub fn taylor_green_3d(n: usize, config: Config) -> TgvRow {
    let speed = 0.05;
    let k = TAU / n as f64;
    let dt = speed * k;
    let tau = 0.5 + 3.0 * speed / (1600.0 * k);
    let steps = (20.0 / dt).ceil() as usize;
    let stride = 8;
    let mut row = TgvRow {
        config,
        n,
        reynolds: 1600.0,
        tau,
        completed: false,
        error: None,
        sample_stride: stride,
        history: Vec::new(),
        peak_dissipation: 0.0,
        peak_time: 0.0,
        peak_relative_error: None,
        peak_time_relative_error: None,
        peak_checks_pass: false,
        health: Health::default(),
    };
    let mut run = || -> Result<(), String> {
        let mut f = flow([n; 3], tau, config, steps)?;
        let grid = *f.grid();
        f.set_equilibrium(|c| {
            let [x, y, z] = grid.centre(c).map(|x| x * k);
            let p = speed * speed * ((2.0 * x).cos() + (2.0 * y).cos()) * ((2.0 * z).cos() + 2.0) / 16.0;
            (1.0 + 3.0 * p, [speed * x.sin() * y.cos() * z.cos(), -speed * x.cos() * y.sin() * z.cos(), 0.0])
        });
        let mass = f.mass();
        for step in 0..=steps {
            if step % stride == 0 || step == steps {
                row.health.sample(&f, mass)?;
                let mut e = [0.0; 3];
                let mut stress = 0.0;
                let mut molecular = 0.0;
                let mut modelled = 0.0;
                for c in 0..grid.len() {
                    let (rho, u) = f.moments(c).ok_or("missing moments")?;
                    for a in 0..3 {
                        e[a] += 0.5 * u[a] * u[a];
                    }
                    stress += u[0] * u[1];
                    let mut gradient = [[0.0; 3]; 3];
                    for axis in 0..3 {
                        let mut plus = grid.coords(c);
                        let mut minus = plus;
                        plus[axis] = (plus[axis] + 1) % n;
                        minus[axis] = (minus[axis] + n - 1) % n;
                        let up = f.moments(grid.index(plus)).ok_or("missing periodic neighbour")?.1;
                        let um = f.moments(grid.index(minus)).ok_or("missing periodic neighbour")?.1;
                        for component in 0..3 {
                            gradient[component][axis] = 0.5 * (up[component] - um[component]);
                        }
                    }
                    let mut ss = 0.0;
                    for (a, row) in gradient.iter().enumerate() {
                        for (b, value) in row.iter().enumerate() {
                            ss += (0.5 * (*value + gradient[b][a])).powi(2);
                        }
                    }
                    let populations: [f64; Q] = std::array::from_fn(|q| f.population(q, c));
                    let flux = turbulence::flux(&populations, rho, u, [0.0; 3], false);
                    let nu_t = if config.smagorinsky {
                        turbulence::eddy_viscosity(&flux, rho, 1.0 / tau, 0.1, if config.dynamic { turbulence::flux_round_off(rho) } else { 0.0 })
                    } else {
                        0.0
                    };
                    molecular += 2.0 * (tau - 0.5) / 3.0 * ss;
                    modelled += 2.0 * nu_t * ss;
                }
                let scale = grid.len() as f64 * speed * speed;
                let [ex, ey, ez] = e.map(|v| v / scale);
                let (smin, smean) = f.dynamic_sigma().unwrap_or((config.sigma, config.sigma));
                let mol = molecular / (scale * dt);
                let sgs = modelled / (scale * dt);
                row.history.push([
                    step as f64 * dt,
                    ex,
                    ey,
                    ez,
                    stress / scale,
                    smin,
                    smean,
                    f.eddy_viscosity_ratio(),
                    mol,
                    sgs,
                    if config.dynamic { 0.0 } else { sgs },
                ]);
            }
            if step < steps {
                f.step().map_err(|e| e.to_string())?;
            }
        }
        Ok(())
    };
    match run() {
        Ok(()) => row.completed = true,
        Err(e) => row.error = Some(e),
    }
    for pair in row.history.windows(2) {
        let [a, b] = pair else { continue };
        let dissipation = (a[1] + a[2] + a[3] - b[1] - b[2] - b[3]) / (b[0] - a[0]);
        if dissipation > row.peak_dissipation {
            row.peak_dissipation = dissipation;
            row.peak_time = 0.5 * (a[0] + b[0]);
        }
    }
    if row.completed {
        // Finite differences of the incompressible energy column in the published
        // 512-point data, Kurz et al. (2024), doi:10.18419/DARUS-4139, file 291392.
        // See the report for extraction, time resolution and the 256-point control.
        let peak_error = row.peak_dissipation / 0.0128393 - 1.0;
        let time_error = row.peak_time / 8.98698 - 1.0;
        row.peak_relative_error = Some(peak_error);
        row.peak_time_relative_error = Some(time_error);
        row.peak_checks_pass = peak_error.abs() <= 0.05 && time_error.abs() <= 0.05;
    }
    row
}

pub fn emit(row: &impl Serialize) {
    println!("ENVELOPE {}", serde_json::to_string(row).unwrap());
}

pub fn bluff_body(collision: Collision, body: bool, periodic: bool, re: f64) -> serde_json::Value {
    use super::common::{cylinder, obstacle};
    use fieldcraft_model::{Outflow, Profile};
    let dx = 0.125;
    let dt = 0.05 * dx;
    let mut d = domain([0.0; 3], [24.0, 12.0, dx], dx);
    d.initial_velocity = [1.0, 0.0, 0.0];
    d.faces.x_min =
        if periodic { FaceCondition::Periodic {} } else { FaceCondition::Velocity { velocity: [1.0, 0.0, 0.0], profile: Profile::Uniform } };
    d.faces.x_max = if periodic { FaceCondition::Periodic {} } else { FaceCondition::Pressure { pressure: 0.0, outflow: Outflow::Convective } };
    d.faces.y_min = FaceCondition::Slip {};
    d.faces.y_max = FaceCondition::Slip {};
    d.faces.z_min = FaceCondition::Periodic {};
    d.faces.z_max = FaceCondition::Periodic {};
    let mut obstacles = Vec::new();
    if body {
        d.obstacles.push("Cylinder".into());
        obstacles.push(obstacle("Cylinder", cylinder([6.0, 6.0], 0.5, -dx, 2.0 * dx, 1024), &["side", "bottom", "top"]));
    }
    let mut p = collide(1200.0 * dt, dt, None, collision);
    if let Procedure::Fluid { turbulence, .. } = &mut p {
        *turbulence = Some(Turbulence::Smagorinsky { constant: 0.1, prandtl: 0.9, van_driest: None, wall_model: None });
    }
    let limits = Limits { max_cells: 20000, max_steps: 1200, max_cell_updates: 24_000_000, ..Limits::default() };
    let setup = Setup::new(&model(1.0, 1.0 / re, d, p), None, &obstacles, &limits).unwrap();
    let mut f = Flow::new(&setup, &obstacles).unwrap();
    let grid = *f.grid();
    // The documented seed degenerates to zero (up to sin(pi) rounding) in a
    // one-cell span. Preserve that fact instead of adding a new planar seed.
    f.set_equilibrium(|c| {
        let [x, y, z] = grid.centre(c);
        let envelope = (-((x - 7.0).powi(2) + (y - 6.0).powi(2)) / 4.0).exp();
        let phase = TAU * z / dx;
        (1.0, [0.05, 0.0005 * envelope * (1.0 + phase.cos()), 0.0005 * envelope * phase.sin()])
    });
    let mut error = None;
    let mut attempted = 0;
    for step in 1..=1200 {
        attempted = step;
        if let Err(e) = f.step() {
            error = Some(e.to_string());
            break;
        }
    }
    let mut max_speed: f64 = 0.0;
    let mut max_at = [0.0; 3];
    let mut min_density = f64::INFINITY;
    let mut max_density = f64::NEG_INFINITY;
    for c in 0..grid.len() {
        if let Some((rho, u)) = f.moments(c) {
            min_density = min_density.min(rho);
            max_density = max_density.max(rho);
            let speed = dot(u, u).sqrt();
            if speed > max_speed {
                max_speed = speed;
                max_at = grid.centre(c);
            }
        }
    }
    serde_json::json!({"collision":collision,"body":body,"periodic_x":periodic,"reynolds":re,
        "tau":setup.tau_plus,"attempted_step":attempted,"completed":error.is_none(),"error":error,
        "max_speed":max_speed,"max_at":max_at,"density_range":[min_density,max_density]})
}
