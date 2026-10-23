---
{
  "schema": 1,
  "kind": "pr",
  "action": "publish",
  "request_id": "pr-610-legend-layout-agent-20261023",
  "agent": "legend-layout-agent",
  "branch": "fix/legend-fit-610",
  "issue": 610,
  "head_sha": "252fa873b5ed4cfc1a0e5714f56806b47b9a9f00",
  "base": "develop",
  "draft": false,
  "title": "Results: keep dense legends readable above navigation",
  "media": "pending"
}
---
## What and why

Closes #610. A clamped 24-band Cantilever plot had a 507-point legend in a 406-point viewport at 960 by 600, hiding its lowest bands and Max/Min links. The legend now keeps its full-size title, metadata and clickable extrema, and fits every colour swatch into the available space above navigation. Short windows show fewer numeric ticks at readable spacing. The over-limit swatch keeps both the true and clamped maxima labelled. Expanding the window restores the normal swatch spacing and tick density without changing the result, range or selected band count.

## Demonstration

A 32-second native app recording shows the failure, the corrected 960 by 600 layout, clicks on Max (node 23) and Min (node 127), expansion to 1400 by 900, and a return to the smaller window. The original Cantilever sample is a small-strain linear-elastic static solve, with 24 stress bands clamped to 0–10 MPa and an extra over-limit colour. This is a UI resize demonstration, not a physical transient. The true maximum (151.0 MPa) and clamp maximum (10.00 MPa) stay distinct. The small legend is 316 points tall; the larger view restores its natural 507-point height.

The recording and two stills were transferred through the separate ordinary-Git `artifact-queue` inbox in commit `bfb109cf307cb7109d55e8743276c272fc01322c`. Maintainer receipts for this exact tested head confirm `received` for `legend-fit-610.mp4`, `after-small.png` and `after-max.png`, with file sizes and SHA-256 hashes verified. The receipts say `github_attachment: pending_maintainer`: the bytes are received, but GitHub PR attachments are still pending. Local originals are preserved. Feature commits retain only original commands, reproduction notes and the small layout/probe report in `tools/demo-video/legend-fit-610*` and `docs/legend-layout.md`.

## Validation

- `cargo build -p fieldcraft`: passed; native app demonstrated on Windows with the NVIDIA GeForce RTX 4090 Vulkan viewport.
- `cargo test -p fieldcraft-ui-egui`: 188 tests and 3 doctests passed, with no ignored or filtered tests; rerun on the submitted head after updating against current develop.
- `cargo check -p fieldcraft-ui-egui`: passed.
- `cargo clippy -p fieldcraft-ui-egui --all-targets -- -D warnings`: passed.
- `cargo fmt --all -- --check` and `git diff --check`: passed.
- Regression coverage checks small/large/small resizing, selection/navigation clearance, unchanged band count and over-limit colour, full-size Max/Min targets, actual focus/probe behavior, and non-overlapping numeric tick glyphs with and without a stale-result warning.
- Maintainer-relayed CI on this exact head is successful. The `cargo xtask ci` rerun in run `40023518993` started at `2026-10-23T23:14:46Z` and completed successfully at `2026-10-23T23:18:58Z` (job `120703853463`). Heavy tests, oracle, dependency checks, bot tests and the plan also passed. Windows app tests and the macOS oracle were skipped, not passed.
- The first CI attempt failed only the unchanged engine cancellation timing check: 5.792319807 seconds exceeded its existing 5-second limit. The rerun passed at the same source head; no solver polling, cancellation deadline or test was changed to obtain that result. The original run also passed all 188 UI tests and the 17-crate wasm check.

## Known limitations

Native sizes below the supported 960 by 600 minimum were not demonstrated. The successful exact-head Linux CI result complements the local Windows checks; the full workspace gate was not run locally. GitHub attachment remains pending. Reviewer housekeeping requests to leave ROADMAP.md unchanged and move the one-run QA notes/recipes into the PR description remain a separate follow-up.

## Maintainer follow-up

The CI rerun request is complete; no further rerun is requested. The received video and stills await maintainer GitHub attachment upload. Relay any further review or CI context through this queue. Only the maintainer handles authenticated GitHub operations and attachment upload.
