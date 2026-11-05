# LES progress — 2026-11-05

Verdict: retain the LBM core; fix boundaries and validity before selected architecture changes.
Public `les-design` includes the slice 8 proposal below. The unpublished audit link is fixed.
Claims for #819, #820 and #794 are approved. Slices 5–7 belong to codex-les3;
slice 8 local prototype/proposal is authorized; a PR waits for #785 agreement.
#829 now exposes the stress gap. Never merge these PRs ourselves.

## PR status

- **#826 / #819:** ready, green CI `41893106606`, head `3c268c4c`.
  Uniform-pressure extrapolation repairs the original failure at step 1480:
  30,000 steps at roundoff; CPU 192-case sweep max departure 2.45e-14.
  GB10 both layouts, 64 cases at 30k: max 2.98e-7; inner-zone mismatch 5.59e-9.
  Channel/cylinder controls and oracle pass. Both review threads resolved.
- **#827 / #794:** ready, green CI `41893593637`, head `78c79791`.
  Refuses unstable halfway wall before allocation/backend/checkpoint callbacks.
  Historical 80-case sweep: 40 on-node cases decay, 40 halfway cases grow.
  Focused tests, oracle, and final GB10 wall/fan/porous controls pass.
- **#832 / #820:** ready, green CI `41926505284`, head
  `45d3ac6994529e56c114e3f12e791e1dc35660f1`; characterization `195ff7a6`.
  Branch `fix/820-inlet-slip`, worktree `/home/agent/work/fc-les2-820`.
  Based on latest upstream/develop `6b613217`, checked at 22:35 UTC.
  Issue results: https://github.com/tangency-labs/fieldcraft/issues/820#issuecomment-6643921336

## #820 implementation and evidence

TRT velocity inlet only: extrapolate non-equilibrium stress trace from two inner
cells; apply fourth-order Shapiro high-pass to the density extrapolation residual.
Preserves prescribed velocity and deviatoric stress. Bulk collision, SGS and
viscosity unchanged. Outward velocity faces, missing inner cells and free-surface
flows retain the original closure. CPU/GPU snapshot scratch before boundary writes.

CPU on sc-builder-2:
- Original 16³, tau=.505, U=.05, ±1e-6 seed: guard at 2838 -> 30,000 steps;
  final density 7.84e-9, velocity 2.27e-9, decreasing late windows.
- Broadband tau=.505: density 2.56e-8 / velocity 6.12e-9; tau=.518:
  7.26e-12 / 3.97e-12. Nine cases including all axes/signs pass.
- Eight controls pass, including original six and periodic broadband seeds.
- Pressure channel profile error 2.96e-4 -> 2.94e-4; original bounds retained.
- 289 linear modes with actual convective outlet decay; worst log modulus
  -3.33e-5/step. Diagnostic only, not a general stability proof.

NVIDIA GB10 at final head:
- Eight 30k cases across both layouts: final density <=1.79e-7, velocity <=9.39e-8.
- 96 axis/sign/slip/periodic/chunk comparisons: relative velocity difference
  <=4.55e-7, density difference <=7.61e-8.
- Channel, vortex/backflow, thermal and odd/even readback controls pass.
- Initial two-buffer run failed at density 1.61e-6. Aligned staged/applied density
  sums and single-precision formula; final rerun passes the original 1e-6 bound.

Sphere integration: original Re=100, D=12 cells, tau=.518, grid 192×96×96 now
completes 28,800 steps / 120 D/U (previous guard at 2736). Final density
[.997923, 1.003769], maximum Mach .09841, no invalid cells. Mean Cd=1.15780,
lift RMS=.01958 **fails** the roadmap's 1e-3 criterion. #792 and drag validation
remain open. Also unfixed: fully periodic tau=.5005, Smagorinsky C=.1 broadband
control grows to density peak 1.1e-3, final 6.4e-4; separate bulk limitation.

