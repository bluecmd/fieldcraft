mod common;
use common::*;
use fieldcraft_lbm::{Backend, Flow, Limits, Setup, coupled::CoupledFlow};
use fieldcraft_model::{BoxFace, FaceCondition, FluidThermal, Outflow, Profile, ThermalCondition};

fn pair(power: f64, open: bool, zero: f64) -> ([Setup; 2], Vec<fieldcraft_lbm::Obstacle>) {
    let obstacles = vec![obstacle("Separator", cuboid([0.0, 2.0, 0.0], [4.0, 3.0, 2.0]), &CUBOID_FACES)];
    let setups = std::array::from_fn(|side| {
        let mut d = domain([0.0, side as f64 * 2.0, 0.0], [4.0, 3.0 + side as f64 * 2.0, 2.0], 1.0);
        d.obstacles = vec!["Separator".into()];
        let temp = 300.0 + 40.0 * side as f64;
        d.thermal = Some(FluidThermal {
            initial_temperature: temp,
            temperature_zero: if side == 0 { 273.15 } else { zero },
            solids: vec![conducting("Separator", power)],
            ..Default::default()
        });
        if open {
            let (entry, exit, velocity) = if side == 0 { (BoxFace::XMin, BoxFace::XMax, 0.01) } else { (BoxFace::XMax, BoxFace::XMin, -0.01) };
            *d.faces.get_mut(entry) = FaceCondition::Velocity { velocity: [velocity, 0.0, 0.0], profile: Profile::Uniform };
            *d.faces.get_mut(exit) = FaceCondition::Pressure { pressure: 0.0, outflow: Outflow::Uniform };
            *d.thermal.as_mut().unwrap().faces.get_mut(entry) = ThermalCondition::Temperature { value: temp };
        }
        let m = cht_model(
            [if side == 0 { 4.0 } else { 20.0 }, 0.1, if side == 0 { 0.05 } else { 0.02 }],
            (10.0, [0.2; 3], 1.0),
            d,
            step(4000.0, 1.0, None),
        );
        let mut setup = Setup::new(&m, None, &obstacles, &Limits::default()).unwrap();
        setup.step = format!("Circuit{side}");
        setup
    });
    (setups, obstacles)
}

#[test]
fn closed_different_fluids_share_one_capacity_and_conserve_energy_with_distinct_zeros() {
    let (setups, obstacles) = pair(0.0, false, 0.0);
    let mut coupled = CoupledFlow::new([&setups[0], &setups[1]], [&obstacles, &obstacles], "Separator", 320.0, &Limits::default()).unwrap();
    assert_eq!(coupled.wall_cells(), 8);
    let initial = coupled.energy();
    for _ in 0..4000 {
        let e = coupled.step().unwrap();
        assert!(e.wall_residual.abs() < 1e-9, "{e:?}");
        assert_eq!(e.wall_power, 0.0);
    }
    assert!((coupled.energy() - initial).abs() < 1e-7, "{}", coupled.energy() - initial);
    let exact = (16.0 * 4.0 * 300.0 + 16.0 * 20.0 * 340.0 + 8.0 * 10.0 * 320.0) / (16.0 * 4.0 + 16.0 * 20.0 + 8.0 * 10.0);
    for (side, setup) in setups.iter().enumerate() {
        let fields = coupled.fields(side).unwrap();
        let zero = setup.thermal.as_ref().unwrap().zero;
        for temp in fields.temperature {
            assert!((temp + zero - exact).abs() < 1e-5, "{temp} + {zero} vs {exact}");
        }
    }
}

#[test]
fn power_is_applied_once_and_open_boundary_exchange_closes_the_combined_budget() {
    for open in [false, true] {
        let (setups, obstacles) = pair(0.2, open, 273.15);
        let mut coupled = CoupledFlow::new([&setups[0], &setups[1]], [&obstacles, &obstacles], "Separator", 320.0, &Limits::default()).unwrap();
        let initial = coupled.energy();
        let mut expected = 0.0;
        for _ in 0..100 {
            let e = coupled.step().unwrap();
            assert!((e.wall_power - 0.2).abs() < 1e-14);
            expected += e.wall_power + e.open.iter().sum::<f64>() - e.external.iter().sum::<f64>();
        }
        let residual = coupled.energy() - initial - expected;
        assert!(residual.abs() < 1e-8, "open={open}: residual {residual} J, expected {expected} J");
    }
}

#[test]
fn splitting_a_single_fluid_preserves_the_existing_conjugate_step() {
    let (_, obstacles) = pair(0.0, false, 273.15);
    let mut d = domain([0.0; 3], [4.0, 5.0, 2.0], 1.0);
    d.obstacles = vec!["Separator".into()];
    d.thermal = Some(FluidThermal { initial_temperature: 300.0, solids: vec![conducting("Separator", 0.1)], ..Default::default() });
    let m = cht_model([4.0, 0.1, 0.05], (10.0, [0.2; 3], 1.0), d, step(100.0, 1.0, None));
    let full_setup = Setup::new(&m, None, &obstacles, &Limits::default()).unwrap();
    let mut full = Flow::new(&full_setup, &obstacles).unwrap();
    let setups = std::array::from_fn::<_, 2, _>(|side| {
        let mut d = full_setup.domain.clone();
        d.min[1] = side as f64 * 2.0;
        d.max[1] = 3.0 + side as f64 * 2.0;
        let m = cht_model([4.0, 0.1, 0.05], (10.0, [0.2; 3], 1.0), d, step(100.0, 1.0, None));
        let mut s = Setup::new(&m, None, &obstacles, &Limits::default()).unwrap();
        s.step = format!("Circuit{side}");
        s
    });
    let mut coupled = CoupledFlow::new([&setups[0], &setups[1]], [&obstacles, &obstacles], "Separator", 300.0, &Limits::default()).unwrap();
    for _ in 0..100 {
        full.step().unwrap();
        coupled.step().unwrap();
    }
    let whole = full.fields().unwrap();
    for (side, setup) in setups.iter().enumerate() {
        let fields = coupled.fields(side).unwrap();
        for (cell, temp) in fields.temperature.iter().enumerate() {
            let other = full_setup.grid.cell_at(setup.grid.centre(cell)).unwrap();
            assert!((temp - whole.temperature[other]).abs() < 1e-11, "{side} {cell} {temp} vs {}", whole.temperature[other]);
        }
    }
}

#[test]
fn unsupported_alignment_ownership_and_steady_settings_are_named() {
    let (setups, obstacles) = pair(0.0, false, 273.15);
    for kind in ["alignment", "overlap", "spacing", "wall", "steady", "memory"] {
        let mut s = setups.clone();
        let mut limits = Limits::default();
        match kind {
            "alignment" => s[1].grid.origin[0] += 0.25,
            "overlap" => {
                s[1].grid = s[0].grid;
                s[1].domain = s[0].domain.clone();
            }
            "spacing" => s[1].grid.spacing = 0.5,
            "wall" => s[1].domain.thermal.as_mut().unwrap().solids[0].power = 1.0,
            "steady" => s[0].thermal.as_mut().unwrap().solids[0].capacity_scale = 0.1,
            _ => limits.max_memory = 1,
        }
        let error = CoupledFlow::new([&s[0], &s[1]], [&obstacles, &obstacles], "Separator", 320.0, &limits).err().unwrap().to_string();
        assert!(error.contains("coupled") || error.contains("Circuit"), "{kind}: {error}");
    }
}
