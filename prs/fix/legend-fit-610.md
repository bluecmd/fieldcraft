---
{
  "schema": 1,
  "kind": "pr",
  "action": "publish",
  "request_id": "pr-610-legend-layout-agent-20261023",
  "agent": "legend-layout-agent",
  "branch": "fix/legend-fit-610",
  "issue": 610,
  "head_sha": "1d4ed083c09f25cc2027e9cc8a57d03b2637a825",
  "base": "develop",
  "draft": false,
  "title": "Results: keep dense legends readable above navigation",
  "media": "pending"
}
---
## What and why

Closes #610. A clamped 24-band Cantilever plot had a 507-point legend in a 406-point viewport at 960 by 600, hiding its lowest bands and Max/Min links. The legend now keeps its full-size title, metadata and clickable extrema, and fits every colour swatch into the available space above navigation. Short windows show fewer numeric ticks at readable spacing. The over-limit swatch keeps both the true and clamped maxima labelled. Expanding the window restores normal swatch spacing and tick density without changing the result, range or selected band count.

The final feature history changes only three UI source files. As requested in review, ROADMAP.md is unchanged and the one-run QA notes, recipe and report are in this description, with originals preserved outside git. UI source is byte-for-byte identical to the previously reviewed head `252fa873b5ed4cfc1a0e5714f56806b47b9a9f00`.

## Demonstration and provenance

A 32-second native app recording shows the failure, the corrected 960 by 600 layout, clicks on Max (node 23) and Min (node 127), expansion to 1400 by 900, and a return to the smaller window. The original Cantilever sample is a small-strain linear-elastic static solve, with 24 stress bands clamped to 0–10 MPa and an extra over-limit colour. This is a UI resize demonstration, not a physical transient. The true maximum (151.0 MPa) and clamp maximum (10.00 MPa) stay distinct. The small legend is 316 points tall; the larger view restores its natural 507-point height.

The recording and two stills were received through the separate ordinary-Git `artifact-queue` inbox in commit `bfb109cf307cb7109d55e8743276c272fc01322c`. Those verified receipts are scoped to original head `252fa873b5ed4cfc1a0e5714f56806b47b9a9f00`, for `legend-fit-610.mp4`, `after-small.png` and `after-max.png`. Their status is `received`, with file sizes and SHA-256 hashes verified, and `github_attachment: pending_maintainer`. Receipt does not mean GitHub attachment upload is complete.

The original QA report below pins implementation commit `37c9ce78158e3a3a524c99c0dc3b1f38258b0521`; that UI source was published in `252fa873…` and remains unchanged in the new housekeeping head. The existing recording is retained with that original provenance; it is not claimed as a new capture of the rewritten head. There is no media in the feature history.

The native capture used Windows, the Vulkan renderer and an NVIDIA GeForce RTX 4090. All sample geometry and captures are original FieldCraft work. Captions use the repository's Inter font (SIL OFL 1.1). Encoding is H.264, 1400 by 948, 10 fps; smaller app windows are padded without rescaling their pixels.

## Reproduce the recorded case

Run the desktop app with `--sample cantilever --control 7102`. Execute this original recipe through the normal `engine.execute` control method, one command at a time:

```json
[
  {"command":"file.sample","params":{"name":"cantilever"}},
  {"command":"solve.run","params":{}},
  {"command":"results.contour","params":{"field":"von_mises","bands":24,"range":{"min":0,"max":10}}}
]
```

Leave the browser, timeline and Results context panel visible. Set `ui.set` to `{"result_extremes":false}` and `ui.resize` to `{"width":960,"height":600}`. The supported native minimum window leaves a 406-point-tall viewport. The previous legend was 507 points tall and extended behind navigation and the command line, hiding its lowest bands and Max/Min links.

The fixed legend reserves its title, metadata and full-size Max/Min links, then gives the remaining room above navigation to its bands. Every selected colour remains present. Numeric ticks are thinned at readable spacing; the over-limit swatch retains normal height so the true and clamped maxima remain legible. A larger window automatically restores normal band spacing and tick density.

Click each Max/Min line to centre its node and show the probe. Resize to 1400 by 900, then back to 960 by 600, and repeat. The recorded probe nodes are 23 and 127; the range and 24-band selection remain unchanged. Automated regressions also cover a stale-result warning, which takes another metadata line.

<details>
<summary>Original capture layout report, including its original source pin</summary>

