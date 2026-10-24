---
{
  "schema": 1,
  "kind": "pr",
  "action": "publish",
  "request_id": "pr-686-legend-layout-agent-20261024",
  "agent": "legend-layout-agent",
  "branch": "fix/tet-checked-move-686",
  "issue": 686,
  "head_sha": "3a24ec965f1d19830af8af3505054ae7ea63a972",
  "base": "develop",
  "draft": false,
  "title": "mesh: check vertex moves before committing coordinates and journal",
  "media": "attached"
}
---
## What and why

Closes #686. A tet-quality caller can no longer commit a vertex move while omitting a validity check. `TetComplex::try_move(v, p)` derives the complete current star internally, checks the existing positivity/admissibility, boundary-turn and indexed-skin conditions, then commits one coordinate and one journal step. Refused candidates and valid no-ops change neither mesh, journal nor skin index; invalid or unused vertex IDs are errors.

ODT and optimization smoothing retain their bounded cheap trial predicates and commit final candidates through the checked API. The final geometry snap now evaluates the candidate's quality floor from coordinates before committing. On the unchanged baseline, a quality-rejected snap restored the coordinates but recorded two spurious `Moved` steps; the regression now leaves the entire state unchanged. The unchecked raw move is private and used only by rollback. Existing predicates, the 30-degree boundary-turn limit, quality objective, skin-index policy, topology checks and final validity gate are unchanged.

## Demonstration

<!-- fieldcraft-attachments:start -->
### Attached files

These uploads are complete; earlier pending-upload wording below describes the original handoff.

Capture/source revision: `3a24ec965f1d19830af8af3505054ae7ea63a972`.

**tet-checked-move-686.mp4**

https://github.com/user-attachments/assets/b8be72b8-e651-4971-9446-250dfa532a1a

**after-mesh.png**

