---
{
  "schema": 1,
  "kind": "pr",
  "action": "publish",
  "request_id": "pr-689-legend-layout-agent-20261024",
  "agent": "legend-layout-agent",
  "branch": "fix/tet-seam-diagnostics-689",
  "issue": 689,
  "head_sha": "89b2c9dcebb0f49a4de73d9968cd2d463e056c27",
  "base": "develop",
  "draft": false,
  "title": "mesh: explain explicit welding for tiny tetrahedral boundary seams",
  "media": "pending"
}
---
## What and why

Closes #689. When tet meshing refuses an open boundary, its error now preserves the located defects and points to `mesh.surface.check`, `weld_distance` (a length), and `weld` (a fraction of the bounding-box diagonal). The guidance is conditional on confirming a small seam, keeps the chosen distance below features to preserve, and directs orientation/nonmanifold defects back to the source surface.

The tet command reference and weld form help explain the two controls and their combined distance. The existing default remains twice the kernel linear tolerance for a tessellated design body; a boolean retry can displace faces farther than that. Boundary validation, meshing algorithms, and default tolerances are unchanged.

## Demonstration

A 34.0-second captioned recording uses real native presented frames on Windows, NVIDIA RTX 4090, Vulkan. It shows the old open-edge-only error, readable new guidance in the full form, the visible numeric Weld distance entry, the successful mesh, and refusal of a separate surface with a reversed face. No offscreen frames or private CAD model were used.

The original analytic fixture is a 10 mm cube with six independently triangulated quadrilateral faces (24 corner records, 12 triangles). The top face alone is raised by 0.00001 mm and the result is packaged as a tagged 3MF. A second cube, offset 14 mm along X, has its top triangles reversed and no seam. This is a synthetic seam and orientation control, not a reproduction of the original SolidCraft boolean retry.

Native command recipe, in mm display units:

```json
[
  {"command":"geometry.import.mesh","params":{"path":"tiny-seam.3mf"}},
  {"command":"mesh.tet.generate","params":{"surface":0,"size":3}},
  {"command":"mesh.tet.generate","params":{"surface":0,"size":3,"weld_distance":0.00002}},
  {"command":"mesh.quality","params":{}},
  {"command":"geometry.import.mesh","params":{"path":"wrong-orientation.3mf"}},
  {"command":"mesh.tet.generate","params":{"surface":1,"size":3,"weld_distance":0.00002}}
]
```

The first and last mesh commands intentionally fail; execute them separately to continue the demonstration.

| Case | Observed result |
|---|---|
| Default, 0.00001 mm seam | 8 open-edge defects, 0 nodes/elements; new inspection/weld guidance visible in form |
| Explicit 0.00002 mm weld distance | 219 tet4, 74 nodes, all 219 certified valid; volume 1000.0003293172924 mm³ |
| Same explicit retry on unchanged baseline | Identical counts and volume |
| Reversed-face control, explicit weld | 4 orientation defects; existing 219-element mesh retained |

Recording: `tet-seam-diagnostics-689.mp4` (H.264, 1280×1098, 10 fps, 511306 bytes). Stills: `after-seam-refused.png`, `after-weld-meshed.png`. The before frame comes from base `7d9c027d71b2c1ea3395e2552e95f0455a7244db`; all after frames come from tested head `89b2c9dcebb0f49a4de73d9968cd2d463e056c27`. Original captures, recipes, and run reports remain outside feature history. Ordinary-Git artifact handoff is in progress; received receipts and maintainer GitHub attachments are pending. `media=pending` is intentional.

## Validation

At head `89b2c9dcebb0f49a4de73d9968cd2d463e056c27`, based on the current fork `upstream-develop` mirror `7d9c027d71b2c1ea3395e2552e95f0455a7244db`:

- `cargo check -p fieldcraft-engine`: passed.
- `cargo test -p fieldcraft-engine -- --test-threads=3 --skip bug_report::tests::a_long_session_and_the_model_are_attachments --skip cmd::file::tests::every_migrated_journal_replays_to_its_migrated_model --skip cmd::render::tests::labels_and_command_lines_read_well`: 368 passed, 15 existing heavy tests ignored, exactly 3 baseline exclusions, 0 failures; doc tests: 0.
- The new `cmd::mesh::tests::tet_seam_diagnostic_and_explicit_weld_preserve_validation` also passed its final targeted rerun after tightening its inch-coordinate assertion. It exercises a generated tiny seam above the actual CAD default, both explicit controls, unit conversion in inch display units, orientation refusal, and unchanged model/journal/source on failure.
- `cargo clippy -p fieldcraft-engine --all-targets -- -D warnings`, `cargo fmt --all --check`, and `git diff --check`: passed.
- `cargo run -p fieldcraft-cli -- features`: regenerated reference is identical; no generated-file diff.
- `cargo build -p fieldcraft` and the native demonstration/parity/quality checks: passed.

Upstream CI is pending publication. No local full workspace gate, heavy suite, wasm check, or oracle run was requested or claimed.

## Known limitations

This improves actionable refusal and option discoverability. It does not measure a safe weld distance, automatically heal an input, enlarge the default tolerance, or repair inconsistent face orientation. The original issue's SolidCraft boolean-nudge design and reported 7153-tet mesh were not rerun; the independently generated seam exercises the failure/retry class without private geometry.

Three pre-existing Windows failures on unchanged base were excluded from the broad local engine run: `bug_report::tests::a_long_session_and_the_model_are_attachments` (JSON-escaped home redaction), `cmd::file::tests::every_migrated_journal_replays_to_its_migrated_model` (imperial-density last bit), and `cmd::render::tests::labels_and_command_lines_read_well` (path separator). They are not addressed by this change. Heavy/wasm/oracle execution remains with upstream CI as applicable. The media still requires maintainer attachment URLs.