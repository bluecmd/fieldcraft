# Buckling results

`solve.buckle` returns load factors of the specified linear preload and maximum-component
normalized mode shapes. A load factor is dimensionless. The shape's amplitude is arbitrary;
the deformed display magnifies it for inspection and animation swings it through both signs.
This is a linear stability eigenproblem, not a physical transient or a postbuckling solve.
The existing supported elements and preload assumptions are described in [fem.md](fem.md).

The solver stores these in `BucklingResult` and `BucklingMode`, and a solved step is
`StepResult::Buckling`. Natural frequency analysis continues to use `FrequencyResult` and
mass-normalized `Mode`: buckling has no frequency, modal mass or participation fields.
`results.modes` selects either kind. Buckling reports and XY plots use `load_factor`, and
VTK exports label the dimensionless value `load_factor` rather than a frequency/eigenvalue
with time units. Probes and contours retain normalized buckling shapes when display units
change; natural modes retain their mass normalization in those units.

## Original column recipe

Run from an empty directory of your choice:

```text
fieldcraft-cli run /path/to/Fieldcraft/tools/buckling-results/column.json
```

The reusable recipe constructs an original 80 x 4 x 4 mm steel column (E = 210 GPa,
Poisson ratio 0.3, density 7850 kg/m³), with 8 x 2 x 2 hex20 cells, a fixed root and
1 MPa compressive end pressure. It solves two buckling modes and three natural modes,
selects the first buckling mode, keeps it with `view.save`, and saves `buckling-column.fieldcraft` with both results
in the working directory. To repeat the same case interactively, issue the recipe commands
through the app command/control interface. Select `Buckle` with `results.modes`, choose
mode 1 or 2, and toggle `results.animate`. Select `Modes` to inspect frequencies instead.
The viewport can use native Vulkan; these implicit solves run on the CPU.

This small recipe demonstrates result types, selection and persistence, not a new verification
benchmark. Existing Euler/Engesser columns, shifted-spectrum and imperfect-column tests check
the unchanged buckling calculation against the cited elastic stability theory.

## Saved-file compatibility

The runtime type distinction does not change the `.fieldcraft` model or current `MCRSLT02`
result encoding. Kind 1 keeps its original frequency payload and optional buckling part at the IO
boundary. Current-encoding buckling records read as the dedicated result, and newly written records keep
the old field order and floating-point bits for readers of that encoding. The compatibility layer borrows
shape arrays when writing and moves them when reading. No new result-kind number is allocated.
Contradictory nonzero or nonfinite frequency-only values in a legacy buckling record are
refused rather than silently discarded. The Rust result API changes; consumers must handle
the dedicated buckling variant and its `load_factor` instead of frequency-only fields.
The upstream version policy is unchanged: an older `MCRSLT01` project opens its definition
without saved results and warns to solve again. This type change does not add cross-version
result migration.