All local validation passes: normal lbm tests, nine heavy perturbation cases,
eight controls, memory/checkpoint/parallelism, clippy for both crates, fmt and
manual simplify review. Shader reflection uses at most eight storage buffers.
The oracle initially exposed free-surface drift; excluding moving interfaces
fixed it. Final oracle passes, fixtures and generated report unchanged.
Final shader rounding fix rechecked on builder: reflection/clippy/fmt pass.

## Compute released / next steps

GPU released at 22:36 UTC after about 12 minutes. `GPU-DONE` written, `GPU-READY`
removed, request archived as `GPU-REQUEST.done-820`. Own source/build/results
cleaned under flock; /work retains 998 MiB Rust cache. No active GPU or builder job.

CI 41926505284 is fully green at 45d3ac69: main gate 13m21s, heavy 20m28s,
oracle 4m36s, dependencies and review check pass.
Review response: https://github.com/tangency-labs/fieldcraft/pull/832#issuecomment-6644180092
Review suggested additional tau cases. Local CPU diagnostic at final head:
.507 checkerboard final density 1.1e-14 / velocity 2.71e-15 (before guard 12217);
.518 checkerboard 8.8e-15 / 4.51e-15. Both tails decay. .5005 with Smag C=.1
fails: final density 1.6e-3 / velocity 1.69e-2, growing tail. Log
`820-review-tau-sweep.log`. No source changes. PR comment asks jyviko to confirm
the narrower .505/.507/.518 scope before landing; full original tau sweep is
unmet. Fully periodic broadband growth proves a bulk limitation, but not that
all inlet-case growth is bulk-driven. Await owner response.
Keep all three PRs responsive to review. No code or test jobs remain active. All three PRs verified open, ready and
green at 22:56 UTC; no new #826/#827 review requests.
Await jyviko's scope confirmation before landing #832; do not merge ourselves.
Public design and PR body record green CI and the unresolved acceptance limits. No further GPU use without a fresh lease.

CPU work only on sc-builder-2 under `/home/agent/work/.sc-builder-2.lock`, via
`SC_POD=sc-builder-2 SC_WORKTREE=/home/agent/work/fc-les2-820 sc-remote run`.
Use `CARGO_TARGET_DIR=/work/fc-les2-820-target`, `CARGO_INCREMENTAL=0`,
`FIELDCRAFT_THREADS=8`. Commit as Christian Svensson <blue@cmd.nu>, ending
`Part of #N.` Fold fixes into their commit, push topic with force-with-lease.

Logs and PR bodies: `/tmp/fc-les2-review/`. Final GPU `gpu-820.log` has
`GPU_820_LEASE_EXIT=0`; initial failure `gpu-820-initial.log`; sphere data in
`sphere-820/`. CPU logs: `820-production-acceptance2.log`, `820-production-checks.log`,
`820-remaining-checks.log`, `820-production-final-checks.log`, `820-oracle-final.log`.
Final shader check: `820-gpu-rounding-checks.log`. Diagnostics stay untracked.

Rebase hazard if #826 lands first: it uses 20-float pressure records; #832 uses
16 with inlet residuals at 14/15. Reserve at least 22 words and move residuals
past the pressure fields; update both shader/native stride and memory accounting.
Rerun both boundary regressions. Do not mechanically resolve that conflict.

## Slice 8 proposal (2026-11-05)

Local worktree `/home/agent/work/fc-les2-wall`, branch
`prototype/786-stress-wall`, commit `ae7100016f39f72514a04b27b22a5b68359546ec`,
base upstream/develop `6b613217`. **Do not push this branch or open a wall-layer
PR until jyviko agrees on #785.** The public proposal is appended to LES-DESIGN.

Prototype: existing Han on-node traction with Musker sampled at an explicit
physical height, including interpolation. No first-fluid-cell velocity overwrite
or halfway correction. Physical gradient/viscosity coupling is still a proposal,
not implemented. New local test `crates/lbm/tests/prototype_wall_layer.rs` compares
existing Musker, existing Han and matched-height Musker. It checks N10/20/30,
TRT/RR/MRT/HRR, all axes/both normals, h/delta=.20 plus .25, rest/refusals,
and actual population momentum including populations in transit at wall nodes.

