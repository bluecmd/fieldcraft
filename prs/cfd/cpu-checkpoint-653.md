---
{
  "schema": 1,
  "kind": "pr",
  "action": "publish",
  "request_id": "pr-653-contour-animation-agent-20261024-cpu",
  "agent": "contour-animation-agent",
  "branch": "cfd/cpu-checkpoint-653",
  "issue": 653,
  "head_sha": "c3867d6cd1647171a01a84cb30c5d68af30f2fb8",
  "base": "develop",
  "draft": false,
  "title": "CFD: checkpoint and resume complete CPU fluid state",
  "media": "pending"
}
---
## What and why

Refs #653. A stopped CPU fluid process can now continue from its latest completed checkpoint instead of repeating the entire solve. `solve.fluid` and its normal form accept an explicit CPU restart file, save interval and resume flag. Project display results remain separate from restart state.

The checkpoint retains both momentum/thermal population buffers, outlet and fan state, conducting-solid state, absolute output cursors, all retained result/history samples and partial steady-monitor windows. Topology is rebuilt and checked against a model/settings/geometry identity. Storage cadence can change on resume; physical settings and the fixed CPU thread count must match. CPU arithmetic and existing stopping rules are unchanged.

The versioned streamed format stores exact floating-point bits, validates shapes/finite values, includes an input identity and SHA-256, and refuses truncation or trailing bytes. Native IO uses a 64 KiB buffer and atomic replacement after flushing; a failed replacement preserves the previous checkpoint. It refuses unrelated model/CAD/output files. Automation read/write-root grants apply. The checked solver-state memory bound includes retained history/monitor workspace and restart IO without cloning a second complete flow.

Replies report `last_checkpoint`, `resumed_from`, cadence and the checked `checkpoint_solver_bytes` bound. Cancellation keeps the latest sound periodic save; failures after a save retain the original error with its last path/step. Completed checkpoints can return their saved result without advancing again. Three logical commits separate solver state, persistence, and command/form/documentation wiring.

## Demonstration

Native media from exact source `c3867d6cd1647171a01a84cb30c5d68af30f2fb8`: `cpu-checkpoint-653.mp4` (25 seconds), `cpu-restart-form.png`, and `cpu-restart-flow.png`. All three were transferred through ordinary-Git artifact commit `1959c0156ae59f1c1dd100ebb9cbe7d0920dbdda`; original/index byte counts and SHA-256 were verified, and the full video decoded successfully. Retrieval receipts and GitHub attachments are pending. No media belongs to the feature branch.

The clip shows the actual CPU restart form, a native cancellation after checkpoint step 20,018, changed storage cadence on resume, and the completed flow with saved physical frames. This is an edited sequence of held presented app frames and accelerated playback, not a real-time solve recording. Captions quote the actual returned summary retained with the original QA record; they do not represent a separate native report panel.

The original dry-flow case has 64 cells (8 x 8 x 1), CPU with 3 threads, dt 0.001 s, 1,000,003 steps and physical end 1000.003 s. The RTX 4090 Vulkan adapter renders the viewport; computation is CPU. The original SolidCraft CLI 0.14 box supplies display bounds only; obstacle/conducting-body selections are empty. No heat-exchanger accuracy, coupled two-fluid model or throughput claim is made. Captions use the repository's Inter font (SIL OFL 1.1). Original recipes, interrupted state, returned summaries, project files and recording scripts are retained externally.

<details>
<summary>Minimal original command reproduction</summary>

Use SI units and save the original project before solving. These commands need no CAD for the solver:

```json
[
  {"command":"model.units","params":{"system":"m-kg-s-N-Pa"}},
  {"command":"model.material.create","params":{"name":"Fluid","density":1,"fluid":{"type":"newtonian","viscosity":0.1}}},
  {"command":"model.fluid_domain.create","params":{"name":"Wind","material":"Fluid","min":[0,0,0],"max":[1,1,0.125],"spacing":0.125,"faces":{"x_min":{"type":"velocity","velocity":[0.1,0,0]},"x_max":{"type":"pressure","pressure":0},"z_min":{"type":"periodic"},"z_max":{"type":"periodic"}}}},
  {"command":"model.step.create","params":{"name":"Flow","procedure":{"type":"fluid","domain":"Wind","steps":1000003,"time_step":0.001,"frames":4}}},
  {"command":"file.save_as","params":{"path":"original.fieldcraft","results":false}},
  {"command":"solve.fluid","params":{"step":"Flow","backend":"cpu","threads":3,"checkpoint":"flow.restart","checkpoint_every":10009}}
]
```

