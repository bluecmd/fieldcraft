---
{
  "schema": 1,
  "kind": "pr",
  "action": "publish",
  "request_id": "pr-654-contour-animation-agent-20261024",
  "agent": "contour-animation-agent",
  "branch": "diagnostics/solver-errors-654",
  "issue": 654,
  "head_sha": "e7b00e713b42c93b574bf279b78d46356975baac",
  "base": "develop",
  "draft": false,
  "title": "Record ordinary solver stops and expose persistent headless report paths",
  "media": "pending"
}
---
## What and why

Refs #654. Returned work-cap, convergence/stability and backend errors now leave a small local `solver_error` record. The explicitly supported non-error stop cases are an unmet configured steady cap/divergence and an incomplete explicit transient. The original typed error remains its source; a supported returned stopped result gains `diagnostic_report`. The actual saved path is printed on stderr and included in the MCP error reply. A requested physical transient completing its end time, malformed command parameters and cancellation do not produce these records.

The record contains bounded command parameters, model size, observed step/progress, the error, and available backend setup context. Native Vulkan setup reports adapter/driver, relevant adapter and device limits, and explicitly scoped process counts. Metadata belongs to each command's `Progress`; overlapping sessions and later steps cannot inherit another solve's device. Cached-device reuse publishes fresh context too. `solve.run` records at each actual child boundary, preserving the child step instead of attributing a later step's time/device to an earlier cap.

Persistence is explicitly enabled by the desktop reporter, CLI/MCP hosts or a supplied report directory. Ordinary embedded/test sessions stay quiet, including when the engine is a dependency of another crate. Reports contain no copied model, replay, log or backtrace, and never trigger an app-crashed startup question. Home paths are redacted before JSON serialization; size/retention budgets and omission explanations are documented. If writing fails, the original result/error is preserved.

Headless workers can set `FIELDCRAFT_CRASH_DIR` to a persistent mounted folder before launch. Confined MCP roots still apply. A minimal ordinary CLI reproduction is:

```json
[
  {"command":"file.sample","params":{"name":"cantilever"}},
  {"command":"solve.static","params":{"step":"Load","max_dofs":1}}
]
```

Run it with `fieldcraft-cli run cap.json`; the deliberately lowered cap refuses the solve and prints the saved record path. `fieldcraft-cli crashes list --json` or `diagnostics.crash_reports` lists it.

## Demonstration

A real presented native-window clip, `solver-errors-654.mp4`, was captured from exact source e7b00e713b42c93b574bf279b78d46356975baac: 23 seconds, 1440 x 952, H.264 / 12 fps, 552269 bytes, SHA256 864557322be22adc78319ab27ef8e4ac4004318c1ffcf4b268ee4fbb44455012. Companion stills are `solver-record-path.png` and `solver-record-device.png`. All native pixels are original FieldCraft output; caption text uses the repository's SIL OFL 1.1 Inter font. The edit contains held frames and a cut to an actual presented scroll state, with no real-time playback claim.

The original diagnostic control is an 8 x 8 x 1 fluid box (64 cells), not a heat-exchanger accuracy benchmark. Its configured steady minimum of 20 s exceeds the deliberate 0.01 s cap. Two strict RTX 4090 / Vulkan stops update one local record to count 2; the second uses `device_reused`, with one managed compute-device creation and one managed instance. The record contains actual driver NVIDIA 616.56 and distinct adapter/device limits (device storage buffers 7, max buffer 4294967295 bytes, storage binding 2147483644 bytes). The native inspection asserts these values exist in JSON; the generic fixed-width reply panel can clip long limit keys when expanded, so the video does not claim to show every limit legibly.

The same model then completes a requested 0.01 s physical transient after removing the steady rule, without adding a diagnostic or incrementing the old record. The clip labels the five stored frames separately from solver iterations. An original SolidCraft CLI 0.14.0 box supplies display bounds only, avoiding the existing automatic empty-viewport gallery; the saved model and result were checked for 64 fluid cells, obstacles [] and conducting solids []. The CAD reference does not enter the fluid solve. Source code is unchanged by this presentation setup.

<details>
<summary>Reproduce the original native control</summary>

Generate `visual-domain-box.solidcraft` using SolidCraft CLI `run` and `--save` with this original millimetre recipe:

```json
{"commands":[{"command":"solid.box","params":{"corner":[0,0,0],"length":1000,"width":1000,"height":125,"body_name":"FluidExtentReference"}}]}
```

Run the following through the ordinary FieldCraft command path. Use strict hardware Vulkan (`FIELDCRAFT_GPU=hardware`, `FIELDCRAFT_REQUIRE_GPU=1`, `WGPU_BACKEND=vulkan`) and set `FIELDCRAFT_CRASH_DIR` before launching the native app/CLI. The optional reference file affects drawing only.

```json
[
 {"command":"file.new","params":{}},
 {"command":"geometry.open","params":{"path":"visual-domain-box.solidcraft"}},
 {"command":"model.units","params":{"system":"m-kg-s-N-Pa"}},
 {"command":"model.material.create","params":{"name":"Fluid","density":1,"fluid":{"type":"newtonian","viscosity":0.1}}},
 {"command":"model.fluid_domain.create","params":{"name":"Wind","material":"Fluid","min":[0,0,0],"max":[1,1,0.125],"spacing":0.125,"faces":{"x_min":{"type":"velocity","velocity":[0.1,0,0]},"x_max":{"type":"pressure","pressure":0},"z_min":{"type":"periodic"},"z_max":{"type":"periodic"}}}},
 {"command":"model.step.create","params":{"name":"Flow","procedure":{"type":"fluid","domain":"Wind","end_time":0.01,"time_step":0.001,"frames":4,"steady":{"window":1,"min_time":20}}}},
 {"command":"solve.fluid","params":{"step":"Flow","backend":"gpu"}},
 {"command":"solve.fluid","params":{"step":"Flow","backend":"gpu"}}
]
```