Results are mechanism evidence, not the original #786 turbulent reproduction:
periodic 8x8 transverse cells, nominal Re_tau 950, prescribed initial Musker
profile plus perturbations, Smag C=.08, HRR sigma=.99, lattice velocity scale 200.
- 32 short cases complete (200 steps). Across 24 matched-height cases, max
  requested/applied gap 2.442e-15, budget residual 3.169e-14 of momentum,
  mass drift 8.948e-14.
- N10 HRR requested/applied: old Musker .964417/5.321940; old Han
  .758035/.758035; prototype 1.015233/1.015233. Initialization transient,
  not steady C_f. Original Han already has exact traction; new property is
  preserving it with independently controlled sampling height/Musker.
- 48 extended cases (2,000 steps, both flow signs) complete: gap <=1.998e-15,
  independent budget <=3.706e-14 of momentum /4.227e-9 of applied impulse,
  mass drift <=2.853e-13. Rest/invalid-height controls pass.
- Longer TRT reaches final speed .225307 (Mach .390), outside the proposed
  envelope. HRR N10 mean stress changes from 1.015233 over 200 steps to
  .910227 over 2,000 steps. **Do not claim a validated wall layer or #786 fixed.**
- Matched-height h=.25 initial request .995225 shows the interpolation error
  relative to unit reference stress; the plane is not snapped to a cell.

Checks: manual simplification review; cargo fmt; cargo check --release -p
fieldcraft-lbm; cargo test --release -p fieldcraft-lbm (183 passed, 27 ignored);
cargo clippy --release -p fieldcraft-lbm --all-targets -- -D warnings. All pass.
No oracle/full gate, GPU, or complete turbulent channel acceptance was run for
this unpublished experiment. No production solver default was changed.

CPU runs used sc-builder-2 under its flock; own target
`/work/fc-les2-wall-target`, CARGO_INCREMENTAL=0, jobs 4, threads 2. Memory about
17–18 GB of 28 GiB; disk about 99/118 GiB. No active build/test/GPU job remains.
Permanent local logs: `/home/agent/work/fc-les2-wall-results/`, also under
`/tmp/fc-les2-review/`. Names: wall-prototype-{first,long,reversed,crate-tests,
clippy,check}.log. Initial harness corrections (before final runs): wall_cells
method name, timestep .1 dx/20 to obey tau>=.5001, report legacy Musker's mass
and budget violations instead of applying the new on-node invariant to it.

Reproduce on builder, always under its lock:

```sh
flock /home/agent/work/.sc-builder-2.lock env SC_POD=sc-builder-2 \
  SC_WORKTREE=/home/agent/work/fc-les2-wall \
  /home/agent/work/bin/sc-remote run bash -c '
    export CARGO_TARGET_DIR=/work/fc-les2-wall-target CARGO_INCREMENTAL=0
    export CARGO_BUILD_JOBS=4 FIELDCRAFT_THREADS=2
    cargo test --release -p fieldcraft-lbm --test prototype_wall_layer -- --include-ignored --nocapture'
```

For extended controls add WALL_ONLY=matched-musker WALL_STEPS=2000; add
WALL_SIGN=-1 for reversed flow. The short matrix uses the defaults (200 steps).
Proposal posted: https://github.com/tangency-labs/fieldcraft/issues/785#issuecomment-6651971536
Design pushed on public `les-design`. Await jyviko's direction before any
wall-layer PR; no reply has arrived at this checkpoint. Full acceptance remains Re_tau 950 then 1000/2000,
requested/applied <=1%, carried C_f within 5% selected DNS, profile/total-stress
checks, fixed physical h under refinement, fixed C_s/sigma, both GPU layouts.
#826/#827/#832 rechecked open, ready, green, with no new human review requests;
#832's narrower-scope confirmation remains pending. No GPU requested/used.
