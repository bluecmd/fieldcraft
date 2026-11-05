# Conservative block refinement for FieldCraft LBM — slice 13

Design proposal, 2026-11-05. Prepared against upstream `develop` `6b613217`
and the pending LES diagnostic PRs #828, #829 and #830. This is a design proposal,
not an implemented or validated solver capability. It is ordered after the
wall-layer (slice 8) and curved-wall (slice 9) proposals. Implementation direction
awaits maintainer agreement on
[#785](https://github.com/tangency-labs/fieldcraft/issues/785).

## Recommendation and decisions to make

Start with a **static, cell-centred, binary block hierarchy**, adjacent levels
at a 2:1 spacing ratio and a 2:1 time-step ratio. Keep the existing uniform-grid
collision kernels as the reference. Use explicit population transfers at block
interfaces, with a local mass/momentum flux register and correction at each
level synchronization. First prove the unforced, isothermal CPU case; then add
forcing, walls, LES and GPU execution in that order. Adaptive regridding,
thermal coupling, free surfaces and moving geometry are separate capabilities.

The proposed interface combines published rescaling and filtering principles
with a FieldCraft-specific conservative correction. **That combination is a
candidate to test, not a result established by the cited papers.** In particular,
conservation does not establish acoustic accuracy or stability at low relaxation
time. Failure of the wave gates means changing the interface method, not relaxing
the gates or hiding the defect with stronger collision filtering.

Decisions requested after the wall-layer and curved-wall proposals:

1. Approve static 2:1 cell-centred blocks and acoustic time scaling as the first
   implementation scope, with CPU support preceding device support.
2. Approve a small comparison of conservative moment transfer plus flux correction
   against a volumetric explode/coalesce reference. Retain the candidate only if
   conservation, channel and wave gates all pass.
3. Require a refinement-free wall-model exchange region initially. Its physical
   matching height must follow the agreed slice 8 contract.
4. Keep GPU refinement unavailable for validated studies until the same mass gate
   is demonstrated. A different GPU conservation tolerance would require an
   explicit roadmap decision; this design does not make one.

No new collision default, wall law, SGS coefficient or acceptance threshold is
proposed here. The existing no-refinement solve must remain bit-identical.

## Evidence and method choices

Only papers and FieldCraft's own source were consulted; no restricted LBM package
source or commercial solver was used. The reference summaries below distinguish
published evidence from the implementation choices in the rest of this document.

| Source | Relevant evidence | Consequence for this proposal |
|---|---|---|
| Lagrava, Malaspinas, Latt & Chopard (2012), *Advances in multi-domain lattice Boltzmann grid refinement*, DOI [10.1016/j.jcp.2012.03.015](https://doi.org/10.1016/j.jcp.2012.03.015), [author manuscript](https://archive-ouverte.unige.ch/unige:34397), §§3–4 | Their cell-vertex method filters fine-to-coarse data and uses local cubic reconstruction toward the fine grid; lower-order interpolation introduces interface error. | Do not use point injection or naive linear population interpolation. Derive cell-centred operators explicitly; the paper's nodal coefficients are not our cell-average coefficients. |
| Astoul, Wissocq, Boussuge, Sengissen & Sagaut (2020), *Analysis and reduction of spurious noise generated at grid refinement interfaces with the lattice Boltzmann method*, DOI [10.1016/j.jcp.2020.109645](https://doi.org/10.1016/j.jcp.2020.109645), [manuscript](https://arxiv.org/abs/2004.11863), §§4–10 | Resolution changes can convert non-hydrodynamic modes into acoustic or shear disturbances. Collision damping matters as well as the interpolation scheme. | Test a moving background, both crossing directions and kinetic perturbations. A stationary acoustic pulse alone is insufficient. Do not equate removal of high-order moments with removal of every non-hydrodynamic eigenmode. |
| Rohde, Kandhai, Derksen & van den Akker (2006), *A generic, mass conservative local grid refinement technique for lattice-Boltzmann schemes*, DOI [10.1002/fld.1140](https://doi.org/10.1002/fld.1140), [author-hosted paper](https://sites.ualberta.ca/~jos/pbl/ijnmf51p439.pdf), §§2–3 | Volumetric propagation imposes mass conservation without their needing population interpolation/rescaling. Their tests also expose staggered interface profiles. | Use a volumetric transfer as an independent conservation comparison, not as a presumption of sufficient wave accuracy. The method proposed below is not a reproduction of Rohde's algorithm. |
| Berger & Colella (1989), *Local adaptive mesh refinement for shock hydrodynamics*, DOI [10.1016/0021-9991(89)90035-1](https://doi.org/10.1016/0021-9991(89)90035-1) | Coarse and fine updates need a conservative synchronization correction when their interface fluxes differ. | Adapt the flux-register principle to actual LBM population transfers; do not invent a continuum face flux inconsistent with streaming. |
| Astoul et al., *Lattice Boltzmann method for computational aeroacoustics on non-uniform meshes: a direct grid coupling approach*, [author manuscript](https://arxiv.org/abs/2004.14887), DOI [10.48550/arXiv.2004.14887](https://doi.org/10.48550/arXiv.2004.14887) | A direct cell-vertex coupling offers an alternative to overlapping transfers and is assessed with waves, a vortex and a cylinder. | A fallback to evaluate if our cell-centred candidate fails wave tests; it would require a separate spatial/geometry contract. The link here identifies the preprint, not the journal DOI. |

Two existing documentation statements need correction in any eventual implementation
PR: `docs/solvers/literature.md` C4.3 reverses the filtering direction; C4.2's
assumption that the hex mesher already provides the required binary hierarchy is
not true of the current `mesh::octree` implementation. That mesher uses a 27-tree
and 3-refinement conforming templates. Reuse predicates and surface queries, not
those templates or their refinement ratio. Those corrections belong with the
eventual implementation; this proposal changes no solver code.

## Spatial ownership and bounded hierarchy

Use Cartesian blocks with `B³` interior cells. Prototype `B=16`, comparing 8 and
32 before choosing a production default. One refined block has eight children,
each with the same cell count and half its parent's spacing. Balance **faces,
edges and corners** to at most one level difference, since diagonal streaming
and interpolation reach beyond face neighbours. Start with axis-aligned static
refinement boxes snapped outward to whole blocks and report the resulting domain.

Every physical cell volume has exactly one owning leaf. Parent cells covered by
children are prediction/restriction storage only. Halos, covered parents and
overlap records contribute neither mass, momentum, wall force, statistics nor
validity extrema. Integer block coordinates determine relationships; floating
point proximity must not decide whether cells share an interface. Surface
classification continues through the existing robust predicate path.

Keep stable `(level, integer block coordinates)` identities, sorted in a fixed
order, with a checked index table for neighbours and interface records. Morton
ordering may improve locality but must not become a source of overflow or an
implicit ownership rule. Reject duplicate blocks, holes, overlaps and inconsistent
periodic images before allocation. Refine periodic partners consistently.

For the initial implementation, propose a hard maximum of eight levels, counting
level zero, further restricted by existing cell/update/memory budgets. All sizes,
coordinate shifts, substep counts and byte estimates use checked arithmetic.
Balancing is a bounded queue over a bounded block set and fails with the limiting
region/count if it exceeds the budget. No silent dropping of requested refinement.

Start with two layers of halo cells for centred cubic reconstruction and gradients.
Build an explicit dependency plan for each stencil, including edge/corner data;
allocate any additional required interface band in the estimator. Require complete
stencils, enlarging refinement regions or refusing unsupported topology rather
than silently substituting one-sided low-order interpolation. Initially keep
coarse/fine interfaces away from walls and open boundaries. Same-level block
boundaries may meet them.

## Level units, populations and stress

Write `h_l = h_0 / 2^l`, `dt_l = dt_0 / 2^l`, `v0 = h_0/dt_0`, and use a
single density scale `rho0` throughout. Physical model values remain SI;
display conversion still belongs to `model::units`. Lattice rescaling is a
solver operation, not a second display-unit system.

For the same physical Newtonian fluid and velocity:

```text
u_lattice,l = u_SI / v0                       (independent of level)
rho_lattice,l = rho_SI / rho0                 (independent of level)
nu_lattice,l = nu_SI * dt_l / h_l²
tau_s,l = 1/2 + nu_lattice,l / cs²            cs² = 1/3 for D3Q19
tau_s,l+1 = 1/2 + 2 * (tau_s,l - 1/2)
a_lattice,l = a_SI * dt_l² / h_l              halves on refinement
S_lattice,l = S_SI * dt_l                     halves on refinement
```

These relations follow from dimensional consistency. They agree with the
unforced pre-collision rescaling in Astoul (2020), §4.1, eqs. 19–25. The shared
`v0` also means shared physical sound speed `v0/sqrt(3)`: a change of grid must
not introduce a change in acoustic impedance for an otherwise uniform fluid.
Refinement does not reduce Mach by itself. Check the stability bounds on every
level, especially the coarsest molecular relaxation time; a fine region does
not license an unstable coarse region. Do not tune `dt` independently per block.

The canonical transfer state is **pre-collision**, at a stated physical time.
It contains density, momentum and a force-corrected non-equilibrium stress.
Use different types or explicit stage tags for pre-collision, post-collision and
AA storage orientations; their scaling formulas are not interchangeable.

For an unforced shear mode with the same physical viscosity and strain:

```text
P = sum_q(c_q c_q f_q) - rho (cs² I + u u)
P_f = (tau_s,f / (2 tau_s,c)) P_c
P_c = (2 tau_s,c / tau_s,f) filtered(P_f)
T_visc,SI = rho0 * v0² * (1 - 1/(2 tau_s)) P
```

`T_visc,SI` denotes the non-equilibrium contribution to momentum flux (the
negative of the usual positive viscous stress convention). Substituting the
level relation for `tau_s` shows the physical shear flux is unchanged. Scaling
`P` by `(tau_f-1/2)/(tau_c-1/2)` instead would be wrong for pre-collision data.
Interpolate a common physical strain proxy, `P/(rho*tau_s*dt_l)`, and reconstruct
with destination density, relaxation time and time step. Apply bulk-mode scaling
with its own rate where it differs from shear. MRT/TRT ghost rates cannot all be
treated as viscosity; the transfer adapter must name the supported moment basis.

For the first candidate use a regularized hydrodynamic reconstruction of interface
ghosts: preserve density/momentum, reconstruct second-order stress, and use the
existing collision's stated recursive closure for higher moments where supported.
This is an explicit interface filter, not an unchanged bulk collision. Start
with one verified collision adapter; refuse others until their shear, bulk and
kinetic-mode tests pass. Do not silently zero stress or call one BGK scaling
formula universal support for MRT, TRT and HRR.

With Guo forcing, the current code uses
`j_h = sum(c_q f_q) + F_l/2`, where `F_l = rho*a_lattice,l`, and
`P = sum(c_q c_q f_q) - rho(cs² I + u u) + (u F_l + F_l u)/2`.
Preserve the transferred hydrodynamic momentum `j_h`, then reconstruct the
destination raw first moment as `j_h-F_destination/2`. Subtract the destination
half-force tensor from the desired corrected stress when constructing the raw
second moment. A direct copy of raw momentum or use of the source half-force
would create a level-dependent velocity jump. Verify these identities against
the existing `turbulence::flux` definition before enabling forced refinement.

## Restriction and prolongation

Treat the conserved state as cell averages for transfer and accounting. On a
uniform lattice, streaming is already a volume-preserving permutation. This
interpretation at a nonuniform interface is part of the new method and must be
tested; the old point-sampling assumptions do not prove its spatial order.

Restriction volume-averages the eight children for density and hydrodynamic
momentum. Obtain velocity by dividing restricted momentum by restricted density;
do not average velocities. Reconstruct destination populations at the destination
stage/rates. Filter and average the normalized stress/strain information before
destination rescaling. Child volume averaging provides a low-pass filter but is
not an ideal anti-alias filter; measure its transfer function and test whether an
additional fixed, conservative filter is required. Do not smooth conserved
moments across an ownership boundary without accounting for the transfer.

For prolongation, fit a tensor-product cubic to **coarse cell averages**, integrate
it over the eight child volumes, and enforce that their weighted conserved
moments equal the parent's. Derive and test those coefficients, including cubic
polynomial reproduction; do not reuse nodal interpolation weights. A fixed-order
roundoff residual correction may be distributed equally over siblings. Transfer
stress separately and reconstruct all ghost populations with the target moments.

In smooth tests the cubic operator is used without clipping. A later positivity
limiter may scale sibling deviations about their conservative mean, but must
preserve that mean and disclose activation. Signed populations are not themselves
invalid in FieldCraft. Non-finite values, non-positive density or a breached
velocity guard fail explicitly; setting negative populations to zero is forbidden.
Any limiter activation in a nominally smooth wave case invalidates an unqualified
high-order accuracy claim.

Restriction also removes resolved subcell motion. Record the tensor
`average(rho*u*u) - rho_coarse*u_coarse*u_coarse` as a subfilter-transfer diagnostic.
Do not inject it as an additional SGS stress without a separate derivation: that
could double-count modeled stress. Matching a shear strain and conserving total
momentum do not mean all second moments or turbulent kinetic energy are conserved.

## Subcycling and synchronization

Use integer ticks in units of the finest configured time step. A level `l` step
advances `2^(Lmax-l)` ticks; all root synchronization times have exact integer
representations. Refuse overflow and excessive total work before starting. Do
not add floating-point time increments independently on different levels.

The following schedule applies recursively, with depth bounded by the level cap:

1. At time `t`, fill same-level and periodic halos. Restrict synchronized children
   into covered parent cells, then fill the coarse stencil support. Decode any
   device storage into the canonical pre-collision interface state.
2. Save the needed coarse interface state at `t`. Advance the coarse predictor
   to `t+dt_c`, including covered parent prediction cells. Record interface
   transfers and source/wall impulses **only for owning coarse cells**. Keep the
   old interface state and predictor available while fine cells subcycle.
3. Advance fine cells from `t` to `t+dt_f`, then to `t+dt_c`. At each fine-step
   start, build fine ghosts at that time, in the fine pre-collision convention;
   collide those ghosts with fine parameters before they supply streamed values.
   Fill same-level halos at both starts, not just at the coarse-step start.
   Record fine interface transfers during both steps.
4. Restrict the completed fine state into covered coarse storage. Compare coarse
   and fine integrated interface transfers and apply the conservative correction
   to the adjacent owning coarse cells. Recompute their macroscopic/stress state.
   Fill corrected halos before those cells are read again.
5. Once the entire hierarchy reaches the same tick, finalize budgets, publish
   field frames/force intervals/statistics, and allow checkpoint writes. Retain
   the corrected coarse history for the next temporal interpolation.

```mermaid
flowchart LR
  A[All levels at t] --> B[Coarse predictor at t + dt]
  B --> C[Fine substep 1]
  C --> D[Fine substep 2]
  D --> E[Restrict and correct interface exchange]
  E --> F[All levels at t + dt: report or checkpoint]
```

For the fine half-time boundary, the proposed temporal interpolation uses the
common physical transfer variables at previous/current coarse times and the
coarse predictor. Quadratic interpolation gives weights `(-1/8, 3/4, 3/8)` at
`t+dt_c/2` on `(t-dt_c, t, t+dt_c)`. Test constant, linear and quadratic temporal
fields, and verify the coupled method's order independently of the interpolation
polynomial's order. At startup, with no previous state, use the two-endpoint
linear startup once and record it; verify its transient/order with analytic data.
Never manufacture a repeated previous frame and describe it as established
second-order startup. If the startup fails the wave/convergence gates, implement
a derived startup predictor before proceeding. Resume must restore history,
not restart this startup procedure.

This schedule needs a dedicated block stepper: today's `Backend::advance` hides
collision and streaming together. Repeated calls to independent `Flow` instances
with box boundary conditions would not supply the required internal interfaces.

## Conservation ledger and correction

At a synchronized time, sum over owning fluid cells only:

```text
M       = rho0 sum_i V_i sum_q f_iq
P_raw   = rho0 v0 sum_i V_i sum_q c_q f_iq
P_hydro = P_raw + K
K       = (rho0 v0 / 2) sum_i V_i F_l(i)
```

`V_i=h_l³`. In the forced case `K` is the level-dependent half-force endpoint
term; report it explicitly. For a root interval, the expected momentum change
is the sum of actual owning-cell collision/source impulses, minus wall impulse,
plus external boundary transport. The physical-momentum balance adds
`K_end-K_start`. Do not compare raw-population momentum against an unqualified
continuum `M*a*dt` when a force controller changes within that interval.
Store both the prescribed physical forcing and the realized collision impulse
so that a forcing error cannot be concealed by a self-consistent ledger.

For each interface tile in the unforced case, record the **signed change into owned cells** due to
all population transfers across that ownership interface. A streamed population
`f*` contributes `rho0*V*f*` to mass and `rho0*v0*V*c_q*f*` to momentum. Sum the
coarse-step transfer into coarse owners as `G_c`, and both fine-step transfers
into fine owners as `G_f`. These are four-component, time-integrated amounts,
not instantaneous face densities. They already contain their cell-volume and
substep multiplicities: do not multiply them by another face area or time step.

The synchronization defect is `D = G_c + G_f`. Correct adjacent owning coarse
cells by a total `-D`; the fine transfer is authoritative. With corrections
partitioned by fixed weights that sum to one, the total internal exchange is
`G_c + G_f - D = 0` in exact arithmetic. This gives a short, independently
checkable conservation proof without assuming that interpolation happens to
conserve a streaming flux.

A D3Q19 diagonal can cross more than one geometric face. Build disjoint
source/destination transfer records, with a canonical tile owner for edge/corner
crossings and periodic wrapping. Count the population change once, not once per
crossed face. At a planar interface, four fine face cells over two fine substeps
give the expected factor of eight relative to one coarse cell volume; that
identity is a test, not a general replacement for edge/corner bookkeeping.

For a recipient of volume `V`, turn its correction into lattice `delta_rho` and
`delta_j_raw`, then apply the moment projection

```text
delta_f_q = w_q [delta_rho + (c_q dot delta_j_raw)/cs²].
```

The D3Q19 quadrature gives exactly those zeroth and first moments algebraically.
If several tiles touch a cell, sum their corrections in a fixed order before
applying the projection. Record the achieved moments and the roundoff residual;
allow at most two local roundoff-correction passes, then fail if the stated
roundoff budget is exceeded. Never use an unbounded correction loop, global
density normalization, discarded negative populations, or an unapplied ledger
entry counted as fluid mass.

**Constant Guo-force extension:** the conserved interface quantity must become
hydrodynamic momentum, not raw momentum alone. At fixed prescribed acceleration,
a population change in an owning cell contributes momentum proportional to
`(c_q + a_l/2)*delta_f_q`. Use that contribution for both sides of the interface
register, with each owning level's own `a_l`; retain a separate raw register for
diagnostics. For a correction targeting `delta_j_h`, reconstruct with
`delta_j_raw = delta_j_h - a_l*delta_rho/2`. This accounts for the half-force
change caused by the mass correction itself. Otherwise raw reflux can close
while physical momentum drifts as mass crosses levels. Stage the prescribed force
consistently with collision and verify a uniform acceleration solution and the
prescribed physical body impulse independently of the register. Force-controller
updates add a separately recorded `delta K` event; variable/implicit forcing
remains unsupported until its time-centred source/transfer derivation passes the
same physical momentum test. Do not absorb a source error into interface flux.

This projection can excite pressure/stress modes. Record the maximum local
correction, accumulated absolute correction and signed correction per interface,
and inspect their spectra along with reflected waves. The mass gate is necessary
but not sufficient. An interface needing a large persistent correction is a
method defect to investigate even if its final global balance closes.

Evaluate global balances at every root synchronization in verification runs and
at bounded reporting intervals in production, with an online maximum that cannot
lose an earlier failure. Check every owning local step for invalid values. Do
not compare asynchronous coarse/fine states as if they represented one physical
instant; optional subcycle budgets must include the pending exchange register.

## LES filtering, walls and force histories

Keep the physical SGS filter width explicit: initially `Delta_l=h_l` for cubic
cells, using the same documented width convention as the uniform solver. Preserve
the chosen model and coefficient, then recompute local effective viscosity from
the destination fields and destination filter width. Do not enforce
`tau_eff,f=1/2+2(tau_eff,c-1/2)` when changing resolution also changes modeled
eddy viscosity. That simple relation applies to the same physical viscosity.
Store molecular, SGS and interface-filter effects separately in diagnostics.

HRR/Vreman gradients must use physical distances and the completed, correctly
timed halos. HRR stress reconstruction and dynamic sigma must not read stale
coarse gradients during the second fine substep. Start with a fixed collision
configuration established by slice 10; treat any interface-width blending or
sigma damping as a declared new model variant with its own comparison, never
an automatic repair for noise. Refuse unsupported collision/SGS combinations.

Walls retain one geometric identity and one force-group identity independent of
block/level. Voxelize the same physical surface at each level that needs it and
use slice 9's verified link fractions when available. Only owning wall links
apply a boundary impulse; covered coarse wall links must not duplicate it.
Recompute wall distance and `y+` with the owning level's spacing and the same
physical viscosity. Refinement must not silently move a halfway/on-node wall.

For initial WMLES, require the complete wall exchange region, matching point and
its interpolation/gradient stencil to lie in a band at one resolution. The
matching height `h_match` is physical and independently specified by the agreed
slice 8 design, not "the second cell" on whichever level happens to own the wall.
Topology setup expands that band by the actual stencil reach. Keep the first
coarse/fine interface outside it and sweep interface distance from the band.
Cross-level wall sampling is a later extension requiring synchronized physical
interpolation and a separate stress-budget test.

Integrate requested, applied and carried wall impulses over each root interval
in physical units. Integrate prescribed traction over disjoint owning surface
patches and local time intervals, not by averaging wall-node stresses. Keep the
physical reference area used in drag coefficients fixed across levels; refining
a patch must not multiply that area or duplicate its diagonal-link impulses.
For a local lattice wall impulse `J_l`, the physical impulse
is `rho0*h_l³*v0*J_l`; divide the sum over all owning links/substeps by `dt_0`
to obtain the interval-average force. Sum impulses before forming stress ratios.
The existing `Backend::forces` means "last local step" and cannot represent this
aggregate without a new explicit contract. Keep the #829 closed-domain momentum
account and expose the interface correction separately from wall force.

#828 statistics must consume equally spaced **physical-time** samples. Supply one
force sample per common reporting interval, containing every fine substep's
impulse. Do not give fine blocks more statistical weight because they stepped
more often. Retain at least 20 complete blocks and the existing interval, drift,
lag and doubled-length gates. A capped or unsupported refined run is inconclusive
or refused, never physically validated.

## GPU layout, execution and precision

The hierarchy description and interface plan live in `fieldcraft-lbm`; all wgpu
objects and kernels remain in `fieldcraft-lbm-gpu`. Store fixed-size blocks in
level batches, with direction-major population arrays and contiguous cell
indices inside each block. Keep flags, block descriptors, transfer stencils and
interface registers in bounded packed buffers with checked offsets. Bind-group
layouts are kernel-specific and checked against adapter limits; do not require
one binding per block or exceed the existing portable storage-buffer budget.

Implement the two-buffer A–B path first. Each phase has one writer per output
cell or transfer record: interiors, same-level halo exchange, interface ghost
reconstruction, boundary/gradient passes, collision/streaming, and synchronization
correction. Dependencies are separate dispatches with explicit command ordering.
Do not assume a workgroup barrier synchronizes different blocks. Reduce flux
records in a fixed tree/order; no floating-point atomic accumulation.

The existing AA in-place path needs a separate parity adapter. Two fine steps
return fine storage to its original parity while one coarse step flips coarse
parity. Read/write canonical directions using **each level's own parity**.
Interface staging must hold canonical populations/moments, not raw AA slots.
Test all coarse/fine parity combinations, both streaming layouts and repeated
root steps before enabling AA refinement. Saving memory is not a reason to
skip that comparison.

For a full leaf block with `g` halo layers, two population buffers alone cost
`2*19*scalar_bytes*(B+2g)^3`. With `B=16,g=2`, that is 2,432,000 bytes for f64
or 1,216,000 bytes for f32, before flags, gradients, histories and interfaces.
For a complete eight-child tree, retained full parent blocks add fewer than
one parent per seven leaves; include their actual halos and history allocation
rather than assuming this ratio is the complete overhead. Store old temporal
data only for required interface bands once correctness is established.

The planner must bound the actual sum of leaf and shadow updates per root step,
`sum_l 2^l * N_updated,l`, plus transfer/filter work and output memory. Account
for source/destination buffers, checkpoint staging, coarse predictions, temporal
history, reductions, wall data and the device's individual buffer caps. Reject
an oversized plan before upload. Benchmark updates per second, physical time
advanced per second, total device bytes and error against a uniform-fine run;
no speedup is assumed from cell count alone.

For planning only, if a fraction `f` of physical volume needs one finer level,
the ideal update cost relative to a uniformly fine run is
`f + (1-f)/16`, and its ideal population count ratio is `f + (1-f)/8`.
At `f=0.1` these are 0.15625 and 0.2125. Those are arithmetic upper bounds on
savings before halos, shadow predictions, transfers, synchronization and wall
work; they are not benchmark results. Low coarse-level molecular `tau` can also
limit the useful coarsening, even when the finely resolved wall region is stable.

GPU f32 populations and deterministic reductions do **not** by themselves prove
relative mass drift below `1e-10`. Retain deviation-from-equilibrium storage where
it improves rounding, use compensated/local registers where justified, and audit
the actual stored fluid state in host f64 during validation. A more accurate
ledger sum cannot repair drift already present in the populations. If the mass
gate fails, investigate accumulation and conservative representable corrections;
keep GPU refinement experimental/unavailable for validated studies until the
gate is met or the maintainer explicitly revises its scope. Native f64 is not
assumed in portable WGSL.

The #830 synthetic CPU/GPU diagnostic tolerances (`1e-12`/`1e-5` absolute) remain
separate from long-run conservation. Passing parity within `1e-5` does not waive
the `1e-10` mass requirement. Run hardware parity on an allocated real device as
well as CI's software adapter when implementation reaches device code.

## #830 validity across levels

Extend the report around **physical times and leaf ownership**, not a flattened
uniform lattice. Each recorded extremum needs level, block/cell identity and
physical time (or integer tick) as provenance. Keep three distinct views:

* A synchronized full-domain snapshot, over owning fluid cells at one time.
* Per-level extrema over an explicitly stated physical reporting window, reduced
  at local substeps so a fine-step excursion cannot disappear before root sync.
* An aggregate window envelope, the min/max of those level envelopes, explicitly
  described as a space-time envelope rather than a simultaneous field snapshot.

Do not average extrema or weight them by cell volume. Density references must
be the same physical `rho_ref(x,t)` on both sides; runtime may retain the material
density reference with its present limitation disclosed. Under acoustic scaling,
Mach uses the same physical sound speed everywhere. Include physical inlet/outlet
cells where supported; exclude solids, covered parents and all ghost cells.
Record interface-band maxima separately to localize transfer defects.

The existing report caps stored samples at 258. Keep a fixed global window count
and bounded online accumulators rather than allocating a history per block or
storing every fine step. Preflight the added `O(levels * windows)` records and
interface summaries in the memory estimator; do not silently retain the old
14,448-byte bound after extending the schema. State the actual sampling cadence
per level. If device reductions are sampled rather than every substep, label
the result as sampled and preserve the separate per-step blow-up guards.

Forces need both common-interval indicators and per-level/local-step indicators,
because root averaging can hide alternating fine-step spikes. Keep force groups
separate, use the norm of the signed mean as #830 does, and report undefined
ratios when that mean is zero. Different-time samples cannot be summed into a
purported instantaneous body force. Report the common interval's duration and
the unresolved finer cadence explicitly.

Any invalid local state marks the corresponding window invalid and preserves the
normal solve error. Keep `execution` separate from `validation: not_assessed`.
The proposed density-departure `0.02` and Mach `0.2` envelope remains diagnostic;
it does not certify interface accuracy, physical validation or refinement quality.

## Checkpoint and restart contract

Initially save **only at a root synchronization boundary**, after restriction,
flux correction and validity-window updates. A request during a subcycle is
deferred to the next such boundary with the actual saved tick reported. Never
write a mixture of coarse time `t+dt_c` and fine time `t+dt_f` under one timestamp.
The checkpoint must contain:

* A new versioned hierarchy format, settings/geometry identity, integer origins,
  spacings/time scales, level/block topology, ownership and transfer-plan version.
* Canonical pre-collision owning populations, source/controller state, collision/
  SGS/wall-model state, level step counters and streaming orientation where needed.
* Previous coarse temporal interpolation state and startup status. Rebuild covered
  parent current states and deterministic halos in a documented order, or store
  them if rebuilding cannot be shown bit-identical.
* Integrated wall/body budgets, statistics blocks, validity accumulators, output
  cursors and all diagnostic maxima that have not yet been published.
* Confirmation that flux registers are empty after synchronization. A nonempty
  register is a corrupt or unsupported midcycle state, not something to discard.

The new layout cannot masquerade as #830's uniform FCRST5. Preserve the existing
uniform checkpoint path and explicitly refuse cross-layout restoration. Validate
version, topology, sizes, finite values, ticks and all references against memory/
work caps before mutating a live solve. Keep signed finite populations legal.
Use the existing IO confinement and atomic-publication boundary; no filesystem
dependency belongs in the solver hierarchy.

CPU resume must be bit-identical at root synchronization across supported thread
counts. GPU checkpoint support is currently absent; first refined CPU support
must refuse GPU restart explicitly. A later device format must record adapter/
arithmetic compatibility and preserve canonical state, with device-specific
determinism and stated comparison tolerances rather than a cross-device bitwise
promise. Never convert f64 CPU populations to f32 and call it the same restart.

## Implementation boundaries and unsupported combinations

Suggested internal boundaries, subject to the eventual issue review:

| Location | Responsibility |
|---|---|
| `model` | Serializable physical refinement regions/levels and validated options; no execution buffers. |
| `lbm::hierarchy` (new) | Bounded immutable block topology, ownership, integer-time schedule and interface plan. |
| `lbm::transfer` (new) | Conservative spatial/temporal operators and collision-specific moment/stage conversion. |
| `lbm::refined_flow` (new) | CPU level stepping, flux registers, synchronization, budgets and checkpoints. |
| Existing `lbm` setup/memory/reporting | Level-wise preflight, explicit capability refusal, owned-cell validity/statistics. |
| `lbm-gpu` | Device storage, ordered dispatches and parity adapters built from the CPU plan. |
| `render` / result data | Block fields with ownership and physical coordinates; avoid expanding sparse blocks into a finest-grid box. |
| `engine` | Model/solve commands, schema/docs, save/reopen, query/probe behavior and hostile-input tests. |

Keep `voxel::Grid` useful as a block-local descriptor; do not pretend a hierarchy
has one global spacing or one flat index formula. `Backend::fields`, force
histories, `FluidResult`, probes and frame storage currently assume a uniform
layout and need an explicit block-aware representation. Queries must choose the
owning leaf, and visual interpolation must not double-count overlap. User-visible
configuration and reports go through commands; this document does not introduce
an unreviewed command name or public schema.

The first capability matrix permits static isothermal single-phase Newtonian
flow, periodic/closed domains, one transfer-compatible collision and no wall
model. Add constant forcing and verified plain walls after budget tests. Add
open boundaries only with a refinement-free boundary band and the #819/#820
regressions retained. Add LES and the agreed wall model after channel tests.
Refuse thermal/CHT, free surface, melting, non-Newtonian viscosity, fan/porous
zones, FSI and moving/refining topology until each has a derived source/flux/
state-transfer contract. A generic block container is not support for those
physics. Dynamic adaptivity later needs conservative regridding, hysteresis,
reproducible decisions and restartable topology transitions of its own.

## Validation ladder and fixed acceptance numbers

The numerical gates below carry forward slice 13 of the
[LES roadmap](https://github.com/bluecmd/fieldcraft/blob/les-design/LES-DESIGN.md). They are
requirements, **not measured results**. Define the normalization and sampling
window in each case before running it. Use original analytic geometry and
initial fields; compare with FieldCraft uniform-grid controls and cited published
reference values. Freeze fixtures only after reference acceptance is achieved.

| Stage | Cases and measurement | Required gate |
|---|---|---|
| Transfer algebra | Constant/cubic spatial fields, constant/linear/quadratic temporal fields; both directions; shear, bulk and force-corrected moment identities; diagonal and corner ownership | Exact identities analytically; CPU bounded normalized moment errors `<=1e-12`. No multiply-owned volume or unaccounted transfer. |
| Uniform equilibrium | Periodic stationary and translating fields; a closed stationary box; interfaces normal to x/y/z plus edges/corners, two and three levels | Maximum density and velocity departures `<=1e-12` on CPU, over every measured owning cell/time. |
| Closed-domain conservation | Periodic translating density perturbation and vortex; closed wall box; then constant acceleration and wall impulse tests | Maximum over synchronized times of `abs(M(t)-M(0))/M(0) <=1e-10`, measured from actual populations. Also check momentum against source/wall/endpoint terms, using the slice 7 scaled residual `<=1e-12` CPU with its documented roundoff floor. |
| Channel | Analytic laminar Poiseuille, coarse/fine interface parallel and normal to the flow, then fine wall bands; matched uniform-fine reference | Volume-weighted velocity RMS error divided by analytic maximum speed `<=1%`; report maximum/profile and interface-local errors separately. Preserve the mass and momentum gates. |
| Acoustic pulse | Both directions through planar and stepped interfaces; quiescent and translating backgrounds; normal and oblique incidence | Reflected acoustic amplitude `<=1%` of incident amplitude, with the measurement below; no growing unexplained interface mode. |
| Advected vortex | A smooth vortex crossing each direction and an oblique/stepped patch; resolved and deliberately under-resolved controls | Vortex amplitude and phase errors within `2%` of uniform-fine reference, with definitions below; preserve conservation and validity. |
| LES channel | Same physical case and wall exchange height as validated slice 8, fixed coefficients, move the refinement interface | Retain slice 8 drag, stress and profile gates, and slice 6 statistical acceptance. Laminar-channel success alone is not WMLES validation. |
| Cylinder, last | Re-run the validated slice 11 Re3900 case with identical geometry, inlet, domain/span, Mach, wall treatment and statistics | Refinement changes accepted mean drag and Strouhal by `<3%` from the uniform reference, while retaining slice 11's absolute bands and grid/domain/Mach/statistical gates. |

Use a fast equilibrium/transfer case in the normal gate and a heavy 30,000-root-
step conservation case. Initially sweep coarse molecular `tau` values
`{0.505, 0.518, 0.55}` where the selected collision supports them, with fine
values determined by scaling, not chosen independently. Any lower-tau extension
requires the slice 10 operating envelope and applicable SGS setup. Test at least
three resolutions at fixed physical geometry, refinement regions and final
physical time. Report observed order; the target is second-order smooth-field
convergence, not an order inferred from two favourable points.

For the first channel recipe, use a periodic streamwise/spanwise box of lengths
`4H` and `H`, stationary walls separated by `H`, and
`u(y)=a*y*(H-y)/(2*nu)`. Fix `Umax*H/nu=50` and `Umax/v0=0.02`, with `a` derived
from the analytic profile. Initialize compatible stress to shorten startup but
still bound the run and verify a time-converged profile. The initial interface
normal to the flow belongs to an embedded refined patch that does not touch the
walls; a separate fine-wall-band case tests parallel interfaces. An interface
meeting a physical wall needs its own stencil/boundary test before it is allowed.

The equilibrium departures are absolute dimensionless errors in `rho/rho0` and
each component of `u/v0` relative to the prescribed constant state. For mass,
retain both global drift and interface-exchange residuals. If a uniform-grid
curved-wall baseline already misses the stricter slice 13 closed-domain mass
gate, that combination cannot be certified for refined use merely because the
interface residual is small. The boundary defect and refinement defect must be
identified separately, with no silent substitution of slice 9's `1e-8` gate.

For acoustics, start with a small smooth pulse of density amplitudes `1e-5` and
`1e-4`, backgrounds `u_lattice=0` and `0.05` along the crossing direction, and
resolved wavelength scales of 16, 32 and 64 coarse cells. Define the pulse's
spectrum so its resolved band is explicit. Use a long periodic domain and stop
before wraparound contaminates probes. Separate outgoing/incoming characteristic
signals using `p' +/- rho0*c_sound*u'_normal`; remove the matching uniform-grid
baseline at the same physical probes/times. Divide the peak reflected excess by
the measured incident peak, and also report the unsubtracted reflection. Run
coarse-to-fine and fine-to-coarse cases with the same pulse, and repeat for oblique
incidence and interfaces with block corners. Keep interface location and probe
windows fixed before looking at the results.

For the vortex, initialize density consistently with the athermal pressure law
and include compatible non-equilibrium stress; quantify initialization noise in
the uniform control before attributing it to refinement. Wissocq, Boussuge &
Sagaut (2020), *Consistent vortex initialization for the athermal lattice
Boltzmann method*, DOI [10.1103/PhysRevE.101.043306](https://doi.org/10.1103/PhysRevE.101.043306),
is the initialization reference. Compare at matched physical times after each
crossing. Define amplitude as peak vorticity and phase from the same selected
Fourier mode in both runs: `abs(A/A_ref-1)<=0.02` and wrapped
`abs(phi-phi_ref)/(2*pi)<=0.02`. Also report circulation, vortex-centre displacement
in core-radius units, energy and emitted pressure; phase averaging must not hide
a split or distorted vortex. Fix the Fourier mode and reference window before
the run. A uniform-fine run must first have demonstrated its own resolution.

Explicitly separate three error mechanisms in the wave/vortex studies:

1. Bulk dispersion/dissipation, measured on uniform coarse and uniform fine grids.
2. Interpolation/filter/reflux error, measured by interface-location sweeps at
   fixed physical fields and by recording correction spectra.
3. Kinetic-mode conversion, tested with a small seeded non-hydrodynamic
   perturbation on the moving background as well as an ordinary hydrodynamic
   pulse. Classify modes using the actual collision's linearized D3Q19 operator;
   a high-order Hermite residual is only a diagnostic, not an eigenmode proof.

Record even/odd-step density/divergence/vorticity, pressure spectra and stress
near the interface at local-step cadence. Otherwise a mode near the fine-step
Nyquist frequency can disappear in root-only output. Compare the selected
collision/filter against controlled alternatives at fixed physical viscosity,
Mach, grid and SGS parameters. No per-grid coefficient tuning. Deliberately
under-resolved waves are rejection controls, not points to omit from the report.

Additional engineering gates: no-refinement equivalence with existing solves;
bit-identical CPU runs at 1/2/8 threads; interrupted/resumed equivalence; exhaustive
same-level/coarse-fine face and diagonal mappings; malformed hierarchy/checkpoint
refusals; memory estimates against allocation; deterministic GPU reductions;
CPU/device parity for fields, integrated impulses and both streaming layouts.
Include short real-device checks when device support is implemented. Long sweeps
remain heavy tests; no multi-hour study is needed to review this design.

## Work packages and present status

After direction is agreed and an issue can be claimed, split implementation into
independently reviewable steps:

1. Bounded hierarchy, ownership and transfer algebra. Keep refined solving
   explicitly unsupported while this infrastructure lands.
2. Two-level unforced CPU periodic/closed stepper and conservation proof/tests,
   including the independent volumetric comparison. No wall model or GPU claim.
3. Temporal/startup accuracy, laminar forcing/walls, channel and acoustic/vortex
   acceptance. Select or reject the proposed interface method from these results.
4. Block-aware command/results/checkpoint integration and hostile-input coverage;
   then compatible LES and the agreed wall exchange layer.
5. Device A–B path, strict conservation investigation, then AA parity. Expose a
   device capability only for combinations whose full gates pass.
6. Matched channel/cylinder validation and cost/error comparisons. Dynamic
   regridding and additional physics remain separate follow-ups.

Do not put refinement into the three existing diagnostic PRs. Their reports and
statistics are dependencies, not evidence that the refined solver already works.
Slice 11 and the wall-layer acceptance are not yet established by this proposal;
the later validation packages wait for those baselines.

Preliminary algebra evidence on 2026-11-05: a small Rust operator probe passed
four tests for force-corrected density/velocity and physical shear-stress transfer, the half-force
term in a mass/momentum correction, cancellation of synthetic two-level integrated
exchange, and eight-child momentum-weighted restriction. The existing D3Q19
quadrature-identity test also passed (five tests total). The assertions use
absolute bounds of `1e-14` for reconstructed moments/stress and `1e-16` for the
small correction examples. A deliberately omitted half-force correction produces
an error above `1e-6` in the rejection example. This CPU probe does not implement
interface geometry, streaming, subcycling, interpolation, waves or a refined
flow solve.

This draft specifies a candidate and its rejection criteria. No two-level flow
prototype, mass-drift measurement, wave reflection measurement or speedup has
been claimed. The main unresolved implementation risks are the conservative
correction's wave emission, temporal startup accuracy, collision/forcing transfer
compatibility and actual f32 mass drift. The first small CPU implementation is
intended to settle those questions before any long channel/cylinder campaign.
