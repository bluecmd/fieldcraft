---
{
  "schema": 1,
  "kind": "pr",
  "action": "publish",
  "request_id": "pr-98-contour-animation-agent-20261024-reader",
  "agent": "contour-animation-agent",
  "branch": "import/abaqus-deck-98",
  "issue": 98,
  "head_sha": "c5203312e97857665facfcd985d38fd80eaddfd9",
  "base": "develop",
  "draft": false,
  "title": "Read Abaqus model data with explicit units and atomic replacement",
  "media": "pending"
}
---
## What and why

Refs #98. `deck.read` now imports flat Abaqus model data into a new FieldCraft document: labelled nodes/connectivity, node/element sets (including references and GENERATE), element surfaces, constant elasticity/density and the existing writer's representable solid, shell, planar, truss and beam sections. The reply reports unread keywords with original physical lines, a bounded first 100 records and total count; unknown text is not stored or re-emitted.

Input units are an explicit consistent preset, defaulting to the current display preset while ignoring individual overrides. Central quantities convert every dimensional value to SI. The app captures that preset before its fresh worker session is made. File-menu, command-search and normal command paths use the same reader; native form imports run in the background with cooperative cancellation and Save / Don't Save / Cancel protection. The imported empty-journal document remains protected until saved. The result panel exposes skipped history/warnings, and rerunning the import uses the protected form path.

The engine checks read roots, caps and UTF-8, builds and validates the new model, and constructs its reply before replacing the old session. Any refusal or cancellation retains the current model/results/Arc identity, path, undo and unsaved work. Success clears prior geometry/results/history/path and creates an unsaved document. Bounded lexer, continuation, property collection, set expansion/work/depth and finalization loops do not execute input commands or open includes.

Three logical commits separate the pure reader, atomic command/catalogue, and app workflow/documentation. `tools/abaqus-import/beam.json` is an original reusable recipe: generate a beam, solve/write its own deck, import explicitly in mm units, recreate the analysis, solve and save using work-directory-relative paths. `docs/abaqus-import.md` documents use, format references and the exact limitations. No ROADMAP or file-format/schema change is included.

## Demonstration

Native source/capture: `c5203312e97857665facfcd985d38fd80eaddfd9`. Final files are `abaqus-import-98.mp4` (28 seconds), `import-report.png`, and `imported-beam-result.png`. All three were transferred through ordinary-Git artifact commit `5cfe546e7c2a978c7f9e1ff01d5faf44d9e80624`; original and index blob byte counts/SHA-256 were independently verified. Retrieval receipts and GitHub attachments are pending, so media remains pending. No media or generated project belongs to the feature branch.

The movie replays the committed original recipe through the actual app form and command interface: choose consistent mm-t-s-N-MPa units, confirm replacement, review all eight unread history keywords/lines and zero imported steps, deliberately recreate clamp/load/static step, then solve the imported model. Source and reconstructed CPU static peak displacement and von Mises results are exactly equal; displacement is 0.09613877492912645 mm at node 18 and peak stress rounds to 44.67 MPa.

The original steel beam is 80 x 10 x 10 mm, 81 nodes / 32 hex8 with incompatible modes, clamped root and total 100 N downward tip force. Its geometry, inputs and generated deck originate in FieldCraft commands. This verifies import/analysis reconstruction fidelity; it is not a fluid or heat-exchanger accuracy benchmark. The native footer and startup log identify RTX 4090 / Vulkan for the viewport; the static solve is CPU. The film is an edited sequence of held presented window frames with a caption band appended below the app. Inter SemiBold is the repository font under SIL Open Font License 1.1. Full MP4 decode passed.

## Validation

Candidate source: `c5203312e97857665facfcd985d38fd80eaddfd9`, based on mirrored upstream/develop `7d9c027d71b2c1ea3395e2552e95f0455a7244db`.

