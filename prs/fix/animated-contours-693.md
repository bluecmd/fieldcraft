---
{
  "schema": 1,
  "kind": "pr",
  "action": "publish",
  "request_id": "pr-693-contour-animation-agent-20261023",
  "agent": "contour-animation-agent",
  "branch": "fix/animated-contours-693",
  "issue": 693,
  "head_sha": "2df672d5120891ee7c112c57cc6be6ce15eb320b",
  "base": "develop",
  "draft": false,
  "title": "Results: animate proportional contour colours and explain missing increment fields",
  "media": "pending"
}
---
## What and why

Refs #693 and #205. Linear static animations now change stress and displacement colours with the displayed proportional load, using one fixed full-load colour scale rather than rescaling each frame. Signed modal components reverse with the mode; magnitudes stay nonnegative. Absolute temperatures remain unchanged. Contours also animate with Deformed unticked, and stopping playback restores solved values. The legend and its Auto controls show the same fixed bounds while Max/Min track the current displayed values.

The legend explicitly calls linear/modal playback display interpolation: it does not create additional solved increments or a physical transient. Node probes retain actual solve data, labelled Solved final state or Unscaled mode shape; supplemental interpolated contour values are prefixed Displayed.

Nonlinear implicit increments currently save displacement/reaction only. Earlier stress/contact fields now return an explicit contour_unavailable reason and a visible warning rather than showing final stress on an earlier shape. Complete final-state contours remain available; a truncated history's last saved frame is not mistaken for the final solve. Recording nonlinear stress/contact history remains #205, so these issue references intentionally stay open. Explicit/fluid frame fields are not proportionally scaled.

## Demonstration

A real native-app recording has been pushed through the fork-only artifact-queue ordinary Git handoff: animated-contours-693.mp4, 29.5 seconds, 1440 x 948, H.264 at 12 fps, 857495 bytes. The maintainer retrieves verified Git blobs and handles GitHub PR attachments; agents require no GitHub sign-in, token or UI upload. Media status remains pending until real GitHub attachment URLs exist. The video shows the load cycle, colours on an undeformed mesh, signed mode playback, labelled solved-state probes and the missing nonlinear history explanation. The original analytic cantilever has 315 nodes and 160 hex8 elements; all solves use the CPU with 3 threads. This is presentation QA, not a physical transient or a solver validation study.

A retained quarter-load native still shows a displayed maximum of 37.74 MPa, fixed legend/Auto bounds of 0 to 151.0 MPa, and a separately labelled final-state probe of 151.0 MPa. Recipe, source pin, numerical observations, recording hash and provenance are in docs/results-animation.md and tools/demo-video/animated-contours-693-report.json. The recording and QA stills are confined to the separate artifact inbox; the solved binary project remains outside git. All are absent from every unmerged feature commit.

The shared ordinary Git artifact batch was pushed successfully as bfb109cf307cb7109d55e8743276c272fc01322c (three recordings and six QA stills). The remote SHA was verified, and the staging helper verified actual byte hashes and sizes for all nine files. This PR's manifests cover animated-contours-693.mp4, proportional-probe.png and nonlinear-unavailable.png under fix/animated-contours-693/2df672d5120891ee7c112c57cc6be6ce15eb320b. All three corresponding receipts under receipts/artifacts/ say received with github_attachment=pending_maintainer; their SHA-256 and byte counts match the manifests. Actual bytes are received, while GitHub PR attachments remain pending. The current protocol requires neither LFS nor an upstream bootstrap; the earlier failed LFS attempt is superseded. Originals remain preserved outside feature history.

## Validation

Windows; AMD Ryzen 9 7950X; NVIDIA RTX 4090 Vulkan viewport. No GPU solver changes or GPU compute benchmark.