Inspect with `diagnostics.crash_reports` and `diagnostics.crash_report`. For the successful transient, `model.step.edit` uses the same procedure without `steady`, then `solve.fluid` runs once more. The native display uses a Z-normal velocity slice at the domain middle, top view and wireframe reference. All generated geometry, solved projects, raw reports, scripts and recordings remain outside feature history.

</details>

The three immutable final files were delivered through the separate ordinary-Git artifact inbox in commit a19b6203948a942649a21d5f6ea131da55d49238, under objects/diagnostics/solver-errors-654/e7b00e713b42c93b574bf279b78d46356975baac/. The staging helper verified actual index bytes, sizes and SHA256 values. Retrieval receipts and GitHub attachment URLs are still pending; received/attached status is not claimed yet. The maintainer scanner can attach this exact-head evidence after publication. No GitHub sign-in, LFS or raw-repository media link is used.

## Validation

Windows x86-64, strict NVIDIA RTX 4090 hardware / Vulkan (`FIELDCRAFT_GPU=hardware`, `FIELDCRAFT_REQUIRE_GPU=1`, `WGPU_BACKEND=vulkan`), three build/solver threads. Source is based on mirrored upstream `7d9c027d71b2c1ea3395e2552e95f0455a7244db`; no production solver numerics, shaders, backend selection or oracle cases changed.

- Combined touched-crate tests: CLI 7 passed; engine 374 unit tests and 2 external-crate process tests passed; GPU 11 passed; LBM GPU 3 unit and 22 hardware verification tests passed with no missing-device skips. Fifteen existing engine heavy tests and three throughput benchmarks remained ignored by annotation. Exactly the three known Windows baseline engine failures were excluded: `bug_report::tests::a_long_session_and_the_model_are_attachments`, `cmd::file::tests::every_migrated_journal_replays_to_its_migrated_model`, and `cmd::render::tests::labels_and_command_lines_read_well`.
- The first parallel MCP run hit the existing `screenshot_takes_a_camera_orientation` PNG-inequality race. Serial MCP rerun: 12 unit and 1 workflow tests passed. Model: 86 unit, 9 definition and 10 units tests passed. UI: 186 tests and 3 doc/compile-fail examples passed. The concurrency defect was independently reproduced earlier on the unchanged baseline and #710's old head; it is outside this diagnostics change and is not repaired by a serial pass.
- Regressions exercise real solver caps, steady-cap versus physical-end semantics, per-child multi-step records, malformed/cancelled exclusions, persistence failure preserving the original typed error, repeat counting, bounded/redacted JSON, startup prompt exclusion, reporter isolation and cached-device reporting. The dependency integration test verifies that an ordinary embedded Session writes no user report without host intent.
- Scoped all-target `cargo clippy -- -D warnings` and `cargo check` passed for all seven touched crates; targeted model/engine `wasm32-unknown-unknown` check passed. `cargo fmt --all --check` and `git diff --check` passed. Regenerating `docs/features.md` produced no content change. Full tests precede one equivalent default-construction lint cleanup folded into the engine commit; final native/wasm checks, clippy and runtime QA cover the final Rust tree.
- Separate real CLI/MCP process QA verified stderr and error-reply paths resolve to the same actual record, repeats update its count, and the folder contains only `report.json`. The exact-head app/CLI build and repeated process QA passed, with e7b00e713b asserted in the report version. A bounded ordinary nonlinear control (max_iterations 1, fixed minimum increment, unchanged tolerances) naturally returned `did not converge`; its original error and saved path were preserved, with the correct step/CPU identity. Native steady-cap, cached-context, cell/obstacle checks and normal physical-end assertions passed; parent read-only visual review passed the refreshed presented frames.

The full workspace gate, ignored/heavy suites, unrun crate doc tests, Linux/macOS and browser runtime were not run locally. No existing or new upstream CI result is claimed for this head; normal PR CI is pending.

## Known limitations

Warning-only arc-length truncation at `max_increments` still returns a successful summary, without a solver-error record. This change does not parse warning text or change that solver's stopping/return semantics.

The broader diagnostics issue remains open. Refusal before setup/allocation leaves unobserved device/progress fields null. Successful CPU fluid summaries identify CPU, but failed CPU-fluid execution currently has no backend identity observation; its actual backend remains unknown rather than inferred from a requested/selected choice. Native counters count successful LBM compute-device creations (including retired devices) and `gpu::select` instance creations, with explicit exclusions for viewport, meshing, external and driver-internal contexts. No device is opened merely to fill a report.

No checkpoint path is inferred from the open model; none of these solvers supplied one in this validation. Browser shared types compile, but native filesystem persistence/counters are unavailable there; existing browser renderer details remain separate. Actual device-loss/OOM faults were not induced, and unsupported historical native GL/Metal/DX12 paths are not advertised as new supported options. The pre-existing shared screenshot race remains a separate renderer follow-up. GitHub video/still attachments remain pending maintainer upload.
