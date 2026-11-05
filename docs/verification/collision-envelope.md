# Collision and filter operating envelope (#790, #788)

This is a **measurement study**, not a new collision default or a validation of
external-flow LES. It uses the production CPU `Flow` stepping paths on periodic,
analytic geometry. Completion, mode stability, rate-fit quality and continuum
accuracy are separate fields in its output. No collision kernel, command,
boundary, setup floor or default changes.

## Reproduce

Run one study at a time (the ordinary tests include a small analytic control):

```sh
cargo test -p fieldcraft-lbm --test collision_envelope -- --nocapture
cargo test -p fieldcraft-lbm --test collision_envelope wave_operating_envelope -- --ignored --nocapture --test-threads=1
cargo test -p fieldcraft-lbm --test collision_envelope low_tau_long_window -- --ignored --nocapture --test-threads=1
cargo test -p fieldcraft-lbm --test collision_envelope held_out_advected_oblique_modes -- --ignored --nocapture --test-threads=1
cargo test -p fieldcraft-lbm --test collision_envelope short_wavelength_stability -- --ignored --nocapture --test-threads=1
cargo test -p fieldcraft-lbm --test collision_envelope analytic_taylor_green_envelope -- --ignored --nocapture --test-threads=1
cargo test -p fieldcraft-lbm --test collision_envelope nonlinear_taylor_green_re1600 -- --ignored --nocapture --test-threads=1
cargo test -p fieldcraft-lbm --test collision_envelope mrt_bluff_body_reproduction -- --ignored --nocapture --test-threads=1
```

Each `ENVELOPE ` line is a JSON record. Failed runs retain their error and last
completed step/history. A test that successfully *records* a rejected case does
not mean that case passed the physical criteria. All loops have fixed caps:
32768 steps, 64³ cells and 400 million updates per periodic case; the selected 3D matrix
uses at most 32³ cells. Long studies are ignored in the normal gate.

Configurations: TRT (Lambda = 3/16), MRT, recursive regularized, HRR sigma 0.99,
each with no SGS and Smagorinsky C_s = 0.1; additional HRR C_s = 0.1 controls use
sigma 0.9 and dynamic sigma. Molecular tau stays fixed when comparing SGS on/off.
Below tau = 0.505 the no-SGS cases are **diagnostic controls**: setup is validated
with C_s = 0.1, then that coefficient is cleared before constructing `Flow`.
They cannot be requested through the ordinary no-SGS command and do not authorize
lowering its floor. Output labels this exception.

## Definitions and acceptance

The linear matrix uses dx = dt = rho0 = 1, perturbation speed 1e-6, wavelengths
N = 16/32/64 and tau = 0.5001/0.5005/0.505/0.55/0.8. An axis-aligned wave lives
in N x 1 x 1 periodic cells. Shear velocity is perpendicular to its wave vector;
sound is initialized as a right-going density/velocity characteristic. Complex
spatial projection removes the travel phase. It does not confuse the zeros of a
standing wave with decay. Separate held-out cases use N = 24, tau =
0.5005/0.505/0.8, direction (0,1,0) or (1,1,0), and mean speed 0.05 along that
direction. An oblique wave has |k| = sqrt(2) 2pi/N.

A log-amplitude least-squares fit starts at step 128, samples every eight steps
and ends after about 1.5 theoretical decay times, bounded by 512 and 8192 steps
(2048 for held-out and 2D Taylor–Green). The output includes fit residual and
relative difference between fits to the first/second half of the window. Rates
are only accepted when that difference is <1%, log RMS residual <0.001, rate
positive, and mass drift <1e-10. Signals below 1e-13 cannot produce an accepted
fit. Amplification above ten times the initial amplitude stops a wave explicitly;
the production blow-up guard remains active. These are finite-time observations,
not eigenvalue proofs. Health extrema are sampled every eight steps, not claimed
as per-step extrema.

For shear, gamma = nu |k|² and nu_eff = gamma_measured/|k|², with
nu = (tau - 1/2)/3. The low-k viscosity criterion is |nu_eff/nu - 1| <=1%.
The shear/sound decay-rate criterion is <=2%. A wavelength is accepted only
where its measured cases meet these thresholds; no interpolation in tau,
wavenumber, amplitude or orientation is implied. SGS rows at amplitude 1e-6
probe the near-linear limit; they are not exact finite-amplitude SGS solutions.