- cargo test -p fieldcraft-engine -p fieldcraft-ui-egui with the three known Windows exclusions below: engine 368 passed, 15 heavy tests ignored; UI 189 passed plus 3 doctests. After the final Auto-widget correction, the complete UI suite and doctests passed again.
- Regressions cover zero/half/full static colour values, fixed bands and actual rendered colours, cached solved values, undeformed playback, signed modal components/nonnegative magnitudes, unchanged absolute temperatures, exact saved explicit stress, unavailable nonlinear increment fields, capped-history refusal and full-state probe labels.
- cargo check -p fieldcraft-engine -p fieldcraft-ui-egui: passed.
- cargo clippy -p fieldcraft-engine -p fieldcraft-ui-egui --all-targets -- -D warnings: passed.
- cargo fmt --all -- --check and git diff --check: passed.
- docs/features.md regenerated with the built CLI; unchanged.
- Native app/CLI built and real presented-window recording visually reviewed. Source commit 6e1747d8ef0445ee1638d0ce8223f2cd40a1fb6e; the final commit adds only the reproduction/QA report. Feature head rebased on current develop 7d9c027d71b2c1ea3395e2552e95f0455a7244db.

## Relayed CI and capture follow-up

The maintainer's git-only feedback for exact unchanged head 2df672d5120891ee7c112c57cc6be6ce15eb320b reports a successful cargo xtask ci rerun, started 2026-10-23T23:14:46Z and completed 23:18:28Z. Relayed results also report engine 371 passed, UI 189 passed, wasm check passed, and successful heavy/oracle jobs. Windows app tests and the macOS oracle were skipped; they are not claimed as passing.

The first ci attempt failed at MCP tests::screenshot_takes_a_camera_orientation (crates/mcp/src/tests.rs:229, PNG bytes differ). A bounded strict-hardware diagnostic reproduced a pre-existing shared-capture concurrency defect on both this head and mirrored upstream baseline 7d9c027d71b2c1ea3395e2552e95f0455a7244db. Two independent MCP servers draw unsolved cantilevers concurrently, 300 pictures per camera, comparing PNG bytes and camera JSON with isolated references. FIELDCRAFT_GPU=hardware and FIELDCRAFT_REQUIRE_GPU=1 require the RTX 4090 Vulkan device with MSAA 4.

- Exact head: 13 and 24 PNG mismatches out of 300 pictures each; zero reported-camera changes.
- Exact baseline, rebuilt with the same normal app/CLI feature set: 12 and 20 PNG mismatches out of 300 each; zero reported-camera changes.
- Baseline control, serialising only the diagnostic picture calls: zero mismatches in 600 pictures, with unchanged reported cameras.

The unchanged shared Capturer owns one Pipelines camera uniform, written before queue.submit; overlapping calls can overwrite it. The observations are consistent with this pre-existing interleaving in an unsolved case. The Rust recipe, JSON results and source pins are preserved locally for a separate handoff. This diagnostic is separate from the successful relayed CI rerun; the fresh local MCP dev/testkit-feature build was cancelled. No test assertion was weakened and no production renderer was patched.

The successful rerun does not repair the baseline concurrency defect. Separately track/claim that shared-renderer follow-up with its own concurrent regression and production fix; it remains outside #693.

## Known limitations

The existing Windows tests excluded explicitly were bug_report::tests::a_long_session_and_the_model_are_attachments (escaped home path, #554), cmd::file::tests::every_migrated_journal_replays_to_its_migrated_model (imperial density last bit), and cmd::render::tests::labels_and_command_lines_read_well (Windows path separator). No new failure was excluded.

Nonlinear stress/contact history remains unavailable (#205); probes remain actual solved final-state data. Proportional interpolation assumes one scaled linear loading path, not independent load histories or nonlinear response. Heavy/ignored tests, the full workspace gate and wasm were not run locally, per repository workflow. The oracle was not run locally because solver numerics did not change; the relayed ci rerun and heavy/oracle jobs passed. The baseline shared-capture concurrency defect remains a separate follow-up. Maintainer GitHub attachment upload is pending; artifact retrieval is complete.
