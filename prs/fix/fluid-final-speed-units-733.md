---
{
  "schema": 1,
  "kind": "pr",
  "action": "publish",
  "request_id": "pr-733-coupled-exchanger-agent-20261025",
  "agent": "coupled-exchanger-agent",
  "branch": "fix/fluid-final-speed-units-733",
  "issue": 733,
  "head_sha": "1bd4d551d70239c819f7b0c01568ec5fdd2a2e72",
  "base": "develop",
  "draft": false,
  "title": "Fluid: report final speed and peak time in display units",
  "media": "attached"
}
---
## What and why

Closes #733. `solve.fluid` converted the all-frame peak speed into display units but left the final-frame speed in SI. In the default millimetre display the two values therefore used different scales. The peak-speed time also bypassed conversion, which matters for the existing millisecond presets.

Both fields now use the central reply unit table: velocity for `final_max_speed`, time for `max_speed_time`. Stored results and CPU/GPU calculations are unchanged. Peak and final speeds retain their different meanings. The units guide explains how to compare the final reply with the last-frame velocity contour using the existing original flow-past sample.

## Demonstration

<!-- fieldcraft-attachments:start -->
### Attached files

These uploads are complete; earlier pending-upload wording below describes the original handoff.

Capture/source revision: `1bd4d551d70239c819f7b0c01568ec5fdd2a2e72`.

**fluid-reply-units-733.mp4**

https://github.com/user-attachments/assets/ff27877b-9285-4b67-8eb3-a2215fff3e06

**final-mm.png**

![final-mm.png](https://github.com/user-attachments/assets/0863a80b-cce8-472f-8cd2-7f45c571d77d)

**final-ms.png**

![final-ms.png](https://github.com/user-attachments/assets/ffc9d692-c31b-4a45-b36b-5f9868533e4e)

<!-- fieldcraft-attachments:end -->

An exact-head 23-second native Windows / RTX4090 / Vulkan video shows the original bracket flow: 184,320 cells, 3,200 GPU steps, eleven frames over 16 seconds of physical time. The final reply and automatic last-frame contour both give 39.287156684819394 mm/s; the all-frame peak is 41.693235691543805 mm/s at 1.6 s. Rerunning the same physical setup in the millisecond preset gives final speed 0.03928715668481939 mm/ms and peak time 1600 ms. Both runs explicitly used GPU compute; neither fell back to CPU.

The H.264 1280x1080 video is an explicitly edited montage of three held native presented frames with captions outside the app pixels. Two stills show the matching velocity legends. Visual review, full video decode and numerical checks passed. The app quit normally and released its control port.

Original media and safe numerical verification were transferred through the shared S3 exchange and downloaded back for exact byte/SHA256 verification. The identical four originals are retained in artifact commit `6588fb579710c1e623072083fb48109e358d2f2c` to bridge the existing native GitHub attachment publisher. All four maintainer received receipts now match the original bytes and hashes. Temporary S3 staging copies were cleared after that confirmed pickup; local originals and the Git inbox remain preserved. Media is absent from feature history. The three native GitHub attachments above are confirmed by matching received receipts and exact original hashes.

## Validation

Source `1bd4d551d70239c819f7b0c01568ec5fdd2a2e72`, based on mirrored develop `3751d3bb93676aa49d1a18c438424be88162ac03`:

- The new synthetic regression first failed on unchanged conversion: final speed 0.01 versus the expected 10 mm/s. All five presets now pass, including both millisecond presets. Zero values, sparse estimate replies, unit labels, physical frame times and unchanged metadata are covered.
- A real CPU Couette solve confirms the final-speed reply equals the last-frame automatic velocity-contour maximum. Existing distinct early-peak semantics remain covered.
- Full touched engine tests: 377 passed, 15 existing ignored heavy tests, three documented Windows baseline exclusions. Exact exclusions: `bug_report::tests::a_long_session_and_the_model_are_attachments` (escaped home path), `cmd::file::tests::every_migrated_journal_replays_to_its_migrated_model` (imperial last-bit replay), and `cmd::render::tests::labels_and_command_lines_read_well` (path separator). No source ignores or new exemptions were added.
- Engine all-target Clippy with warnings denied, fmt, diff checks, assets (377), layers (23 crates) and unit-factors (zero new factors; nine frozen migration factors) passed. The dedicated helper reports zero changed paths against HEAD, confirming the intended worktree root.
- Exact-head native app and CLI build passed in the dedicated per-worktree target (dev profile, optimized dependencies with debug information). Genuine presented-frame capture and both GPU command executions passed; the final and peak velocity contours match their corresponding reply fields exactly.
- Separate read-only real-hardware baseline verification at develop `3751d3b`: 27 GPU kernel/fluid/meshing tests passed on RTX4090 / Vulkan, zero missing-device or software/CPU skips; four existing throughput tests ignored. GPU/LBM/mesh-GPU sources are unchanged by this patch. This is small-case correctness evidence, not throughput, large-buffer or coupled-exchanger validation.

No solver algorithm, oracle case, expected reference or fixture is changed. No full local workspace gate, workspace Clippy, full oracle or wasm gate is claimed. At this exact head, relayed remote CI, oracle, heavy tests, dependency checks and affected-crate planning all passed. Windows app and macOS oracle jobs were skipped; they are not claimed as passing.

## Known limitations

This fixes command-boundary reporting shared by CPU and GPU solves. It does not add GPU fan/porous, LES, population-buffer splitting, backend support, steady-convergence proof or two-fluid exchanger acceptance. The native example is a 16-second physical startup calculation; edited held presented frames are a result showcase rather than real-time simulation playback. The exact-head native video and two stills are attached above; numerical checks and their scope remain stated separately.