Sound follows the linear isothermal equations, giving
Gamma = (4nu/3 + zeta)/2 and gamma = Gamma |k|². For TRT and the regularized
operators zeta = 2nu/3 in the long-wave limit, so Gamma = nu. For MRT,
zeta = 2(1/1.19 - 1/2)/9 because the energy/trace moment has rate 1.19.
This is a distinct bulk viscosity, not an error to subtract from the measurements.
The moment viscosities follow d'Humières et al. (2002),
[doi:10.1098/rsta.2001.0955](https://doi.org/10.1098/rsta.2001.0955), section 3
and appendix A; the longitudinal attenuation follows by linearizing continuity
and momentum about rho0 = 1. HRR filtering at finite k is measured against these
continuum rates, not its discrete eigenvalue (which would test implementation
fidelity rather than continuum accuracy).

The analytic 2D Taylor–Green control has velocity
U(sin(kx)cos(ky), -cos(kx)sin(ky), 0), pressure
U²(cos(2kx)+cos(2ky))/4, and exact incompressible amplitude U exp(-2nu k²t).
It uses U=1e-6, N=16/32/64, tau=.5005/.505/.8 and the same fitted-rate criteria.
All reduced grids have a singleton dimension: dynamic HRR's shortest-box-length
cap is one cell there. Their dynamic-sigma result must not be generalized to a
cubic box; the nonlinear 3D cases below exercise the cubic cap.

## Nonlinear 3D Taylor–Green

The independent case is the periodic cube with side 2pi L, Re=U L/nu=1600,
U_lattice=.05, u/U=(sin(x/L)cos(y/L)cos(z/L),
-cos(x/L)sin(y/L)cos(z/L),0), and the usual pressure perturbation
rho U²(cos(2x/L)+cos(2y/L))(cos(2z/L)+2)/16. L=N/(2pi) cells;
tau=.5+3 U_lattice L/1600, run to t U/L=20 at N=16/24/32 without retuning.
The initial Mach number is .0866. This is an isothermal weakly compressible
comparison, not the full energy equation at Mach .1.

The reference is the openly published dataset by Kurz, Kempf, Blind, Kopper,
Offenhäuser, Schwarz, Starr, Keim and Beck (2024),
[GALÆXI Validation: Taylor-Green Vortex](https://doi.org/10.18419/DARUS-4139),
version 1.1, CC BY 4.0. Only numerical result columns were read; no solver source,
input file, mesh or build script was used. The CPU 64³-elements, degree-seven
result (512 points per axis), file 291392, has an incompressible kinetic-energy
column. Adjacent-sample differences give peak -dE/dt = 0.0128393 at t=8.98698.
File 291383 (256 points per axis) gives 0.0128474 at t=8.94505: peak difference
0.063%, time difference 0.47%. These are reference-resolution differences, not
formal uncertainty bounds. The output records peak/time errors against the
512-point values; the roadmap requires **both within 5% after resolution study**.
A single passing coarse-grid peak is insufficient.

The history columns are t, E_x, E_y, E_z, <u_x u_y>, sigma_min, sigma_mean,
max(nu_t/nu), molecular 2nu<S:S>, modelled 2<nu_t S:S>, explicit SGS dissipation.
Velocity energy is volume averaged, matching the reference's incompressible
column; rates are in U³/L. Strain uses periodic centred differences and modelled
viscosity is reconstructed from the sampled non-equilibrium populations using
the production closure. This is a sampled diagnostic, not an exact discrete
energy budget. For dynamic HRR the last column is zero: the estimated SGS
viscosity sets sigma instead of being added to the momentum relaxation. The
residual -dE/dt minus molecular and explicit SGS includes numerical filtering,
compressibility and diagnostic discretization error; it must not be labelled
pure HRR dissipation. Paired regularized/HRR and fixed/dynamic runs isolate
configuration sensitivity without asserting additive filter and SGS effects.

## Interpretation limits

Periodic cases remove inlet, outlet, obstacle and wall-law effects. A failure
here establishes a bulk limitation; survival cannot acquit the boundaries of
#790; the small bluff-body controls below test that distinction. These cases cannot reproduce Musker's +80% cylinder drag in
#788. Its separated wall treatment needs the wall-layer study under #786 and a
controlled cylinder comparison. Dynamic HRR must retain three-dimensional
energy/stress statistics within reference uncertainty before it meets slice 10;
energy suppression by itself is not a validation or a reason to retune sigma.
The original external-flow issues remain open.

For Musker, [the original #788 measurements](https://github.com/tangency-labs/fieldcraft/issues/788)
already put the cylinder run outside the proposed study validity bounds:
rho=.920–1.106 exceeds the 2% density envelope and local Mach .26 exceeds .2.
The matching point at 1.5 cells is .125D from the separating surface at N=12.
Those recorded conditions and the equilibrium wall law's scope prevent treating
the +80% drag change as an accepted model comparison. They do not establish
which part of wall sampling, stress imposition or separation causes the bias.
No new Musker measurement is claimed here; the requested/applied/carried-stress
study under #786 is the prerequisite for that attribution.

## Measurements on the CPU

Measured 2026-11-05 with the unchanged 0.7.5 kernels at `6b613217`, f64,
eight worker threads. The [wave rows](collision-envelope/waves.csv),
[3D summaries](collision-envelope/taylor-green.csv) and
[3D histories](collision-envelope/taylor-green-history.csv) retain failures and
fit diagnostics. The CSV files are original FieldCraft measurements; they do not
contain the external DNS histories. Fixed sigma columns have no effect outside HRR.

The first 300 axis-wave, 120 advected/oblique and 90 analytic Taylor–Green runs
all completed. That is not their acceptance result: 79/120 advected/oblique and
67/90 analytic Taylor–Green rows passed the 2% decay check including fit quality.
The low-tau axis fits exposed persistent kinetic transients. A separate 20-case
repeat fits steps 20,000–32,768, keeping the original short-window rows. It
resolves the misleading short-window viscosity estimates without widening any
accuracy tolerance.

Low-k shear viscosity error at N=64, from the late fit (percent):

| Collision/filter | SGS | tau .5001 | tau .5005 | 1% criterion |
|---|---|---:|---:|---|
| TRT | none | -0.0354 | -0.0402 | pass both |
| TRT | C_s=.1 | -0.0329 | -0.0397 | pass both |
| MRT | none | +0.0803 | +0.0803 | pass both |
| MRT | C_s=.1 | +0.0828 | +0.0808 | pass both |
| Regularized | none | +0.0803 | +0.0802 | pass both |
| Regularized | C_s=.1 | +0.0828 | +0.0806 | pass both |
| HRR sigma .99 | none | +6.134 | +1.290 | fail both |
| HRR sigma .99 | C_s=.1 | +6.136 | +1.291 | fail both |
| HRR sigma .9 | C_s=.1 | +63.478 | +12.752 | fail both |
| HRR dynamic (one-cell cap) | C_s=.1 | +0.170 | +0.0836 | pass both, reduced-grid cap only |

Thus HRR's excess damping near tau=1/2 is present **without SGS**, and cannot be
explained by double-counting the explicit eddy viscosity. The regression
`low_tau_hrr_filter_exceeds_the_viscosity_budget` checks a regularized control
within 1% and the HRR .99 result outside 1% at tau .5005. Sigma .9 adds much
more filtering at the same molecular viscosity. These observations quantify a
mechanism that can affect #788; they do not calibrate sigma against cylinder drag.

For the short-window axis study, both shear and sound meet the 2% decay criterion
at the following **sampled** wavelengths. These are conditional on the stated
amplitude, zero mean flow, time window and (for dynamic HRR) one-cell cap; the
held-out table must still be checked before applying them elsewhere.

| Configuration | tau .505 | tau .55 | tau .8 |
|---|---|---|---|
| TRT, MRT, regularized, each SGS off/on | 16, 32, 64 | 16, 32, 64 | 16, 32, 64 |
| HRR .99, SGS off/on | 32, 64 | 16, 32, 64 | 16, 32, 64 |
| HRR .9, C_s=.1 | 64 | 32, 64 | 16, 32, 64 |
| HRR dynamic, C_s=.1, one-cell cap | 16, 32, 64 | 16, 32, 64 | 16, 32, 64 |

An additional 160 cases with N=4/8, mean speed .05, tau=.5001/.5005,
axis/diagonal waves and both polarizations completed without tenfold growth of
the seeded mode. Many of their decay fits are inaccurate or unresolved; these
short wavelengths are stability probes, not additions to the resolved band.

The low-k 1% criterion is stricter: HRR .9 still misses it at tau .505
(+1.338%). Short-window rows at .5001/.5005 with inconsistent fits remain
inconclusive; the late shear repeat does not retrospectively accept their sound
rates or all their other wavelengths.

### Re1600: no accepted configuration at 16³/24³/32³

All 30 cases were attempted. TRT without SGS failed the production blow-up guard
on all three grids; the other 27 finished. **None passed both peak magnitude and
peak time within 5%.** The full-window dissipation peak includes startup; MRT
and several coarse regularized/HRR runs have their largest peak in the first
sample interval. That is a failure of this equilibrium-start, coarse-grid
measurement to match the reference, not evidence of correctly timed turbulent
dissipation. No startup peak is silently removed from acceptance.

At N=32 (reference peak 0.0128393 at t=8.98698):

| Collision/filter | SGS | Peak -dE/dt | Peak time | Result |
|---|---|---:|---:|---|
| TRT | none | — | — | blow-up, step 565 |
| TRT | C_s=.1 | .011098 | 7.0293 | fail |
| MRT | none | .046387 | .03927 | startup peak, fail |
| MRT | C_s=.1 | .046803 | .03927 | startup peak, fail |
| Regularized | none | .013020 | 7.0293 | magnitude within 5%, time fails |
| Regularized | C_s=.1 | .012963 | 6.7937 | magnitude within 5%, time fails |
| HRR .99 | none | .013247 | 7.0293 | magnitude within 5%, time fails |
| HRR .99 | C_s=.1 | .013131 | 6.7152 | magnitude within 5%, time fails |
| HRR .9 | C_s=.1 | .021610 | .03927 | startup peak, fail |
| HRR dynamic | C_s=.1 | .065407 | .03927 | startup peak, fail |

At the sample nearest t=10 on N=32, HRR .99 with C_s=.1 has spanwise energy
E_z=.0067154; dynamic sigma has .0026911, **59.9% lower**. At t≈20 those values
are .0016546 and .00041045, **75.2% lower**. The minimum sampled dynamic sigma is
.00023375. This independently reproduces the direction of #788's suppressed
spanwise motion in a wall-free flow, although its magnitude differs from the
cylinder's and there is no converged directional DNS statistic here. Dynamic
sigma's 3D energy/stress acceptance criterion remains unmet.

### #790 reproduction: the body is not necessary for the failure

The additional [bluff-body controls](collision-envelope/bluff-body.csv) reproduce
the issue's CPU quasi-2D arm: D=U=rho=1, Re=3900, eight cells per D, box
24D x 12D x D/8, cylinder at (6D,6D) with radius D/2, 1024 analytic segments,
uniform velocity inlet, convective pressure outlet, slip y faces, periodic z,
C_s=.1, lattice speed .05, tau=.5003076923. The original spanwise seed cancels
in a one-cell span (apart from sin(pi) rounding); it is preserved, not replaced
by a stronger planar seed. Runs stop at 1200 steps (7.5 D/U), bounded by 20,000
cells and 24 million updates.

| Control | Result | Maximum-speed position (x/D,y/D) |
|---|---|---|
| Original MRT cylinder | **fails at step 599**, lattice speed .55955 | (.3125,7.6875) |
| MRT, remove cylinder only | **fails at step 864**, speed .51371 | (.1875,6.5625) |
| MRT cylinder, periodic x instead of inlet/outlet | completes 1200, speed .07767 | (7.5625,7.0625) |
| Original cylinder, HRR .99 instead of MRT | completes 1200, speed .07957 | (7.5625,4.9375) |
| MRT cylinder, increase viscosity to tau .505 (Re240) | **fails at step 857**, speed .51721 | (.3125,7.1875) |

The original failure step and speed match #790's recorded CPU result. Growth
reaches the guard within three cell centres of the inlet, far upstream of the
cylinder. Removing the body does not cure it; changing the x boundaries to
periodic does. Together with the periodic MRT low-k and Re1600 controls, this
locates a **collision/open-boundary interaction** rather than a blanket failure
of MRT at every LES tau, and rules out the body as a necessary trigger in this
reproduction. Raising tau to .505 alone does not cure it either.

This does not distinguish an inlet closure defect from a coupled inlet/outlet
mode or establish the 3D GPU trigger. The periodic-body control also allows the
mean flow to decelerate without an inlet, so it is a diagnostic, not an identical
long-time flow. The characterization test asserts failure of the three open-MRT
cases, inlet-local maximum speed, and survival of the periodic-MRT/HRR controls.
The boundary corrections in #826/#832 do not change this control's convective
MRT path (#832 changes TRT); no boundary fix is duplicated here.
No production correction or validated cylinder prediction is delivered by this
measurement PR; #790 and #788 remain open.
