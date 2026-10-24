---
{
  "schema": 1,
  "kind": "pr",
  "action": "publish",
  "request_id": "pr-685-legend-layout-agent-20261024",
  "agent": "legend-layout-agent",
  "branch": "fix/tet-edge-winding-685",
  "issue": 685,
  "head_sha": "d031a93e6fc024e09ee424c7ccf0c15cf2f30ce7",
  "base": "develop",
  "draft": false,
  "title": "mesh: refuse over-wound tetrahedral edge stars",
  "media": "pending"
}
---
## What and why

Closes #685. A positive tetrahedral star can wrap twice around an edge while every face is held at most twice and its summed volume agrees with the winding-weighted boundary volume. The existing topology, admissibility and volume checks alone therefore accept that local overlap.

The final `tet::linear_gate`, used by both raw CDT output and the quality pipeline, now checks the accumulated dihedral coverage of every edge after all existing checks pass. Interior edges must fill one turn within the stated numerical budget; boundary edges must leave a gap larger than that budget. Refusal names the first canonical offending edge, its location through the existing unit-aware message path, angular coverage, incident tetrahedron count and budget. These are internal mesh vertex IDs, not imported model labels.

The policy is **1e-8 radians per incident tetrahedron**. It is a conservative declared numerical acceptance policy, not a rigorous error bound for every ill-conditioned tetrahedron. A nearly closed boundary gap inside that budget is uncertified and refuses. This tolerance is separate from the existing solver, volume and normalized-Jacobian criteria; none of those changes.

Canonical corner/tetrahedron ordering, sorted edge maps, compensated summation and the existing portable `atan2` keep reduction and first-edge diagnostics deterministic. Work/storage are bounded by six incidences per input tetrahedron, plus canonical sorting; no pairwise global intersection search is added. Existing exact/adaptive predicates, all prior gate checks, topology/move/replace policies, the 30-degree boundary-turn bound and the quality objective remain intact. No command, dependency, oracle fixture or ROADMAP change is needed.

## Demonstration

An exact-head native Vulkan workflow demonstrates that a valid original 10 mm cube still completes surface checking and quality tetrahedralization: **330 valid tet4 elements, 125 nodes, volume 1000 mm³**. The full quality report and mesher quality values match the unchanged baseline cube workflow. The clip shows the source surface, generation settings, solid mesh, wireframe nodes and readable quality report.

This is an internal safety certificate, not a new UI command. The overlapping-star refusal is demonstrated by the independent regression below; the native movie demonstrates the actual working meshing pipeline. Captions make this distinction explicit. Geometry is static and no physical transient, solver result or performance benchmark is claimed.

Native media: `tet-edge-winding-685.mp4` (27.0 seconds, H.264, 1280×1080, 10 fps), `after-mesh.png` and `after-wire.png`. Exact capture source is `d031a93e6fc024e09ee424c7ccf0c15cf2f30ce7`, Windows / NVIDIA GeForce RTX 4090 / Vulkan. Original analytic tagged-cube generator, captures, report, recipe and media remain outside feature history. All three exact files passed original SHA-256/byte-count verification and full video decode, then were transferred in ordinary-Git artifact commit `7f9c9d9c9e080a3651a501a0a85232d7872266e3`, under `objects/fix/tet-edge-winding-685/d031a93e6fc024e09ee424c7ccf0c15cf2f30ce7/`. Received receipts and native GitHub attachment URLs are pending; `media=pending` until actual attachment URLs are verified.

## Validation

At tested head `d031a93e6fc024e09ee424c7ccf0c15cf2f30ce7`, rebased without source changes onto the latest fork `upstream-develop` mirror `7d9c027d71b2c1ea3395e2552e95f0455a7244db`:

- `cargo check -p fieldcraft-mesh`: passed.
- Full `cargo test -p fieldcraft-mesh -- --test-threads=3`: 281 passed, no failures or exclusions; five existing heavy tests and two existing benchmark tests remain ignored. Existing box/sphere/seeded meshing, determinism, skin, detached-tet, face-of-three and boundary-list controls pass. No local full workspace gate or broad heavy suite is claimed.
- Four new focused regressions pass. An original five-sector integer star has positive/admissible tetrahedra and independently passes the actual old face-use, boundary, detachment and volume checks, then refuses for coverage of two full turns. Its cyclic ordering fills one turn and passes.
- A four-sector boundary fan independently passes old checks at gaps 0, 1e-8, 8e-8 and 1e-4 radians. The new check refuses the zero/1e-8 gaps and accepts 8e-8/1e-4, exercising real geometric measurements on both sides of its 4e-8-radian budget. Tests also cover reordered tetrahedra/even corner permutations, scaled/translated geometry, identical first-edge diagnostics, unit-aware locations, invalid indices, nonfinite/out-of-range coordinates and unmeasurable/degenerate input.
- `cargo clippy -p fieldcraft-mesh --all-targets -- -D warnings`, `cargo fmt --all --check`, and `git diff --check`: passed. Review changes were folded into the working layer.
- Native `cargo build -p fieldcraft`: passed. Presented GPU frame capture checks confirm the RTX 4090 Vulkan backend at the exact head, native mesh/quality values match the unchanged baseline, all frames/captions are readable, and the app exited cleanly after capture. Parent source and native visual review passed before publication. Upstream CI is pending.

## Known limitations

This is the local per-edge winding certificate requested by the issue. It does not prove absence of every nonlocal intersection between unrelated mesh regions, and does not implement operation-level swept-face checks or take over the core topology work.

The declared floating-point budget is not an all-conditioning error proof. An otherwise geometrically valid boundary gap inside the budget conservatively refuses rather than being treated as a distinguishable gap. Nonfinite/zero fallback dihedral measurements also refuse. Existing exact/adaptive admissibility checks run first and keep their original failure report.

No solver numerics or oracle fixtures changed. The native evidence is an original analytic cube, not a new complete corpus or performance study. Run-specific QA and binary media stay outside the feature branch; native GitHub attachment completion remains pending, with no raw-repository-video fallback.
