# FieldCraft LES: design review and recovery plan

2026-11-04. Tracking: [#785](https://github.com/tangency-labs/fieldcraft/issues/785).
Code reviewed: `develop` at `4b680571`. Measurements labelled **audit** below are
from `cb976117` and an earlier local audit (not published); its numbers are
reproduced in the linked issues. They are not new measurements on this revision.
See `PROGRESS.md` for new runs and claims.

## Verdict: re-architect parts, retain the LBM core

FieldCraft has credible ingredients for modern LBM LES, but does **not yet have
a validated predictive LES system**. Keep its Rust implementation, deterministic
CPU backend, GPU layouts, model/command boundaries, collision kernels, forcing,
and thermal/conjugate machinery. Repair the open boundaries first; develop
explicit contracts for wall momentum/heat transfer, numerical filtering,
geometry/refinement and statistical acceptance. These are substantial changes to
the numerical architecture, not grounds for a wholesale solver rewrite.

HRR is a published LES method, not a dead end. Conversely, selecting cumulant,
WALE or a different wall law would not by itself cure this evidence. Contemporary
comparisons evaluate collision, SGS and wall treatment together: Spinelli et al.
[R3] compare MRT, HRR and cumulant on D3Q27 at Re_tau 1000, finding different
accuracy/performance tradeoffs. That motivates a controlled alternative later,
not a claim that their preferred combination automatically works in FieldCraft.
Recent work by Waters et al. [R23] still compares explicit SGS against implicit
cumulant filtering through spectral bandwidth, reinforcing the need to measure
resolved scales and numerical dissipation rather than rank model names.

Three independent acceptance failures dominate:

* **Boundary stability fails before turbulence matters.** #819 loses an exact
  equilibrium in an empty four-cell box; #820 amplifies a seeded inlet mode in
  an empty slip-sided box. #792 locates the sphere failure at the inlet. A
  better SGS model cannot repair a laminar boundary eigenmode.
* **The wall law and transported wall stress disagree.** #786 has inferred
  C_f +2.0% against Dean but carried C_f −16.75%; requested/carried stress is
  1.245. The momentum budget closes. Reporting the inferred force instead would
  conceal the error. The on-node Han scheme carries its stress but still misses
  the channel drag badly: conservation is necessary, not sufficient.
* **Completed runs are being mistaken for acceptable evidence.** #791 has
  negative mean drag, #793 excessive density excursions, #796 long-time drift,
  and #787–789 no converged drag/frequency result. The accepted laminar tests and
  backend comparisons do not cover these failures.

The work should establish one trustworthy LES configuration and its domain of
validity before broadening the menu. Hybrid RANS–LES/DES and wall-modelled heat
transfer depend on these repairs; they are not small additions to `Turbulence`.

## Evidence and current implementation

The source is authoritative where older documentation disagrees. All four
collisions and both current SGS models run on CPU and GPU; some literature and
manual paragraphs still say HRR/Vreman are CPU-only. The momentum lattice is
D3Q19; D3Q27/cumulant and spatial refinement remain planned. The scalar lattice
is D3Q7. No public solver sources or commercial products were used in this review.

| Issues | Evidence | Design consequence |
|---|---|---|
| #786 | M&S Re_tau 950, N=10: carried C_f 0.0043803, −16.75% vs Dean; model C_f 0.0053667, +2%; interval on force only 0.82% | Separate wall-law inference, imposed traction and measured impulse; gate the carried force |
| #787, #788 | Re3900 cylinder C_d 1.354/1.323/1.338 at N=12/16/24 vs experimental 0.99; Musker +80%, dynamic HRR +22% and greatly reduced spanwise energy | Require 3D resolution, spectra, wall-treatment and dissipation studies; no universal drag multiplier |
| #789 | Square Re22000 C_d 4.11 vs about 2.1; axis-aligned faces, force spikes, invalid crossing frequency | Curved-wall staircasing cannot explain every error |
| #790, #791 | MRT fails within one D/U in 3D cases; coarse TRT finishes with negative drag | Test the coupled collision/boundary scheme and physical diagnostics, not just operator algebra |
| #792, #819, #820 | Inlet-local sphere growth; uniform-outlet equilibrium growth; slip-sided inlet checkerboard | Two distinct reduced boundary regressions before another large external-flow sweep |
| #793, #797 | Completed runs with rho 0.72–1.37 and local Ma 0.35; halving lattice speed greatly reduces density variation without fixing drag | Runtime validity diagnostics and independent Mach sensitivity; stability limit is not an accuracy limit |
| #794 | Halfway Spalding checkerboard grows across every tested collision/tau; on-node version damps it | Refuse the known unstable variant in production, preserve diagnostic history without silently changing wall position |
| #795 | Signed lag-1 condition accepted anticorrelation | **Already fixed** by `b7cbfe0d` on current develop; do not submit a duplicate |
| #796 | Re3900 mean rises 22.6% in the 400–600 D/U window; narrow intervals can coexist with bias | Require stationarity, block-length sensitivity and independent spatial/model error checks |
| #798 | Stokes K=2.25717, 2.054996, 2.068625 at R=3,6,9 vs series 2.0215; cylinder sensitive to placement | The recent R9 comment reverses the apparent two-grid convergence; require three grids and translation controls |

### Collision and the numerical filter

`flow.rs`, `regularized.rs`, `mrt.rs` and `hrr.rs` already separate useful
kernels. TRT remains valuable for laminar controls and its halfway wall
properties. MRT's free relaxation rates are a scheme choice, not a guarantee of
high-Re stability (#790). Recursive regularisation removes non-hydrodynamic
content, but the existing regularized density outlet additionally replaces the
whole distribution with equilibrium. Its survival in #819 therefore does not
isolate the effect of *bulk* regularisation.

HRR blends the population stress with a finite-difference stress. Its spatial
stencil, one-sided boundary derivatives, sigma and relaxation time jointly set
dispersion and dissipation. The manual's channel spectra show HRR filtering
dominating explicit SGS dissipation in the problematic band. Dynamic sigma uses
the SGS estimate to set filtering and omits that viscosity from the momentum
relaxation; this is an implicit LES choice, not a dynamic Smagorinsky model.
It needs its own accuracy envelope, including #788's loss of spanwise motion.
Jacob et al. [R1] provide the method; Coreixas et al. [R2] explain why equilibrium,
moment representation and regularisation must be compared together.

Keep fixed-sigma HRR as the first LES candidate, with no declaration that 0.99
is universally optimal. Measure shear/acoustic-wave decay versus wavenumber,
orientation and tau, then Taylor–Green energy transfer and channel stress.
Do not tune sigma or C_s to one drag coefficient. D3Q27 cumulant [R4] is a later
comparison branch, accompanied by consistent forcing, walls and boundaries;
changing only the collision name would be an uncontrolled experiment. D3Q27
also increases population storage/traffic by 27/19 before other overheads.

### SGS model and filter width

`turbulence.rs` implements the force-corrected non-equilibrium Smagorinsky
closure; `vreman.rs` uses the velocity gradient shared with HRR. The audit's
simple-shear check and formula review found no factor-of-two or lattice-unit
error. Keep those invariants. The grid spacing is currently the implicit filter
width; explicit SGS viscosity and collision filtering are different mechanisms.

WALE [R5] provides the near-wall y-cubed scaling without an explicit damping
function; it is a useful later model, not a substitute for a wall model on a
coarse mesh. Vreman [R6] already supplies a different algebraic closure with
laminar zero-viscosity cases. A dynamic model would need an actual test filter,
averaging/regularisation policy and boundary/refinement treatment. Near-wall
vanishing of an SGS viscosity is appropriate for a resolved no-slip wall, but
does not automatically provide missing stress below a WMLES matching plane.

Represent molecular viscosity, physical modelled stress/diffusivity and
numerical filtering separately. In particular, do not infer turbulent thermal
diffusivity from HRR's artificial dissipation. Before hybrid thermal work,
explicitly audit the dynamic-sigma path's scalar closure and its meaning.

### Walls, sampling and carried stress

`wall.rs` uses a nearest halfway-link wall point, with distance propagation that
does not cross periodic faces. Musker samples the second cell, rebuilds the
first and supplies its wall viscosity. `bounce.rs` implements Han's on-node
scheme [R8], with a different physical wall location and restricted geometry.
Neither is a validated general wall treatment for separated external flow.

Target a wall interface with physical position/normal, sample height, sampling
stencil, relative velocity, requested tangential traction and **actual momentum
impulse**. Preserve pressure/normal impermeability separately. The returned
impulse must include every reconstruction/forcing/link contribution once. The
surface used for wall area and force integration must be the same one used for
sampling distance and geometry, including on-node versus halfway conventions.

This does not mean replacing Musker by another formula. M&S [R7] and Han [R8]
describe different imposition schemes. Asmuth et al. [R9] show that directly
imposing stress still leaves near-wall-gradient problems. Kawai–Larsson [R10]
motivate sampling outside the most contaminated first cells. First expose the
stress balance; then assess matching height at fixed physical h/delta while
refining, with enough modelled stress below h. Avoid prescribing both the full
velocity profile and independent shear traction as conflicting constraints.

Retain the old M&S reproduction as a reproduction, explicitly distinguished
from engineering validation. Its published E measure decreases with point
count; report an ordinary RMS and maximum relative profile error as well.
Do not identify a paper's inferred C_f with the simulated momentum transfer.
Non-equilibrium/pressure-gradient wall models are later work, gated on an
attached channel first and a separate separation case afterwards.

### Inlet, outlet and far field

The current open-cell reconstruction is Zou–He with D3Q19 transverse correction
[R11]. The convective outlet updates incoming populations, shifts density and
falls back to the uniform pressure closure on backflow. Slip side faces are
reflecting boundaries, not an unbounded far field. Corners and mixed faces need
explicit ownership and compatibility rules.

Use #819 and #820 to identify which kinetic modes grow; preserve their periodic,
convective, zero-seed and relaxation controls. Latt et al. [R12] compare closure
and whole-node replacement approaches and their stability/accuracy tradeoff.
A boundary reconstruction that retains viscous stress is a candidate to test;
zeroing all non-equilibrium stress merely to pass the empty-box test could
repeat the documented outlet-profile error. No particular replacement is yet
selected by this review.

After the local fixes, test an advected vortex/acoustic pulse, oblique inflow,
backflow and finite-domain sensitivity. A later far-field/sponge or
characteristic option must measure reflection and mean-flow bias. Turbulent
inlets for developing boundary layers and hybrid regions need specified
intensity, length scales and correlated fluctuations, with deterministic seeds.
Changing to a convective outlet is a control, not closure of #819 or #792.

### Force evaluation

Retain momentum exchange and the reconstruction impulse. The audit's independent
budgets close at about 1e-14 of total momentum in the channel and 7e-12 lattice
force for a periodic cylinder. That makes an omitted static-body Galilean term
an implausible explanation: the moving-body correction vanishes here.

Add an independent control-volume balance for open cases, including momentum
storage, flux through open faces and forcing. Changing curved-wall interpolation
must change the force accounting consistently; force is the actual population
momentum transfer [R13], not an area-scaled correction to match a benchmark.
Separate pressure and shear estimates only when their discrete definitions close
to the total. Report total-force spectra before interpreting noisy lift crossings
as physical shedding. Moving/deforming FSI later needs relative-velocity terms,
geometric conservation and a separately validated transfer interface.

### Geometry, refinement and feasible resolution

`voxel::Grid` is one uniform cubic grid; obstacle momentum walls are staircase
halfway links. The thermal interface work does not establish an interpolated
momentum boundary. Add robust link intersections and wall fractions before
claiming curved-boundary accuracy. Bouzidi et al. [R14] are a candidate method,
with dedicated mass, force and small-link-fraction tests; interpolation alone
does not guarantee conservation or stability.

For N cells per body diameter, tau = 0.5 + 3 U_lattice N/Re. At Re3900,
U_lattice=0.05, N=12 gives 0.500462. A blanket floor 0.505 would require N>=130;
0.508 would require N>=208. Lowering Mach at fixed N also lowers tau. A floor
is a setup constraint, not a resolution certificate, and raising it alone would
make useful external LES prohibitively expensive on uniform grids.

Resolution must cover the boundary layer or its wall-model exchange region,
separated shear layers, wake and three-dimensional span, with fixed physical
geometry and consistent seeding. Nominal cells/diameter and y-plus alone are
insufficient. For wall-resolved planning, start with first-cell y-plus near 1
and streamwise/spanwise spacing of roughly 50/20 wall units, then establish
case-specific convergence; these are planning targets, not guaranteed accuracy.
WMLES also needs outer-layer resolution and a valid matching height.

Later block refinement must specify level-local dt, population/stress scaling,
restriction/prolongation, halo ordering, reflux or equivalent conservation,
and filter width across levels. Reusing an octree container is not this method.
Lagrava et al. [R15] supply a published refinement approach; Astoul et al. [R16]
identify kinetic-mode conversion and spurious waves at refinement interfaces.
Validate waves and a vortex crossing an interface before a turbulent wake.

For #798, keep physical geometry fixed when varying grid phase: translate the
whole configuration or shift the lattice origin consistently. Moving just a
cylinder in its confined channel changes physical wall clearance as well as
voxel phase. Use the periodic sphere to isolate phase effects; its analytic-
volume and voxel-volume drag normalisations must both be recorded.

### Statistics, convergence and validity

Keep execution status separate from numerical validity, statistical convergence
and validation against a reference. `steady.rs` and the study-local `Blocks`
serve different purposes; neither should be advertised as a general LES
accuracy certificate. #795 is fixed, but |lag1|<0.5 alone is not proof of
independence or stationarity. Higher-lag dependence can remain.

Use bounded sampling with burn-in, a minimum number of complete blocks,
Student-t intervals for small block counts, a block-length sensitivity check,
half-window drift, and additional lags or integral-correlation estimates. Longer
averaging must not silently make a drifting case pass. Record extrema over the
sampling window, not only the last frame: density, local Mach, force spikes,
mass/momentum balance, model viscosity and filtering activity. A spike is a
diagnostic, not a universally invalid flow event.

For the low-Mach benchmark programme, initially require local Ma<=0.2 and
max |rho/rho_ref−1|<=0.02 throughout the accepted window, or an explicitly
justified case-specific envelope. These are proposed study gates, not new
universal production abort limits. Pressure-driven cases need the intended
pressure/density gradient included in rho_ref. Keep the existing blow-up guard.

A reported confidence interval covers sampling uncertainty only. Report grid,
domain, Mach and model sensitivities separately. Freeze passing references and
recipes in `tests/oracle/`, with theory/experiment/published-benchmark source
tags. A fixture of a known biased coarse result guards reproducibility but must
not be called a successful validation. No widening of tolerances to bless a fix.

### GPU parity and hybrid readiness

Keep the CPU f64 reference and deterministic reductions. Mirror each numerical
change in GPU kernels and both streaming layouts, including odd/even steps,
backflow, wall ownership and history/checkpoint state. Small laminar problems
should agree in fields/forces to fixed tolerances. Chaotic long runs should be
compared through converged statistical intervals and budgets, not pointwise
trajectory equality. The audit's excellent laminar parity establishes a useful
base, not LES validation. Hardware testing uses the GB10 handover protocol;
software CI alone cannot satisfy the hardware check.

DES needs transported RANS state (e.g. a selected one- or two-equation model),
wall distance, production/destruction/diffusion, inflow/restart conditions,
bounded time integration and a documented LES length scale. DDES shielding
[R17] addresses modelled-stress depletion/grid-induced separation; simply
blending RANS and SGS viscosity by distance does not. The transition region
also needs resolved turbulence, without double-counting the explicit SGS and
HRR filter. Hybrid state must be portable across CPU/GPU, checkpoints and levels.

For conjugate heat transfer, share physical interface geometry and carry equal
and opposite heat flux into the solid. Add a thermal wall law or wall-layer
energy equation, with its own exchange temperature, conductivity and turbulent
Prandtl assumptions. D3Q7 advection/diffusion requires scalar accuracy checks in
the selected velocity range. Momentum wall validation does not validate heat
transfer; Kuwata–Suga [R18] explicitly treat both wall shear and heat flux.
Steady heat-capacity acceleration cannot be used to claim transient thermal
accuracy. DES and turbulent CHT are separate later capabilities.

## Target architecture

Keep the current crate layers; introduce these contracts incrementally inside
`lbm` and mirror their data in `lbm-gpu`. Avoid a generic abstraction rewrite
before a measured need.

| Responsibility | Contract and owned data |
|---|---|
| Setup/model | Validated numerical configuration; physical geometry; limits; explicitly supported collision/boundary/thermal combinations |
| Grid/geometry | Fluid ownership, link intersections, physical normals/areas, wall distances and sampling stencils; later level/halo map |
| Closure | Physical SGS/RANS stress and scalar diffusivity; filter width; bounded optional transported state |
| Collision/filter | Conservative kinetic update; explicit molecular/model viscosity inputs; numerical-filter parameters and diagnostics |
| Boundary update | Prescribed macroscopic constraints plus kinetic closure; actual mass, momentum and heat exchange; mixed-face ownership |
| Wall layer | Sample at physical h; produce traction/heat request; compare with applied impulse and carried stress below h |
| Backend | Execute the same ordered stages; device reductions/readback and checkpoint lifecycle; no independent physical model selection |
| Acceptance/results | Bounded histories, budgets, validity flags, statistical status, reproducible benchmark metadata; shared commands expose them |

The dependency order is geometry/sample -> closure/wall law -> collision and
boundary transport -> measured exchanges -> acceptance. A wall-model request
must never be substituted for the last stage's measured force. New user-facing
diagnostics/options go through existing model and solve commands with schema,
hostile-input, save/reload and CPU/GPU coverage.

## Staged roadmap and acceptance numbers

These are **proposed gates**, fixed before running the corresponding changes,
not claims of achieved accuracy. Each row is one independently reviewable PR;
split CPU infrastructure and GPU parity only if the unsupported path is refused
explicitly. Fast representative tests run normally; sweeps and long studies are
heavy. Work begins only after the issue can be claimed. Future capabilities
without a sub-issue need a new issue and agreement first. Re-architecture PRs
wait for the maintainer's response on #785.

| Slice / issue | Change and validation case | Acceptance |
|---|---|---|
| 0 / #795 | Verify upstream anticorrelation repair; no duplicate PR | Alternating 1±1e-4 blocks, lag1≈−0.95: rejected despite <1% interval; both channel stop sites use `decorrelated()` |
| 1 / #819 | Repair uniform-pressure boundary after isolated-mode diagnosis; empty box and pressure-driven channel | All four collisions, 4×1×1 and 16×8×2, 30,000 steps, tau {0.5001,0.505,0.518,0.55}; CPU max density/velocity departure <=1e-12; low-tau cases explicitly use a supported SGS setup. Both face signs/all axes; preserve existing channel accuracy bounds. GPU proposed <=1e-5 absolute density/velocity over the long empty-box run |
| 2 / #820 | Repair inlet/slip interaction independently; seed 1e-6, sign reversed, periodic controls | 16³, 30,000 steps, tau {0.5005,0.505,0.518}; no growing tail and final max velocity departure <=1e-6, density departure <=1e-6; sub-floor case explicitly uses supported SGS configuration. Compare the whole time history, allowing a bounded initial acoustic transient |
| 3 / #792 | Re-run original sphere after boundary repair, preserving geometry and tau | Original TRT run reaches 120 D/U; rho in [0.98,1.02], local Ma<=0.2, last-window lift RMS <1e-3. Drag within 5% of a selected documented Re100 reference only after domain/grid study; do not close the remaining geometry/drag gap merely for survival |
| 4 / #794 | Production refusal of known unstable halfway variant; keep old files readable with actionable solve error | Every command/backend route refuses before allocating a solve; no silent migration to a wall half a cell away. Existing on-node checkerboard growth <=0. Legacy numerical sweep retained only through an explicitly isolated diagnostic path if agreed |
| 5 / #793, #791, #797 | Runtime validity report (one topic under #793); analytic injected-field tests and audit histories | Report window density/Mach extrema accurately (CPU 1e-12, GPU 1e-5 absolute on bounded synthetic fields), flag the proposed 0.02/0.2 study envelope, preserve blow-up errors, never label completed as validated; expose sampling cadence and memory caps |
| 6 / #796 | Statistical acceptance helper and study integration | >=20 complete blocks; 95% mean interval <1%; half-window drift <1%; |lag1|<0.5 plus longer-lag check; doubled block length changes half-width <20%. Alternating, lag-2 correlated and drifting synthetic signals rejected; replays of #796 do not pass. Reaching time cap reports inconclusive |
| 7 / #786 | Expose and gate requested/applied/carried stress, without changing wall numerics | CPU closed-box momentum residual <=1e-12 of a documented momentum scale (with absolute roundoff floor); GPU <=1e-5. Original N10 retains its ~1.245 request/carried discrepancy and is explicitly a failed carried-drag validation |
| 8 / #786 | After design agreement: stress-carrying wall layer and controlled matching height; channel Re_tau 950 then 1000/2000 | Requested/applied mean stress within 1%; carried C_f within 5% of selected DNS, with Dean separately reported; ordinary profile RMS <=5% above h and total-stress budget error <=5% of wall stress. At least N=10/20/30 and fixed h/delta refinement; no C_s/sigma retuning per grid |
| 9 / #798 | Interpolated momentum walls; periodic Stokes sphere and confined laminar cylinder | Sphere R=3/6/9 plus a finer point and >=4 sub-cell phases: finest drag-factor error <=2%, phase spread <=2%, no unsupported monotone-order claim. Closed periodic mass drift <=1e-8 relative per steady study. Cylinder C_d,max within 2% of published 3.22–3.24 band and St in [0.295,0.305], with <2% grid-phase sensitivity at the accepted resolution |
| 10 / #790, #788 | Quantify collision/filter operating envelope; waves and Taylor–Green, no new default | Low-k effective viscosity within 1%; shear/acoustic decay error <=2% over declared resolved band; stable held-out modes. Re1600 Taylor–Green peak dissipation/time within 5% of a cited DNS after resolution study. Record filtering and SGS contributions separately; dynamic sigma must retain 3D energy/stress statistics within the reference uncertainty |
| 11 / #787 | Re3900 cylinder validation after local repairs | Three geometrically consistent 3D grids; last two mean C_d differ <3%, span/domain/Mach sweeps <2%; mean C_d in [0.94,1.04], St in [0.210,0.220] against the experimental compilation in Kravchenko–Moin [R19]. Pass slice 6 statistics and field validity; no raw-crossing St accepted without a compatible spectrum |
| 12 / #789 | Independent square-cylinder validation | Match Lyn et al. [R20] Re21400, 7% blockage and 2% inlet turbulence (or label mismatch); mean C_d within 10% of 2.1, St [0.128,0.136], grid change <3%, stationarity and no unexplained numerical force band |
| 13 / new issue | Conservative block refinement; channel, advected vortex/acoustic pulse, then cylinder | Uniform equilibrium <=1e-12 CPU; closed-domain relative mass drift <=1e-10; channel velocity error <=1%; reflected acoustic amplitude <=1% of incident; vortex amplitude/phase within 2% of uniform-fine reference. Re-run slice 11 with <3% change |
| 14 / new issue | D3Q27/cumulant or WALE comparison, only if earlier gates warrant it | Same physical cases, grids, budgets and statistical gates as slices 8/10/11; demonstrate accuracy or cost improvement at equal error before promoting a default; CPU/GPU parity and oracle fixtures |
| 15a / new issue | One selected RANS transport model; TMR 2DZP flat plate [R24] and manufactured transport | Skin friction within 5% of Karman–Schoenherr for Re_theta 4000–13000; manufactured solution with expected refinement order and nonnegative model state. Add the Wieghardt–Tillmann experimental comparison before a capability release |
| 15b / new issue | Shielded hybrid switching; attached plate and Driver–Seegmiller backward-facing step [R25], Re_H about 36000, zero upper-wall angle | Attached limit within 2% of the validated RANS solution with no grid-induced separation; reattachment within 10% of the experimental value; use a 3D periodic span for the hybrid solve and verify span sensitivity <2% |
| 16a / new issue | Thermal wall flux; laminar heated channel and JAXA heated-channel DNS [R26], Re_tau 180 then 640, Pr=0.71 | Laminar solution within 1%; turbulent channel Nu and mean-temperature RMS error within 5% of DNS; applied/requested wall heat within 1% |
| 16b / new issue | Turbulent CHT; two-material slab then Cukurel–Arts–Selcan flat/ribbed slab cooling-channel experiment [R27] | Slab solution within 1%; interface heat imbalance <0.1% of imposed heat; area-mean Nu within the experiment's stated uncertainty plus a predeclared 5% numerical allowance; measured temperature profile checked separately |

Slices 1–2 are first because their controls are small, unambiguous and upstream
of interpreting external drag. Slice 4 is a small protective change that can
proceed independently when claimed. Slices 5–7 prevent false acceptance before
large wall/geometry experiments. Slices 8–16 require staged agreement and
published case definitions; the table does not authorize speculative large PRs.
For slices 15–16, transcribe the precise dataset, its uncertainty, operating
point and reference numbers in the new issue before implementing; the named
cases are capability proposals, not completed benchmark recipes.

## References and access scope

References describe algorithms or supply published results only. No source code,
input decks or artwork from other solvers was read or copied. Publisher abstracts,
author manuscripts and institutional records were checked for this review;
equation-level implementation will require the full applicable paper. In
particular, direct access to Jacob et al.'s publisher text was denied: the HRR
equation audit here uses FieldCraft's existing documented derivation and tests,
not a newly verified transcription of the entire article. M&S's accessible
publisher sections are not a full independent reproduction of its derivation.

* **R1.** Jacob, Malaspinas & Sagaut (2018), *A new hybrid recursive regularised
  Bhatnagar–Gross–Krook collision model for lattice Boltzmann method-based large
  eddy simulation*. DOI [10.1080/14685248.2018.1540879](https://doi.org/10.1080/14685248.2018.1540879).
* **R2.** Coreixas, Chopard & Latt (2019), *Comprehensive comparison of collision
  models in the lattice Boltzmann framework: Theoretical investigations*.
  DOI [10.1103/PhysRevE.100.033305](https://doi.org/10.1103/PhysRevE.100.033305);
  [author manuscript](https://arxiv.org/abs/1904.12948).
* **R3.** Spinelli, Gericke, Masilamani & Klimach (2024), *Key ingredients for
  wall-modeled LES with the Lattice Boltzmann method: Systematic comparison of
  collision schemes, SGS models, and wall functions on simulation accuracy and
  efficiency for turbulent channel flow*. DOI [10.3934/dcdss.2023212](https://www.aimsciences.org/article/doi/10.3934/dcdss.2023212).
* **R4.** Geier, Schönherr, Pasquali & Krafczyk (2015), *The cumulant lattice
  Boltzmann equation in three dimensions: Theory and validation*.
  DOI [10.1016/j.camwa.2015.05.001](https://doi.org/10.1016/j.camwa.2015.05.001).
* **R5.** Nicoud & Ducros (1999), *Subgrid-Scale Stress Modelling Based on the
  Square of the Velocity Gradient Tensor*.
  DOI [10.1023/A:1009995426001](https://doi.org/10.1023/A:1009995426001).
* **R6.** Vreman (2004), *An eddy-viscosity subgrid-scale model for turbulent
  shear flow: Algebraic theory and applications*.
  DOI [10.1063/1.1785131](https://doi.org/10.1063/1.1785131).
* **R7.** Malaspinas & Sagaut (2014), *Wall model for large-eddy simulation based
  on the lattice Boltzmann method*. DOI [10.1016/j.jcp.2014.06.020](https://www.sciencedirect.com/science/article/pii/S0021999114004276).
* **R8.** Han, Ooka & Kikumoto (2021), *A wall function approach in lattice
  Boltzmann method: algorithm and validation using turbulent channel flow*.
  DOI [10.1088/1873-7005/ac1782](https://doi.org/10.1088/1873-7005/ac1782);
  [author preprint](https://arxiv.org/abs/2009.04352) (not assumed identical to final text).
* **R9.** Asmuth, Janßen, Olivares-Espinosa & Ivanell (2021), *Wall-modeled
  lattice Boltzmann large-eddy simulation of neutral atmospheric boundary layers*.
  DOI [10.1063/5.0065701](https://doi.org/10.1063/5.0065701);
  [institutional manuscript](https://uu.diva-portal.org/smash/get/diva2%3A1640098/FULLTEXT01.pdf).
* **R10.** Kawai & Larsson (2012), *Wall-modeling in large eddy simulation:
  Length scales, grid resolution, and accuracy*.
  DOI [10.1063/1.3678331](https://doi.org/10.1063/1.3678331);
  [earlier author research brief](https://web.stanford.edu/group/ctr/ResBriefs/2010/04_larsson.pdf).
* **R11.** Zou & He (1997), *On pressure and velocity boundary conditions for
  the lattice Boltzmann BGK model*, DOI [10.1063/1.869307](https://doi.org/10.1063/1.869307);
  Hecht & Harting (2010), *Implementation of on-site velocity boundary conditions
  for D3Q19 lattice Boltzmann simulations*, DOI [10.1088/1742-5468/2010/01/P01018](https://doi.org/10.1088/1742-5468/2010/01/P01018).
* **R12.** Latt, Chopard, Malaspinas, Deville & Michler (2008), *Straight velocity
  boundaries in the lattice Boltzmann method*.
  DOI [10.1103/PhysRevE.77.056703](https://journals.aps.org/pre/abstract/10.1103/PhysRevE.77.056703).
* **R13.** Mei, Yu, Shyy & Luo (2002), *Force evaluation in the lattice Boltzmann
  method involving curved geometry*. DOI [10.1103/PhysRevE.65.041203](https://doi.org/10.1103/PhysRevE.65.041203).
* **R14.** Bouzidi, Firdaouss & Lallemand (2001), *Momentum transfer of a
  Boltzmann-lattice fluid with boundaries*.
  DOI [10.1063/1.1399290](https://doi.org/10.1063/1.1399290).
* **R15.** Lagrava, Malaspinas, Latt & Chopard (2012), *Advances in multi-domain
  lattice Boltzmann grid refinement*. DOI [10.1016/j.jcp.2012.03.015](https://doi.org/10.1016/j.jcp.2012.03.015).
* **R16.** Astoul, Wissocq, Boussuge, Sengissen & Sagaut (2020), *Analysis and
  reduction of spurious noise generated at grid refinement interfaces with the
  lattice Boltzmann method*. DOI [10.1016/j.jcp.2020.109645](https://doi.org/10.1016/j.jcp.2020.109645);
  [author manuscript](https://arxiv.org/abs/2004.11863).
* **R17.** Spalart, Deck, Shur, Squires, Strelets & Travin (2006), *A new version
  of detached-eddy simulation, resistant to ambiguous grid densities*.
  DOI [10.1007/s00162-006-0015-0](https://doi.org/10.1007/s00162-006-0015-0).
* **R18.** Kuwata & Suga (2021), *Wall-modeled large eddy simulation of turbulent
  heat transfer by the lattice Boltzmann method*.
  DOI [10.1016/j.jcp.2021.110186](https://www.sciencedirect.com/science/article/pii/S0021999121000814).
* **R19.** Kravchenko & Moin (2000), *Numerical studies of flow over a circular
  cylinder at Re_D=3900*. DOI [10.1063/1.870318](https://doi.org/10.1063/1.870318).
  Table II's compiled experimental C_d and St intervals are used; its numerical
  C_d=1.04 is a distinct reference, not a measurement from FieldCraft.
* **R20.** Lyn, Einav, Rodi & Park (1995), *A laser-Doppler velocimetry study of
  ensemble-averaged characteristics of the turbulent near wake of a square cylinder*.
  DOI [10.1017/S0022112095004435](https://doi.org/10.1017/S0022112095004435).
* **R21.** Lee & Moser (2015), *Direct numerical simulation of turbulent channel
  flow up to Re_tau approximately 5200*. DOI [10.1017/jfm.2015.268](https://doi.org/10.1017/jfm.2015.268).
  Keep the audit's interpolated Lee–Moser 950 profile distinct from the Torroja
  Re950 file (the del Álamo–Jiménez Re_tau 934 case).
* **R22.** Nathen, Gaudlitz, Krause & Adams (2018), *On the Stability and Accuracy
  of the BGK, MRT and RLB Boltzmann Schemes for the Simulation of Turbulent Flows*.
  DOI [10.4208/cicp.OA-2016-0229](https://doi.org/10.4208/cicp.OA-2016-0229).
  Its MRT failures motivate investigation; they do not prove the cause of
  FieldCraft's boundary-local failures.
* **R23.** Waters, Pascoe, Kirkpatrick & Thornber (2026), *On the impact of
  Large-Eddy simulation approaches on spectral bandwidth in cumulant Lattice
  Boltzmann methods*. DOI [10.1016/j.compfluid.2026.107246](https://doi.org/10.1016/j.compfluid.2026.107246).
  Publisher abstract/introduction and author-preprint metadata only; no solver
  source or implementation was inspected.
* **R24.** [Turbulence Modeling Resource, 2DZP](https://tmbwg.github.io/turbmodels/flatplate_val.html),
  theory comparison with Karman–Schoenherr and Coles; construct our own grid.
  Experimental companion: Wieghardt & Tillmann, *On the turbulent friction layer
  for rising pressure*, [NACA-TM-1314](https://ntrs.nasa.gov/citations/19930093916)
  (1951; report identifier, no DOI supplied).
* **R25.** Driver & Seegmiller (1985), *Features of a reattaching turbulent shear
  layer in divergent channel flow*. DOI [10.2514/3.8890](https://doi.org/10.2514/3.8890).
  [TMR case description and experimental data](https://tmbwg.github.io/turbmodels/backstep_val.html)
  clarify the corrected Re_H about 36000. No provided grid or solver files used.
* **R26.** Abe, Kawamura & Matsuo (2004), *Surface heat-flux fluctuations in a
  turbulent channel flow up to Re_tau=1020 with Pr=0.025 and 0.71*.
  DOI [10.1016/j.ijheatfluidflow.2004.02.010](https://doi.org/10.1016/j.ijheatfluidflow.2004.02.010);
  [JAXA case descriptions/data](https://jaxa-dns-database.jaxa.jp/channelflow.html).
* **R27.** Cukurel, Arts & Selcan (2012), *Conjugate Heat Transfer Characterization
  in Cooling Channels*. DOI [10.1007/s11630-012-0546-1](https://doi.org/10.1007/s11630-012-0546-1);
  [author paper](https://bcukurel.net.technion.ac.il/files/2015/01/Cukurel-2012-Conjugate-Heat-Transfer-Characterization-in-Cooling-Channels.pdf).

## Execution status

Design review complete; proposed acceptance numbers await maintainer steering.
#795 is upstream. Requests to take #819 and #820 were posted on their issues;
neither was labelled help wanted at review time. No re-architecture PR is open.
The implementation sequence and fresh reproduction results are in `PROGRESS.md`.
