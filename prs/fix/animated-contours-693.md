---
{
  "schema": 1,
  "kind": "pr",
  "action": "publish",
  "request_id": "pr-693-contour-animation-agent-20261023",
  "agent": "contour-animation-agent",
  "branch": "fix/animated-contours-693",
  "issue": 693,
  "head_sha": "f79299f7791e69e6f01cd4094acc00749bbcb48f",
  "base": "develop",
  "draft": false,
  "title": "Results: animate proportional contour colours and explain missing increment fields",
  "media": "attached"
}
---
## What and why

Refs #693 and #205. Linear static animations now change stress and displacement colours with the displayed proportional load, using one fixed full-load colour scale rather than rescaling each frame. Signed modal components reverse with the mode; magnitudes stay nonnegative. Absolute temperatures remain unchanged. Contours also animate with Deformed unticked, and stopping playback restores solved values. The legend and its Auto controls show the same fixed bounds while Max/Min track the current displayed values.

The legend explicitly calls linear/modal playback display interpolation: it does not create additional solved increments or a physical transient. Node probes retain actual solve data, labelled Solved final state or Unscaled mode shape; supplemental interpolated contour values are prefixed Displayed.

Nonlinear implicit increments currently save displacement/reaction only. Earlier stress/contact fields now return an explicit contour_unavailable reason and a visible warning rather than showing final stress on an earlier shape. Complete final-state contours remain available; a truncated history's last saved frame is not mistaken for the final solve. Recording nonlinear stress/contact history remains #205, so these issue references intentionally stay open. Explicit/fluid frame fields are not proportionally scaled.

Review follow-up: idle heat-only plots and undeformed invariant thermal contours reuse their cached plot while Animate is on. Static/modal geometry still animates under an invariant contour, and saved nonlinear/explicit/fluid histories keep playing. Standalone heat histories do not gain automatic playback in this change. The maintainer-owned ROADMAP.md is unchanged; run-specific reproduction and QA evidence have moved from source files into this description.

## Demonstration

<!-- fieldcraft-attachments:start -->
### Attached files

These uploads are complete; earlier pending-upload wording below describes the original handoff.

Capture/source revision: `2df672d5120891ee7c112c57cc6be6ce15eb320b`.

Historical capture of the static/modal/probe behavior described in this PR. It does not demonstrate the later idle-heat cache guard; that change has separate regressions. No new recording of the revised head is claimed.

**animated-contours-693.mp4**

https://github.com/user-attachments/assets/0815e20e-9335-4255-8fd9-4fa062abedf1

**nonlinear-unavailable.png**

