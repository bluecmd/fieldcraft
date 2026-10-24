---
{
  "schema": 1,
  "kind": "pr",
  "action": "publish",
  "request_id": "pr-690-legend-layout-agent-20261024",
  "agent": "legend-layout-agent",
  "branch": "fix/partition-box-tools-690",
  "issue": 690,
  "head_sha": "d79cabfc206be1619af79d56044945def5d49302",
  "base": "develop",
  "draft": false,
  "title": "geometry: keep axis partition tools compact for revolved grooves",
  "media": "pending"
}
---
## What and why

Closes #690. The original 60 × 30 × 60 mm counterbored block with a revolved relief groove now partitions on `yz = 0.3 mm` and `xy = -10 mm`, preserving linked source geometry and named half-cells. Previously both cuts needed a kernel tool nudge and were refused.

Exact coordinate-axis planes now use the kernel box primitive, with bounds derived separately from each body dimension: twice the bounding-box size, giving half an extent of clearance per side. This avoids unnecessarily oversized tools on the thinner axes. Almost-axis and arbitrary planes retain the existing extrusion path; both other normal components must be exactly zero to select the box. Constructor domain limits retain the existing fallback. Every boolean still rejects any diagnostic retry; no plane is moved, no tolerance is enlarged, and the SolidCraft pin is unchanged.

## Demonstration

A 28.0-second captioned native recording shows the original refusal, the unchanged partition form/datum, and successful isolated half-cells for both formerly failing planes and the perpendicular `zx = 20 mm` control. Windows, NVIDIA GeForce RTX 4090, Vulkan; every app frame was actually presented, with no offscreen substitute. The before frame is from base `7d9c027d71b2c1ea3395e2552e95f0455a7244db`; all after frames are from exact head `d79cabfc206be1619af79d56044945def5d49302`.

The source design is generated from the issue's eight public SolidCraft primitive/sketch commands, also embedded in the durable regression. The original recipe and binary design, capture logs, and run reports are preserved outside feature history. In a fresh model with mm display units:

```json
[
  {"command":"geometry.open","params":{"path":"groove.solidcraft"}},
  {"command":"geometry.datum","params":{"name":"D","by":{"type":"offset","plane":"yz","offset":0.3}}},
  {"command":"geometry.partition","params":{"name":"H","datum":"D","bodies":["Block"]}},
  {"command":"geometry.bodies","params":{}}
]
```

| Independent cut on the original design | Unchanged baseline | This head |
|---|---|---|
| yz = 0.3 mm, parallel to groove axis | Refused: tool displacement 5.435255e-4 mm and rotation 2e-6 rad | 2 named cells, clean cut |
| xy = -10 mm, parallel to groove axis | Same refusal | 2 named cells, clean cut |
| zx = 20 mm, perpendicular control | 2 cells | 2 cells |

Video: `partition-box-tools-690.mp4` (H.264, 1280×1098, 10 fps, 269001 bytes). Stills: `after-yz-cut.png` and `after-xy-cut.png`. Ordinary-Git artifact transfer and received receipts are pending; actual GitHub attachment URLs are pending maintainer handling. `media=pending` is intentional.

## Validation

At `d79cabfc206be1619af79d56044945def5d49302`, rebased on the latest fork `upstream-develop` mirror `7d9c027d71b2c1ea3395e2552e95f0455a7244db` (no-op rebase):

- `cargo check -p fieldcraft-geometry --features scripts`: passed.
- Geometry normal tests with `--features scripts`: 65 passed (57 unit, 3 format, 1 persistent-name, 4 void-shell), 1 heavy test ignored; doc tests: 0. The full run initially hit the existing Windows CRLF golden-checkout mismatch; the persistent-name and remaining integration tests then passed using the exact LF Git blob, with no golden content or expectation change. The original checkout line endings were restored afterward.
- The new heavy `partition::tests::a_revolved_groove_partitions_on_parallel_and_perpendicular_planes`, run explicitly with `--ignored --exact`: passed, 65.11 seconds at this head. It uses the original recipe and a 2× scaled copy translated by [120, -70, 45] mm, checks all three planes on both, transformed bounds, two cells, body validity, estimated volume coverage, and exact named cut faces within 1e-9 mm of each plane.
- Fast tests cover both normal directions on every coordinate axis, inherited and paired cut-face names, exact-box volume coverage within 1 ppm, an ordinary oblique plane, and `[1, 1e-12, 0]` without snapping it to an axis.
- `cargo test -p fieldcraft-engine --lib cmd::datum::tests::cuts_stay_on_the_plane_and_risky_ones_are_named -- --exact`: passed, 1 test. Its clean Air cuts remain on-plane and its actual heat-sink pedestal nudge is still refused; the integration test is unchanged.
- `cargo clippy -p fieldcraft-geometry --features scripts --all-targets -- -D warnings`, `cargo fmt --all --check`, and `git diff --check`: passed.
- `cargo build -p fieldcraft` and the native original-design demonstration: passed. No command specification changed, so no command-reference regeneration was needed.

Source review and native visual review found no blocker. Upstream CI is pending; no local full workspace gate, wasm check, or oracle is claimed.

## Known limitations

This fixes the reproduced coordinate-axis groove cuts; other kernel boolean failures and actual retry diagnostics remain refusals. A successful kernel return alone is insufficient: the private bounded investigation observed retries on oversized 2000/20000 mm tools even when they returned bodies, so those results were not accepted as clean cuts.

Volume checks use the kernel's tessellation-based mass estimates, not exact integrals. The unchanged perpendicular control already differs from the original whole by 2.04 ppm; the groove regression allows 10 ppm measurement variation, while exact cut-plane, no-retry, validity, and fast analytic-box checks remain strict. The private old/new plate-tool comparison agreed within 3e-12 mm³ measured and 3e-11 mm³ at a common chord tolerance; its ordinary oblique refusal and very-near-axis result were identical. No runtime volume acceptance or mass-measurement algorithm changed.

The Windows persistent-name golden issue is a checkout line-ending mismatch, not renamed geometry, and has no source fix in this PR. Media attachments still require maintainer URLs. ROADMAP and run-specific QA files are unchanged in feature history.
