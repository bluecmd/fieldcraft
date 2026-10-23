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

GitHub video/still attachment upload is pending. The recording, native stills and solved project are preserved outside git for attachment after publication. No demo raster or video is present in any feature-branch commit. Original commands, reproduction notes, and the small layout/probe report are in `tools/demo-video/legend-fit-610*` and `docs/legend-layout.md`.

## Validation

- `cargo build -p fieldcraft`: passed; native app demonstrated on Windows with the NVIDIA GeForce RTX 4090 Vulkan viewport.
- `cargo test -p fieldcraft-ui-egui`: 188 tests and 3 doctests passed, with no ignored or filtered tests; rerun on the submitted head after updating against current develop.
- `cargo check -p fieldcraft-ui-egui`: passed.
- `cargo clippy -p fieldcraft-ui-egui --all-targets -- -D warnings`: passed.
- `cargo fmt --all -- --check` and `git diff --check`: passed.
- Regression coverage checks small/large/small resizing, selection/navigation clearance, unchanged band count and over-limit colour, full-size Max/Min targets, actual focus/probe behavior, and non-overlapping numeric tick glyphs with and without a stale-result warning.

## Known limitations

Native sizes below the supported 960 by 600 minimum and other operating systems were not tested. No solver numerics or commands changed; solver oracles and the full workspace gate were not run locally. GitHub CI remains responsible for the upstream gate. The media receipt remains pending until GitHub attachments are actually uploaded.