Cancel after a periodic save, or stop the owning CLI process. Reopen `original.fieldcraft` in a fresh process, then:

```json
{"command":"solve.fluid","params":{"step":"Flow","backend":"cpu","threads":3,"checkpoint":"flow.restart","checkpoint_every":20011,"resume":true}}
```

Select the velocity Z slice to view saved frames. The exact interruption step depends on when cancellation is requested. The native form exposes the same parameters. See `docs/cpu-fluid-checkpoints.md` for confinement, identity and retention details.
</details>

## Validation

Tested source `c3867d6cd1647171a01a84cb30c5d68af30f2fb8`, based on current mirrored develop `7d9c027d71b2c1ea3395e2552e95f0455a7244db`. Windows, CPU thread count 3. Final tree equals the tested tree before logical commit folding.

- Combined touched-crate suite: engine 368 passed, IO 47 unit + 18 integration passed, LBM 81 passed; doc tests passed. Existing annotated heavy/benchmark ignores were not run (15 engine, 1 IO, 19 LBM). Exactly three existing Windows failures were excluded: `bug_report::tests::a_long_session_and_the_model_are_attachments` (home-path JSON escape), `cmd::file::tests::every_migrated_journal_replays_to_its_migrated_model` (imperial density last bit), `cmd::render::tests::labels_and_command_lines_read_well` (path separators).
- Eight new fast LBM regressions compare complete ordinary/resumed results and final archive bytes: both collision schemes, convective outlets, relaxed fan state, thermal buoyancy/conducting solid, partial and late steady windows, early steady completion, changed cadence, 10,003-step genuine advance-chunk splits, 169,999-step rounded-window history, unstable-step error fidelity, and hostile/truncated state. The tiny long-step controls keep runtime bounded; the eight tests take about 0.44 s together.
- Native IO regressions exercise successful replacement of an existing checkpoint, failed replacement retaining it, identity mismatch, corruption/truncation/trailing bytes, caps and unrelated-file refusal.
- All-target clippy with `-D warnings` for LBM/IO/engine, formatting and diff checks passed. Targeted wasm32 checks for those shared crates passed. Final native app and CLI build passed with exact build SHA; generated command documentation had no diff.
- Fresh-process CLI interruption/resume: original 64-cell case, 100,003 steps, stopped after checkpoint 1,009, then resumed with changed cadence. All 1,387,944 bytes of `results.bin` exactly match uninterrupted execution; SHA-256 `d4d05bd9410e620826b1e9350b38b8401313284e31870b09d8ec21260c033064`. Corrupted state was refused. Confined headless MCP returned a completed checkpoint without advancing and refused changed thread identity.
- Exact-head native app: actual cancellation retained step 20,018; resume completed 1,000,003 steps, CPU 64 fluid cells, no solids, physical-end success. Forms and unobscured saved flow frames were visually reviewed. No GPU solver was exercised for this CPU-only feature.

Remote CI for this new PR is pending publication. The full local workspace gate, oracle and annotated heavy suites were not run. An initial build encountered a transient IO doc-test API mismatch while implementation was still changing; the frozen combined suite subsequently passed. No existing renderer-race code or test was changed.

## Known limitations

This is the explicit-path native CPU-fluid slice of #653. GPU, explicit/nonlinear/transient-heat restart, browser persistence, automatic discovery/offers and wall-clock scheduling remain open. Existing fluid/conjugate-heat approximations are unchanged; this validates continuation fidelity rather than additional physical accuracy.

Restart compatibility is versioned, not a cross-version migration promise. Model/geometry/settings/caps/thread identity must match; save cadence alone may change. Each save rewrites all retained state and can be large. Atomic replacement temporarily needs disk space for both the old and new checkpoint. A hard process kill can leave an unpublished adjacent temporary file (observed in the fresh-process QA); no automatic discovery or cleanup is added. The valid published checkpoint remained usable. Use separate paths for simultaneous writers; cross-process writer locking is not included. Existing checkpoints are deliberately retained after completion. A cancellation or unstable partial advance can lose work since the latest sound save.

The reported bound covers solver state/workspace and IO, not exact whole-process memory; application, geometry preparation, allocator and OS overhead remain separate. Display project files do not contain the full restart state. The native demo is a deliberately tiny original dry-flow workflow, not a heat exchanger or performance benchmark.