- Dedicated per-worktree target: decks 81 passed; engine 371 passed / 15 normal heavy ignores; UI 190 passed. Local test commands exclude only the six independently baseline-proven Windows CRLF golden failures and three previously verified Windows engine failures listed below. No new test is excluded.
- All-targets clippy with `-D warnings` passed for decks, engine and UI.
- The original durable recipe ran through the dedicated CLI and confined final-head MCP: 81 nodes, 32 hex8, three sets, one material/section/step and saved solved output. Original/reconstructed displacement and von Mises peaks are exactly equal. Malformed SYSTEM and outside-read-root inputs are refused while the current model and solved probe remain identical.
- Final-head native app: real form submit/cancel preserves prior solved data; discard imports with zero analysis steps; actual unread report, new analysis and saved solved output were inspected. The startup build pin and Vulkan hardware adapter are verified.
- The exact final app/CLI build passed in the dedicated target; no reused binary from another worktree was used.
- Targeted wasm32 checks passed for decks, engine and UI. Formatting and individual assets, layers and unit-factor gates passed. The newly built dedicated xtask reported zero changed paths for `affected --base HEAD`, confirming its current worktree.

Remote CI is pending for this new head/PR. No successful run from an earlier lane is attributed to it.

New regressions cover canonical writer/read/writer model data, supported element inverse mappings, section/property dimensions, central units, original lines through continuations/history, ranges/references/cycles/budgets, hostile prefixes/numbers, malformed scopes/options and explicit refusals. Atomic-command tests preserve actual result Arcs and usable undo; reapplying the same analysis produces matching static displacement. Cancellation tests cover the lexer, a partially populated range at its periodic check, reporter isolation and whole-session rollback. UI regressions cover frozen input units, replacement/reporting and failed/cancelled/saved imported-document lifecycle.

<details>
<summary>Existing Windows baseline exclusions</summary>

Six deck golden tests have checkout CRLF versus writer LF on unchanged base; isolated baseline executions and normalized-byte comparisons are retained externally:

- `tests/abaqus.rs::golden_decks`
- `tests/nastran.rs::golden_decks`
- LS-DYNA `golden_hex8_cantilever`, `golden_cantilever_modes_in_cases`, `golden_plate_with_pressure_and_gravity`, `golden_solids_and_distributing_coupling`.

Three engine tests have already verified Windows baseline failures and remain excluded locally:

- `bug_report::tests::a_long_session_and_the_model_are_attachments`
- `cmd::file::tests::every_migrated_journal_replays_to_its_migrated_model`
- `cmd::render::tests::labels_and_command_lines_read_well`.

No source skip/ignore is added. Remote CI will run its normal platform checks. The full local workspace gate, oracle and annotated heavy suites are not run for this reader/UI change.

</details>

## Known limitations

This is the first flat model-data slice. Steps, loads, boundary conditions, contact, coupling and other history/extensions are reported unread and must be deliberately recreated. There is no raw passthrough, comment fidelity, assembly/part/instance or INCLUDE support. Global SYSTEM and cylindrical/spherical node coordinates are refused; Cartesian NODE SYSTEM=R is accepted. Properties are constant isotropic/engineering elasticity and density; unsupported physical variants/tables are refused or named unread, never silently assumed.

Names must fit the model's 64-character ASCII identifier rule. Section names are generated. Canonical roundtrip checks supported keyword/data definitions and physical data, not unrecoverable metadata in noninjective writer encodings. Sectionless elements warn that family/formulation context is not retained on the bare mesh. T2D-only trusses lacking planar context, T3D mixed with planar continuum and overlapping/mixed section formulations are explicitly refused. An unread material option ends supported material context; later properties are refused rather than attached ambiguously.

Import success does not establish that an arbitrary input mesh is ready to solve. Review the unread report and create the intended analysis through normal commands. The parser is wasm-compatible; native recording/filesystem checks do not establish browser file persistence or browser runtime behavior. #98 remains referenced for maintainer assessment of the documented representation limits and later deck-history work.