![after-mesh.png](https://github.com/user-attachments/assets/54eee328-cbf9-44ee-b706-52d01a47d095)

**after-wire.png**

![after-wire.png](https://github.com/user-attachments/assets/f89185dd-f97c-4ff0-961e-b6d4821f3112)

<!-- fieldcraft-attachments:end -->

A 27.0-second captioned recording uses actual native presented frames on Windows, NVIDIA GeForce RTX 4090, Vulkan. It shows the same 10 mm cube input, the 2.5 mm tet form, the readable quality panel, and wireframe/node views. This is a static meshing demonstration, not a physical transient. The before binary is `d79cabfc206be1619af79d56044945def5d49302`, whose entire `crates/mesh` source is byte-identical to base `7d9c027d71b2c1ea3395e2552e95f0455a7244db`; all after frames are from tested head `3a24ec965f1d19830af8af3505054ae7ea63a972`.

Both runs produced 330 valid tet4, 125 nodes and volume 1000.0000000000005 mm³. Their complete quality responses match, including minimum scaled Jacobian 0.3866983804965625, minimum/maximum dihedral angles 40.127831718721545° / 121.31482476311642°, 1027 ODT moves, 109 smoothing improvements, and 13 topological improvements. No stage stopped early.

This is a demonstration of the existing meshing workflow through the changed internal commit boundary. Rejection, full-star coverage, journal behavior and rollback are evidenced directly by regression tests; the recording does not present an internal library API as a new UI command.

The independently generated input is a closed 10 mm cube with six face tags, 24 duplicated face-corner records and 12 outward triangles, packaged as 3MF. In mm display units:

```json
[
  {"command":"geometry.import.mesh","params":{"path":"cube.3mf"}},
  {"command":"mesh.surface.check","params":{"surface":0}},
  {"command":"mesh.tet.generate","params":{"surface":0,"size":2.5,"quality":true}},
  {"command":"mesh.quality","params":{}}
]
```

<details>
<summary>Original native-demo input generator (Python standard library)</summary>

```python
from pathlib import Path
import zipfile
out=Path('.')
faces=[[(0,0,0),(0,1,0),(1,1,0),(1,0,0)],[(0,0,1),(1,0,1),(1,1,1),(0,1,1)],[(0,0,0),(1,0,0),(1,0,1),(0,0,1)],[(0,1,0),(0,1,1),(1,1,1),(1,1,0)],[(0,0,0),(0,0,1),(0,1,1),(0,1,0)],[(1,0,0),(1,1,0),(1,1,1),(1,0,1)]]
pts=[];tris=[]
for f,quad in enumerate(faces):
 b=len(pts);pts += [(x*10,y*10,z*10) for x,y,z in quad];tris += [(b,b+1,b+2,f),(b,b+2,b+3,f)]
vertices=''.join(f'<vertex x="{x}" y="{y}" z="{z}"/>' for x,y,z in pts)
triangles=''.join(f'<triangle v1="{a}" v2="{b}" v3="{c}" ft:face="{f}"/>' for a,b,c,f in tris)
model=f'<model unit="millimeter" xmlns="http://schemas.microsoft.com/3dmanufacturing/core/2015/02" xmlns:ft="https://github.com/bherbruck/solidcraft/formats/3mf-face-tags/2026/10"><resources><object id="1" name="checked-move-cube" ft:faces="6"><mesh><vertices>{vertices}</vertices><triangles>{triangles}</triangles></mesh></object></resources><build><item objectid="1"/></build></model>'
with zipfile.ZipFile(out/'cube.3mf','w',zipfile.ZIP_DEFLATED) as z:z.writestr('3D/3dmodel.model',model)
print('Original closed10mm cube generated with6tagged sides/12triangles')
```

</details>

Video: `tet-checked-move-686.mp4` (H.264, 1280×1098, 10 fps, 559128 bytes). Stills: `after-mesh.png` and `after-wire.png`. Original captures, the input generator and 3MF, and full before/after replies remain outside feature history. All three originals were transferred through ordinary Git in artifact commit `a19b6203948a942649a21d5f6ea131da55d49238`, with index byte counts and SHA-256 values verified. All three artifact receipts are received and their native GitHub attachments are verified above; `media=attached`.

## Validation

At `3a24ec965f1d19830af8af3505054ae7ea63a972`, based on the latest fork `upstream-develop` mirror `7d9c027d71b2c1ea3395e2552e95f0455a7244db` (no-op rebase):

- Baseline `cargo test -p fieldcraft-mesh --lib -- --test-threads=3`: 277 passed, 5 existing heavy tests ignored. The new snap regression then failed on that unchanged production code, specifically because refusal added two journal steps.
- `cargo check -p fieldcraft-mesh`: passed.
- `cargo test -p fieldcraft-mesh -- --test-threads=3`: 280 unit tests passed, 5 existing heavy library tests and 2 existing integration benchmarks ignored, 0 failures or filtered tests; doc tests: 0.
- All 24 targeted tet tests passed, including input-order determinism and the ordinary quality pipeline. New coverage checks a partial-star trap, valid no-op, invalid/unused IDs, unchanged whole state on refusal (including skin/journal), one accepted journal step with rollback, and the quality-rejected snap. Existing boundary-turn and skin-crossing fixtures now exercise the checked commit API.
- `cargo clippy -p fieldcraft-mesh --all-targets -- -D warnings`, `cargo fmt --all --check`, and `git diff --check`: passed.
- `cargo build -p fieldcraft`: passed. The native comparison asserted identical node/element counts, volume, complete stage statistics and the full `mesh.quality` reply, and every screenshot was a presented frame. The app exited cleanly after both captures.

Source review found no blocker. No command specification or solver numerics changed; no command-reference or oracle regeneration was required. Relayed exact-head CI reports success for `cargo xtask ci`, the heavy suite, the oracle, and all other enabled jobs. The macOS oracle and Windows app jobs were skipped. No local full workspace gate, heavy suite, wasm check, or oracle is claimed.

## Known limitations

`TetComplex` is a public library API: downstream callers of the unchecked `move_point` or separate `move_is_valid` must migrate to `try_move`. A checked move validates the current complex; callers still use the existing boundary projection/freedom rules when choosing a candidate. No new meshing algorithm, automatic geometry repair, tolerance increase or topology policy is introduced.

The native comparison covers one original analytic cube and is integration evidence, not a new full corpus/refinement validation or a solver benchmark. Run-specific captures, recipe generator, reports and logs remain outside feature history; ROADMAP is unchanged. The video/stills have been received, attached, and verified through their exact-head receipts; the native attachment block preserves the original capture provenance.