![nonlinear-unavailable.png](https://github.com/user-attachments/assets/b468ff75-8825-48c1-9298-0b1ce0615470)

**proportional-probe.png**

![proportional-probe.png](https://github.com/user-attachments/assets/47ca3e09-1f98-428e-b24b-984b672df2df)

<!-- fieldcraft-attachments:end -->

A real native-app recording has been pushed through the fork-only artifact-queue ordinary Git handoff: animated-contours-693.mp4, 29.5 seconds, 1440 x 948, H.264 at 12 fps, 857495 bytes. The maintainer retrieves verified Git blobs and handles GitHub PR attachments; agents require no GitHub sign-in, token or UI upload. The verified artifact receipts now provide the attached GitHub media URLs embedded above. The video shows the load cycle, colours on an undeformed mesh, signed mode playback, labelled solved-state probes and the missing nonlinear history explanation. The original analytic cantilever has 315 nodes and 160 hex8 elements; all solves use the CPU with 3 threads. This is presentation QA, not a physical transient or a solver validation study.

A retained quarter-load native still shows a displayed maximum of 37.74 MPa, fixed legend/Auto bounds of 0 to 151.0 MPa, and a separately labelled final-state probe of 151.0 MPa. The original capture was made from source commit 6e1747d8ef0445ee1638d0ce8223f2cd40a1fb6e, preserved in prior head 2df672d5120891ee7c112c57cc6be6ce15eb320b; it demonstrates the unchanged static/modal/probe behavior. It is not a recording of the revised head or the idle-heat cache guard, which has separate regressions. The recording and QA stills are confined to the separate artifact inbox; the solved binary project remains outside git. All are absent from every unmerged feature commit.

The original analytic beam is 100 x 10 x 20 mm, using 315 nodes, 160 hex8 elements, six contour bands and deformation exaggeration 25. Full-load von Mises extrema were 3.9981482484799424 to 150.96411589008667 MPa. The recording SHA-256 is d6fa974768bba6ad995b267759b937f6664e76c14e52ea4f099d16aa5be3516c. Captures are original FieldCraft native-window output and analytic geometry; captions use the repository's SIL OFL 1.1 Inter font. No commercial CAE output or third-party geometry is used. Original notes, recipe and report remain preserved externally with the capture's original source pin.

Reproduce the linear/modal display through the same normal session commands:

```json
[
  {"command": "file.sample", "params": {"name": "cantilever"}},
  {"command": "solve.static", "params": {"step": "Load", "threads": 3}},
  {"command": "solve.frequency", "params": {"step": "Modes", "threads": 3}},
  {"command": "results.contour", "params": {"step": "Load", "field": "von_mises", "bands": 6}},
  {"command": "results.animate", "params": {"on": true}}
]
```

Keep the camera still through a load cycle, untick Deformed to see colours continue without mesh motion, then stop playback to restore solved values. Select displacement_z on Modes and animate to see signed normalized mode shapes against one symmetric scale. Intermediate nonlinear stress/contact contours explain that the field was not saved; probes remain labelled Solved final state. Deformation exaggeration scales the drawn shape only, not contour values.

The shared ordinary-Git artifact batch was delivered in bfb109cf307cb7109d55e8743276c272fc01322c. This PR's three verified receipts now say github_attachment=attached and provide the native GitHub URLs above. The original capture remains pinned to 2df672d5120891ee7c112c57cc6be6ce15eb320b; the maintainer's applicability note explicitly excludes the later idle-heat guard. No revised-head capture or duplicate unchanged upload is claimed. Originals remain outside feature history.

## Validation

Windows; AMD Ryzen 9 7950X; NVIDIA RTX 4090 Vulkan viewport. No GPU solver changes or GPU compute benchmark.

- cargo test -p fieldcraft-engine -p fieldcraft-ui-egui with the three known Windows exclusions below: engine 368 passed, 15 ignored; UI 190 passed plus 3 doctests on the revised source tree.
- Regressions cover zero/half/full static colour values, fixed bands and actual rendered colours, cached solved values, undeformed playback, signed modal components/nonnegative magnitudes, unchanged absolute temperatures, exact saved explicit stress, unavailable nonlinear increment fields, capped-history refusal and full-state probe labels. Review regressions additionally prove steady/transient heat plot cache reuse for temperature and heat flux, invalidation on an explicit view-frame change, moving static/modal geometry under invariant thermal contours, and nonlinear recorded playback on an undeformed mesh. Existing thermal-fluid and explicit-frame tests still pass.
- cargo check -p fieldcraft-engine -p fieldcraft-ui-egui: passed.
- cargo clippy -p fieldcraft-engine -p fieldcraft-ui-egui --all-targets -- -D warnings: passed.
- cargo fmt --all -- --check and git diff --check: passed.
- docs/features.md regenerated with the built CLI; unchanged.
- Revised tested source head: f79299f7791e69e6f01cd4094acc00749bbcb48f, based on fork upstream-develop mirror 7d9c027d71b2c1ea3395e2552e95f0455a7244db. Review changes were folded into the source commit; ROADMAP.md and run-specific QA files are absent from the final feature diff. Native app/CLI and the reviewed recording belong to the original source pin stated above; no unchanged artifact was uploaded again and no fresh capture is claimed for the revised head.

## Relayed CI and capture follow-up

The maintainer's git-only feedback confirms success on current head f79299f7791e69e6f01cd4094acc00749bbcb48f: cargo xtask ci completed 2026-10-23T23:46:16Z, cargo xtask oracle completed 23:44:59Z, and cargo xtask test --heavy completed 23:59:09Z. Dependency licences/advisories/sources, affected-crate planning, and claim/triage bot tests also succeeded. Windows app tests and the macOS oracle were skipped; skipped checks are not claimed as passing. The previous head's failed MCP capture test subsequently passed on its exact-head rerun before this review update.

The first ci attempt on previous head 2df672d5120891ee7c112c57cc6be6ce15eb320b failed at MCP tests::screenshot_takes_a_camera_orientation (crates/mcp/src/tests.rs:229, PNG bytes differ). A bounded strict-hardware diagnostic reproduced a pre-existing shared-capture concurrency defect on that head and mirrored upstream baseline 7d9c027d71b2c1ea3395e2552e95f0455a7244db. Two independent MCP servers draw unsolved cantilevers concurrently, 300 pictures per camera, comparing PNG bytes and camera JSON with isolated references. FIELDCRAFT_GPU=hardware and FIELDCRAFT_REQUIRE_GPU=1 require the RTX 4090 Vulkan device with MSAA 4.

- Previous exact head 2df672d: 13 and 24 PNG mismatches out of 300 pictures each; zero reported-camera changes.
- Exact baseline, rebuilt with the same normal app/CLI feature set: 12 and 20 PNG mismatches out of 300 each; zero reported-camera changes.
- Baseline control, serialising only the diagnostic picture calls: zero mismatches in 600 pictures, with unchanged reported cameras.

The unchanged shared Capturer owns one Pipelines camera uniform, written before queue.submit; overlapping calls can overwrite it. The observations are consistent with this pre-existing interleaving in an unsolved case. The Rust recipe, JSON results and source pins are preserved locally for a separate handoff. This diagnostic is separate from the successful relayed CI rerun; the fresh local MCP dev/testkit-feature build was cancelled. No test assertion was weakened and no production renderer was patched.

The successful CI runs do not repair the baseline concurrency defect. Separately track/claim that shared-renderer follow-up with its own concurrent regression and production fix; it remains outside #693.

## Known limitations

The existing Windows tests excluded explicitly were bug_report::tests::a_long_session_and_the_model_are_attachments (escaped home path, #554), cmd::file::tests::every_migrated_journal_replays_to_its_migrated_model (imperial density last bit), and cmd::render::tests::labels_and_command_lines_read_well (Windows path separator). No new failure was excluded.

Nonlinear stress/contact history remains unavailable (#205); probes remain actual solved final-state data. Proportional interpolation assumes one scaled linear loading path, not independent load histories or nonlinear response. Heavy/ignored tests, the full workspace gate and wasm were not run locally, per repository workflow; the current-head remote CI, heavy and oracle jobs passed as described above. The oracle was not run locally because solver numerics did not change. The baseline shared-capture concurrency defect remains a separate follow-up. The unchanged original demonstration is attached with its capture revision and applicability limitation preserved.