```json
{
  "case": "Original Cantilever sample, static von Mises stress",
  "units": "MPa",
  "range": [
    0,
    10
  ],
  "bands": 24,
  "over_limit_colour": true,
  "hardware": "Windows; NVIDIA GeForce RTX 4090, Vulkan viewport",
  "media": {
    "duration_seconds": 32,
    "pixels": [
      1400,
      948
    ],
    "format": "H264, 10 fps",
    "delivery": "GitHub PR attachments pending; binaries preserved outside git"
  },
  "observations": {
    "before": {
      "viewport": [
        250.0,
        126.0,
        401.0,
        406.0
      ],
      "legend": [
        262.0,
        173.0,
        179.53125,
        507.0
      ],
      "navigation": [
        340.5,
        495.0,
        220.0,
        30.0
      ],
      "probe_node": null,
      "bands": 24
    },
    "after-small": {
      "viewport": [
        250.0,
        126.0,
        401.0,
        406.0
      ],
      "legend": [
        262.0,
        173.0,
        179.53125,
        316.0
      ],
      "navigation": [
        340.5,
        495.0,
        220.0,
        30.0
      ],
      "probe_node": null,
      "bands": 24
    },
    "after-max": {
      "viewport": [
        250.0,
        126.0,
        401.0,
        406.0
      ],
      "legend": [
        262.0,
        173.0,
        179.53125,
        316.0
      ],
      "navigation": [
        340.5,
        495.0,
        220.0,
        30.0
      ],
      "probe_node": 23,
      "bands": 24
    },
    "after-min": {
      "viewport": [
        250.0,
        126.0,
        401.0,
        406.0
      ],
      "legend": [
        262.0,
        173.0,
        179.53125,
        316.0
      ],
      "navigation": [
        340.5,
        495.0,
        220.0,
        30.0
      ],
      "probe_node": 127,
      "bands": 24
    },
    "after-large": {
      "viewport": [
        250.0,
        126.0,
        841.0,
        706.0
      ],
      "legend": [
        262.0,
        138.0,
        179.53125,
        507.0
      ],
      "navigation": [
        560.5,
        795.0,
        220.0,
        30.0
      ],
      "probe_node": 127,
      "bands": 24
    },
    "after-small-again": {
      "viewport": [
        250.0,
        126.0,
        401.0,
        406.0
      ],
      "legend": [
        262.0,
        173.0,
        179.53125,
        316.0
      ],
      "navigation": [
        340.5,
        495.0,
        220.0,
        30.0
      ],
      "probe_node": 127,
      "bands": 24
    }
  },
  "implementation_commit": "37c9ce78158e3a3a524c99c0dc3b1f38258b0521",
  "checks": {
    "ui_tests": {
      "passed": 188,
      "failed": 0,
      "ignored": 0,
      "filtered": 0
    },
    "doctests_passed": 3,
    "cargo_check": "passed",
    "clippy_all_targets_deny_warnings": "passed",
    "fmt": "passed"
  }
}
```

</details>

## Validation

- New housekeeping head: `cargo test -p fieldcraft-ui-egui` passed 188 tests and 3 doctests, with no ignored or filtered tests.
- New housekeeping head: `cargo check -p fieldcraft-ui-egui`, `cargo clippy -p fieldcraft-ui-egui --all-targets -- -D warnings`, `cargo fmt --all -- --check` and `git diff --check` passed.
- Comparing the old and new heads gives no diff under `crates/ui-egui`; only the roadmap/QA housekeeping was removed. The native app build and demonstration were performed on the original implementation, not repeated for this housekeeping rewrite.
- Regression coverage checks small/large/small resizing, selection/navigation clearance, unchanged band count and over-limit colour, full-size Max/Min targets, focus/probe behavior, and non-overlapping numeric tick glyphs with and without a stale-result warning.
- Current-head upstream CI at `1d4ed083c09f25cc2027e9cc8a57d03b2637a825` passed: `cargo xtask ci` completed at `2026-10-23T23:42:28Z` (job `120711077592`); dependency checks, bot tests and plan also passed. Heavy tests, oracle, Windows app tests and the macOS oracle were skipped, not passed. Historical CI at the original `252fa873…` head separately passed its main gate rerun, heavy tests and oracle.
- The first old-head CI attempt failed only an unchanged engine cancellation timing check: 5.792319807 seconds exceeded its existing 5-second limit. Its rerun passed at the same old source head; no solver polling, cancellation deadline or test was changed to obtain that result.

## Known limitations

Native sizes below the supported 960 by 600 minimum were not demonstrated. Below the fixed text plus one pixel per band, the legend can still reach navigation; the supported minimum case is covered above. The full workspace gate was not run locally. The already-received media remains applicable because the UI source is unchanged; no duplicate upload is needed. GitHub video/still attachment upload remains pending and is handled by the maintainer.
