//! Measurement harness for #790/#788: production CPU collisions, periodic analytic modes.
//!
//! Run the ignored studies individually with `--ignored --nocapture --test-threads=1`.
//! Lines prefixed ENVELOPE are JSON records, including failed runs. Completion is not acceptance.
//! No production defaults or validation limits are changed. See docs/verification/collision-envelope.md.

mod common;
mod envelope;

use envelope::*;
use fieldcraft_model::Collision;

#[test]
fn decay_fit_recovers_a_known_rate_and_refuses_unresolved_signals() {
    let points: Vec<_> = (0..101).map(|t| (t as f64, 0.003 * (-0.007 * t as f64).exp())).collect();
    let fit = fit_decay(&points).unwrap();
    assert!((fit.rate / 0.007 - 1.0).abs() < 1e-12);
    assert!(fit.half_window_relative_difference < 1e-12);
    assert!(fit_decay(&[(0.0, 0.0), (1.0, 1.0)]).is_none());
    assert!(fit_decay(&[(0.0, 1.0), (1.0, f64::NAN)]).is_none());
    let transient: Vec<_> = (0..101).map(|t| (t as f64, (-0.007 * t as f64).exp() + 0.2 * (-0.1 * t as f64).exp())).collect();
    assert!(fit_decay(&transient).unwrap().half_window_relative_difference > 0.01);
}

#[test]
fn resolved_shear_and_sound_recover_continuum_viscosities() {
    for collision in [Collision::Trt, Collision::Mrt, Collision::Regularized, Collision::Hrr] {
        for kind in [Kind::Shear, Kind::Acoustic] {
            let row = wave(WaveCase { kind, n: 64, tau: 0.8, config: Config::new(collision, false), ..WaveCase::default() });
            emit(&row);
            assert!(row.completed, "{row:?}");
            assert!(row.decay_pass, "{row:?}");
            if kind == Kind::Shear {
                assert!(row.viscosity_pass, "{row:?}");
            }
        }
    }
}

/// A reproducible accuracy failure, not an accepted LES configuration. Keep the
/// molecular control and the 1% criterion separate from the measured HRR band.
#[test]
fn low_tau_hrr_filter_exceeds_the_viscosity_budget() {
    let case = WaveCase { tau: 0.5005, fit_start: 20000, max_steps: 32768, ..WaveCase::default() };
    let regularized = wave(WaveCase { config: Config::new(Collision::Regularized, false), ..case });
    let hrr = wave(WaveCase { config: Config::new(Collision::Hrr, false), ..case });
    emit(&regularized);
    emit(&hrr);
    assert!(regularized.completed && regularized.viscosity_pass, "{regularized:?}");
    assert!(hrr.completed && !hrr.viscosity_pass && hrr.decay_pass, "{hrr:?}");
    assert!((0.012..0.014).contains(&hrr.relative_rate_error.unwrap()), "{hrr:?}");
}

#[test]
#[ignore = "heavy: cargo xtask test --heavy (CI heavy job)"]
fn wave_operating_envelope() {
    for config in configurations() {
        for tau in [0.5001, 0.5005, 0.505, 0.55, 0.8] {
            for n in [16, 32, 64] {
                for kind in [Kind::Shear, Kind::Acoustic] {
                    emit(&wave(WaveCase { kind, n, tau, config, ..WaveCase::default() }));
                }
            }
        }
    }
}

/// Low-tau kinetic transients can outlive the short window. Keep the original
/// measurements and independently repeat low-k shear with a much later fit.
#[test]
#[ignore = "heavy: cargo xtask test --heavy (CI heavy job)"]
fn low_tau_long_window() {
    for config in configurations() {
        for tau in [0.5001, 0.5005] {
            emit(&wave(WaveCase { n: 64, tau, config, fit_start: 20000, max_steps: 32768, ..WaveCase::default() }));
        }
    }
}

#[test]
#[ignore = "heavy: cargo xtask test --heavy (CI heavy job)"]
fn held_out_advected_oblique_modes() {
    for config in configurations() {
        for tau in [0.5005, 0.505, 0.8] {
            for direction in [[0, 1, 0], [1, 1, 0]] {
                for kind in [Kind::Shear, Kind::Acoustic] {
                    emit(&wave(WaveCase { kind, n: 24, tau, config, direction, drift: 0.05, max_steps: 2048, ..WaveCase::default() }));
                }
            }
        }
    }
}

/// Held out from the resolved band: short wavelengths with mean advection test
/// growth of lattice modes rather than continuum accuracy.
#[test]
#[ignore = "heavy: cargo xtask test --heavy (CI heavy job)"]
fn short_wavelength_stability() {
    for config in configurations() {
        for tau in [0.5001, 0.5005] {
            for n in [4, 8] {
                for direction in [[1, 0, 0], [1, 1, 0]] {
                    for kind in [Kind::Shear, Kind::Acoustic] {
                        emit(&wave(WaveCase { kind, n, tau, config, direction, drift: 0.05, max_steps: 2048, ..WaveCase::default() }));
                    }
                }
            }
        }
    }
}

#[test]
#[ignore = "heavy: cargo xtask test --heavy (CI heavy job)"]
fn analytic_taylor_green_envelope() {
    for config in configurations() {
        for tau in [0.5005, 0.505, 0.8] {
            for n in [16, 32, 64] {
                emit(&wave(WaveCase { kind: Kind::TaylorGreen, n, tau, config, max_steps: 2048, ..WaveCase::default() }));
            }
        }
    }
}

#[test]
#[ignore = "heavy: cargo xtask test --heavy (CI heavy job)"]
fn nonlinear_taylor_green_re1600() {
    for config in configurations() {
        for n in [16, 24, 32] {
            emit(&taylor_green_3d(n, config));
        }
    }
}

/// Original CPU quasi-2D reproduction from #790, plus one-factor controls.
/// Keeping it small lets us distinguish periodic-wave survival from the actual
/// bluff-body failure without a new large 3D LES sweep.
#[test]
#[ignore = "heavy: cargo xtask test --heavy (CI heavy job)"]
fn mrt_bluff_body_reproduction() {
    for (collision, body, periodic, re) in [
        (Collision::Mrt, true, false, 3900.0),
        (Collision::Mrt, false, false, 3900.0),
        (Collision::Mrt, true, true, 3900.0),
        (Collision::Hrr, true, false, 3900.0),
        (Collision::Mrt, true, false, 240.0),
    ] {
        let row = bluff_body(collision, body, periodic, re);
        emit(&row);
        // Characterize the unfixed issue: open MRT fails even without a body
        // and at tau .505; periodic MRT and open HRR are the surviving controls.
        let fails = collision == Collision::Mrt && !periodic;
        assert_eq!(row["completed"].as_bool(), Some(!fails), "{row}");
        if fails {
            assert!(row["max_at"][0].as_f64().unwrap() < 0.5, "growth stays near the inlet: {row}");
        }
    }
}
