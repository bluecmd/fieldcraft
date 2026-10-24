---
{
  "schema": 1,
  "kind": "pr",
  "action": "publish",
  "request_id": "pr-99-windows-dxc-agent-20261024-1",
  "agent": "windows-dxc-agent",
  "branch": "model/material-library-99",
  "issue": 99,
  "head_sha": "35ef3b12c7fb78cd1065042e9667736cb2cd15a9",
  "base": "develop",
  "draft": false,
  "title": "Materials: add a cited library of nine structural metals",
  "media": "pending"
}
---
## What and why

Closes #99. The Material menu now offers a fixed, cited library: browse all nine structural metals or filter by family, then add one as an ordinary editable material. Both actions are typed commands (`model.material.library.list` and `model.material.library`) available through the desktop, CLI, scripts and MCP. There is no runtime download or arbitrary external database importer.

The nine entries cover S235, S355, 304/1.4301, 316/1.4401, 6061-T6, 6082-T6, 7075-T6, 2024-T3 and annealed Ti-6Al-4V. Each of the 54 individually selected public facts carries its SI unit, design/minimum/typical basis, exact document/edition/table or clause/page/link, and its product/temperature applicability. This is original curation and annotations, not a copied handbook layout or commercial material library. The data attribution and a source/usage guide are included.

The model already stores SI and has a public Quantity/display boundary. This implements the issue against that current architecture: all five presets and temperature overrides work, including imperial/Fahrenheit; the older unspecified alias follows the existing mm-tonne-second behavior and default follows user settings. No duplicate conversion factors or obsolete imperial-thermal refusal is introduced. The reply explicitly labels `material_si` separately from displayed values and their units. Missing properties stay absent. Yield/tensile information appears only in `not_applied`; no plasticity, damage, erosion, safety factor or failure criterion is selected. Adding refuses duplicate/invalid names and unknown/malformed input atomically.

The stress-free reference temperature is an explicit modelling choice, defaulting to 20 degrees Celsius in every display system. It is reported separately from cited handbook facts, including whether expansion exists to use it. The library provides constant isotropic approximations, not temperature-dependent curves.

Source specifics: the 2024-T3 bare-sheet entry explicitly identifies MIL-HDBK-5H (1998), Table 3.2.3.0(b1), reproduced with Change Notice 1 (2001). Its accessible link is a reproduction of the original handbook text; the official DLA index identifies later 5J, but these values are not relabelled as that edition. Other primary sources include the cited EN 1993/1999 editions, NASA's public 1972 alloy handbooks and manufacturer Hydro, Outokumpu and TIMET sheets. Every selected value was checked individually. The Ti isotropic approximation explicitly combines NASA's design modulus with TIMET physical values rather than pretending they describe one tested lot.

## Demonstration

The 28-second native workflow montage shows the actual new Add Library Material form, the ordinary material inspector and the solved temperature field/probe. Every source frame is a presented screenshot of the current-head app, without an offscreen fallback. The CPU solver and NVIDIA RTX 4090 Vulkan viewport are distinguished; this is a guided feature demonstration, not an elapsed-time performance claim.

Both executables were built after frozen tested head `35ef3b12c7fb78cd1065042e9667736cb2cd15a9`. Native executable SHA-256: `f6a141336ef485c7ae1aaeb517f266d78f804651c986366045069a94851a5542`; CLI SHA-256: `98be90a58502311e43f761b5708dbb5a3e2d6e73d9d48fca4f700bf0aab6952f`. The owned demo app was closed after capture.

The original runnable `docs/materials/conduction.json` recipe builds a 20 x 30 x 120 mm bar from library 304 stainless, uses its cited 15 W/(m K) constant conductivity at 20 C, holds the ends at 293.15 K and 303.15 K and leaves the other faces insulated. The CPU solution with 36 hex8 cells / 84 nodes gives midpoint 298.15 K, axial heat flux -1250 W/m2 and opposite endpoint heat rates 0.75 W. This is a synthetic implementation check of the material/units/solver path, not experimental material validation, a transient result or coupled-exchanger design accuracy.

## Validation

- `cargo test -p fieldcraft-model`: 87 unit + 9 definition + 10 units tests pass. Catalogue completeness/citations/plausibility, all-five-preset unit round trips, original non-SI parser conversions, missing citation/constant/duplicate/hostile IDs and no applied inelastic laws are checked.
- `cargo test -p fieldcraft-engine` with only the three previously confirmed Windows baseline failures excluded: 372 passed, 15 existing heavy tests ignored, 3 filtered out, 0 failures; 0 doctests. This includes the five new regressions, command/form/catalogue contracts and global hostile-parameter suite.
- All 9 entries x 5 presets x 3 temperature scales are the same SI materials; displayed values round-trip through Quantity. Library steel gives bit-identical cantilever displacement to the sample. The cited 304 conductor matches its exact linear patch and signed endpoint heat balance.
- `cargo clippy -p fieldcraft-model -p fieldcraft-engine --all-targets -- -D warnings` and `cargo check -p fieldcraft-model -p fieldcraft-engine --target wasm32-unknown-unknown` pass.
- `cargo test -p fieldcraft-ui-egui form::`: all 8 generic form drawing/labels/required-field/units tests pass. `cargo fmt --all -- --check` and `git diff --check` pass. `cargo build -p fieldcraft -p fieldcraft-cli` passes at the frozen head. Actual CLI commands --json advertises both typed commands, its features output exactly matches committed `docs/features.md`, and the original conductor recipe runs successfully. Actual native form submission/material inspection/CPU solve/probe match the expected properties and numerical result.
- Fresh fetch/rebase on the fork upstream-develop mirror was a no-op. Feature history is text-only; no new numerical solver, unit conversion rule, dependency or golden fixture changed.

## Known limitations

This is a small room-temperature constant-property library. Missing conductivity/specific heat/expansion must be independently supplied before analyses that need them. Mean expansion coefficients over reported intervals remain constant approximations. Strengths are source/product/condition-specific information, not current procurement/design allowables or an automatically selected failure model. Material lots, texture and temperature can differ from these nominal values.

The engine test command excludes the previously reproduced Windows baseline failures `bug_report::tests::a_long_session_and_the_model_are_attachments` (#554 escaped home path), `cmd::file::tests::every_migrated_journal_replays_to_its_migrated_model` (imperial-density last-bit comparison) and `cmd::render::tests::labels_and_command_lines_read_well` (path separator). Existing heavy tests remain ignored. No broad local CI gate/oracle, non-Windows native runtime, browser runtime or sustained GPU solver benchmark is claimed. ROADMAP remains unchanged.

The coordinator normal-pushed the immutable `cited-material-library.mp4`, `library-form.png`, `library-conduction.png` and small safe `verification.json` through ordinary Git in artifact-queue commit `88ca45c8c162cacdafe834e1277b3504f22e22af`, under `objects/model/material-library-99/35ef3b12c7fb78cd1065042e9667736cb2cd15a9/`. Index SHA-256 values and byte counts matched all four originals, and the full video decoded successfully. Maintainer received receipts and actual GitHub video/image attachment URLs are still pending. The JSON is inbox numerical/provenance evidence, not a native attachment. Media remains pending until real attachment URLs exist. Raw native logs, source-PDF research and original capture/encoding recipes remain private; there are no video, still, project or binary files in feature history.
