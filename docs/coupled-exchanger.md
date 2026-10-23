# Two fluid circuits with one conducting wall

`file.sample {"name":"heat-exchanger"}` opens an original short counterflow model.
`solve.run {}` or `solve.exchanger {"name":"Counterflow","threads":4}` advances
both CPU fluids through **one wall state**. This is a first implementation toward
[#700](https://github.com/tangency-labs/Fieldcraft/issues/700), not the full refined
steady-state acceptance case in [the release contract](solvers/heat-exchanger.md).

The recording, still and solved project are preserved outside git for GitHub PR
attachment upload; that upload is pending. Reproduction recipes and numerical
reports remain in the repository. The recording and original model are FieldCraft
contributors' work (MIT OR Apache-2.0); captions use the attributed Inter font.
The 47.7-second H.264 clip captures the actual 1440 × 1000 app window with a
48-pixel caption strip added below it (10 fps). It shows defining the pair,
choosing CPU, live solve progress, the air transient, switching to coolant,
the common budget, and save/reopen. The original source design is temporarily
removed before reopening, so the last scene checks the embedded geometry and
saved pair rather than reading an external design or solving again. The short
sample's saved frames play faster than physical time.

Build the app/CLI; compile [the original separator recipe](../tools/demo-video/coupled-exchanger-plate.json)
with `cargo run --example compile_recipe -p fieldcraft-geometry --features scripts --
tools/demo-video/coupled-exchanger-plate.json coupled-exchanger-plate.solidcraft`.
Run the [command recipe](../tools/demo-video/coupled-exchanger.json) from that output
directory (or substitute an absolute geometry path), through the normal CLI or
control protocol. `geometry.open` registers the source before embedding it in the
project. UI captures use `ui.screenshot`, not an offscreen substitute for the window.

The [recorded command replies](media/features/coupled-exchanger-report.json) include
the solve, units and reopened pair. On Windows / Ryzen 9 7950X with four CPU threads,
the recorded dispatch-to-result interval was about 6.95 s, including screenshot
overhead and concurrent test activity; the RTX 4090 rendered the viewport. This is not a throughput benchmark.
The recording was built from original implementation commit
`098f2f7e08ec4d4629f659b50d8e7f042f845250`, before the unmerged history was
rebased to remove demonstration binaries. That cleanup changes no application
source or numerical inputs. Reproduction notes and reports retain the capture
provenance; demonstration binaries are delivered as PR attachments rather than git assets.

At 5 s, the mixed outlet temperatures are 48.18181 °C (air) and 46.49707 °C
(coolant). Over the last 0.5 s, coolant gives the wall 50.03858 mW, while the wall
gives air 3.67940 mW and stores 46.35919 mW. Fluid storage, external losses and
discrete open-boundary exchange remain separate. The combined residual in that
saved interval is about 9.0e−13 mW and interface residual −2.5e−11 mW: arithmetic
conservation evidence, **not** a discretization-accuracy or steady-state claim.

## Supported model

Create two ordinary thermal fluid steps, then pair them explicitly:

```json
{"command":"model.exchanger.create","params":{
  "name":"Counterflow","first":"AirFlow","second":"CoolantFlow",
  "wall":"Separator","initial_temperature":"35 °C"
}}
```

Both domains contain the same complete conducting body. Their fluid occupancy
must be disjoint, grids aligned with equal cubic spacing, and their time step,
length and output frame count identical. Fluid materials, momentum populations,
temperatures and lattice scales remain independent. The shared material, power and
limit must agree. Periodic directions have matching conditions/extents. The initial
wall temperature is independent of either fluid's initial temperature.

This first path supports one separator, CPU + CPU and bounded physical transients.
It refuses per-circuit steady rules, accelerated capacities, other conducting solids,
thermal contacts, mismatched wall occupancy, held/flux wall ends and GPU/mixed
backends. The separator ends are insulated; other fluid walls may exchange heat
and their loss is reported. `solve.fluid` refuses a step belonging to a coupled
system. `solve.run` advances each defined exchanger once, rather than solving its
circuits independently. Deleting a referenced step or removing its conducting
wall is refused. Renaming a step updates the pair.

`solve.exchanger {"estimate":true}` returns the combined preparation/frame bound
before either lattice is allocated. It includes temporary preparation copies,
coupling metadata/scratch and output histories. Once prepared, the second wall's
capacity/source arrays are discarded. Cell maps and mirrored output temperatures
do not create a second thermal owner. Both circuits' work counts and memory are
checked against common caps; cancellation runs at bounded intervals.

## Conservative step

The method extends the existing CHT link operation (Patankar, *Numerical Heat
Transfer and Fluid Flow*, 1980; Li, Mei and Klausner, *Boundary conditions for
thermal lattice Boltzmann equation method*, JCP 237:366, 2013). Each fluid–solid
link's received heat has the affine form `a - b theta_wall`. The two half-cell
conductivities determine its interface weight, as for ordinary CHT.

For circuit i, the energy scale is `E_i = rho_i cp_i dx³` joules per lattice
temperature. Convert both `a` and `b` by `E_i/E_owner`, including the circuits'
temperature-zero offset, and accumulate them into the single wall. Its physical
capacity and applied source are present once. Conduction is subcycled at the
existing explicit face limit, with all fluid links implicit in each cell balance.
The resulting mean wall temperature supplies **each circuit's own** incoming
population, with equal/opposite energy transfers. No outer coupling iteration,
manual flux handoff or hidden under-relaxation is used. The time step is common;
both existing fluid stability checks still apply.

## Results and budgets

Choose `AirFlow` or `CoolantFlow` in Results, or use the circuit buttons above
the shared-wall budget. Temperature/velocity/pressure slices, streamlines and probes
refer to that named circuit. Shared-wall temperatures appear in both views; its
surface triangles use the saved solid temperatures, averaged over their sampled
vertices. The first circuit owns its capacity and power. Temperatures and
budget quantities use the display units. The existing directional port integrals
use the immutable solve-time fluid properties.

The command reply exports the complete pair/budget as JSON. Generic per-cell CSV/VTU
export is still unsupported and keeps the existing refusal; field/budget CSV is
part of the remaining release workflow.

`results.exchanger {"name":"Counterflow","frame":10}` reads the saved pair.
It gives both circuits' directional port reports and a budget **averaged over
the interval since the previous saved frame** (zero at the initial frame):

- `into_wall[0/1]`: positive from that circuit into the common wall.
- `fluid_storage[0/1]`, `wall_storage`: actual stored energy changes per second.
- `wall_power`: applied wall source, counted once.
- `open[0/1]`: net scalar energy inserted by the discrete open-boundary reset.
- `external[0/1]`: loss through the circuit's other walls, positive out.
- `residual`: combined storage minus source/open exchange plus external losses.
- `interface_residual`: wall storage minus source and the two interface transfers.

The `open` term includes the boundary algorithm's advective and conductive effects;
it is **not** a separately resolved physical port-conduction measurement. The port
reports remain directional **advective** sensible heat relative to their stated
reference. Independent hot/cold port duties may differ during a transient because
of storage, port conduction and flow imbalance; the report does not hide them in
one fitted duty. Cell temperatures are output in f32; budget reductions use f64
scalar populations and the one wall state, not rounded output fields.

Both circuit results carry an identical coupled solve descriptor and common times.
The kind-6 package envelope keeps it alongside the unchanged fluid payload and
solve-time properties. Save/open retains both sides and budgets without solving;
incomplete/mismatched pairs are refused. Editing either circuit's step, a material,
or the system makes both results stale while preserving their original properties.
Legacy fluid packages remain readable. After deleting a system definition, solving
one released circuit independently retires its old coupled result group, so a
leftover partner cannot make the next save incomplete.

## Verification and remaining acceptance

Fast solver tests check closed different-fluid/wall equilibration against the
capacity-weighted analytical temperature, including different lattice temperature
zeros; round-off energy conservation; source counted once; open-boundary energy
accounting; and identical temperatures to an unsplit single-fluid CHT step.
Engine tests cover normal commands, two saved results, kind-6 roundtrip, refusal of
an incomplete pair, CPU thread determinism, frame-dependent budget queries and
stale properties. Existing single-domain suites remain part of the checks.

The public sample is a **5-second physical transient**, 0.25 mm spacing,
0.000625 s steps, 8000 steps, 11 frames, regularized collision. A 10 × 1 × 2 mm
steel separator lies between 10 × 2 × 2 mm passages. Air enters at 20 °C and
1 mm/s; coolant enters at the opposite end at 50 °C and 0.5 mm/s. The other
plates are no-slip and insulated, z is periodic. Constant prescribed properties:
air (1.2 kg/m³, 1.8e−5 Pa·s, 1000 J/(kg K), 0.024 W/(m K)); coolant
(1000 kg/m³, 0.001 Pa·s, 4200 J/(kg K), 0.6 W/(m K)); wall
(8000 kg/m³, 500 J/(kg K), 15 W/(m K)). Wall initially 35 °C, zero source.
These are benchmark inputs, not a material-library or experimental data claim.

The sample is laminar and deliberately small; it is not the release's longer
parallel-plate reference, a converged steady operating point, or an NTU validation.
Remaining #700 work includes the independent prescribed-conductance effectiveness
network, spatial/time-step refinement and interior pressure taps, joint steady
stopping and acceptance budgets, and the final release demonstration. Experimental
forced-convection validation (#623) remains separate.

The energy-accounting reference is [MIT Unified Engineering §18.5](https://web.mit.edu/16.unified/www/FALL/thermodynamics/notes/node131.html).
The contract cites the [EnergyPlus fluid-to-fluid effectiveness reference](https://energyplus.readthedocs.io/en/stable/guides/engineering-reference/16.8-heat-exchangers.html)
for the later independent network. No external solver code, input deck or geometry
was used.
