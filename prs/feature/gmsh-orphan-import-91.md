---
{
  "schema": 1,
  "kind": "pr",
  "action": "publish",
  "request_id": "pr-91-legend-layout-agent-20261024",
  "agent": "legend-layout-agent",
  "branch": "feature/gmsh-orphan-import-91",
  "issue": 91,
  "head_sha": "dc2b1d43c307b1c18fa728f99ffd2c67313c68e1",
  "base": "develop",
  "draft": false,
  "title": "mesh: import ASCII Gmsh meshes with physical boundary sets",
  "media": "pending"
}
---
## What and why

Closes #91. `mesh.import.orphan` imports ASCII Gmsh MSH 4.1 without CAD. Node and retained element tags stay as labels; physical groups become model sets. The native Orphan Mesh entry, File Open picker, recent files and drag-and-drop reach the same command, with parsing in a background job and one undo step.

The independently written I/O reader follows only the public [Gmsh format and reference-node diagrams](https://gmsh.info/doc/texinfo/gmsh.html#MSH-file-format); no Gmsh implementation, new dependency, commercial-tool file or generated Gmsh fixture is used. It supports rod2/rod3, tri3/tri6, quad4/quad8, tet4/tet10, wedge6, hex8/hex20, including the distinct tet10/hex20 midpoint orderings.

Physical volume groups become element sets. In a volume mesh, matching boundary triangles/quads select existing element faces instead of adding stiffness shells; an internal physical surface deliberately selects both adjacent element faces. Curves select matching element edges and physical points select nodes. Surface-only meshes retain their surface elements and map curves to edges. Unmatched surfaces refuse unless `unmatched: "shell"` explicitly requests intentional shells; unmatched curves still refuse. Physical names are validated, never silently renamed.

MSH coordinates are unitless: their numeric values use the current input length unit, reported as `coordinate_unit`, through the existing unit boundary. Import replaces the whole mesh and geometry classification while preserving the open geometry and non-mesh definitions. Existing geometry/label sets, direct node-specific step limits, or reference-point label collisions refuse before commit: even identical incoming tags do not prove that old constraints still select the same locations. Use a new model or remove those selections first.

Every count is bounded against model/session caps and remaining input before input-sized allocation. Physical membership expansion and queried side matches have global caps. Late/repeated entity tables are resolved deliberately after all sections; duplicate tags, missing entities/nodes, nonfinite coordinates, malformed/truncated sections, unsupported versions/types, binary and partitioned files refuse without partial changes. Ignored metadata sections are reported. Generated `docs/features.md` marks the entry ready; ROADMAP stays maintainer-owned and unchanged.

## Demonstration

A 32.0-second captioned video uses actual native presented frames at exact tested head `dc2b1d43c307b1c18fa728f99ffd2c67313c68e1`, Windows / NVIDIA GeForce RTX 4090 / Vulkan. It shows the import form, the readable quality report, the physical root-face selection, a force applied through the tip set, and a solved result with an actual viewport node probe. All frames show a working workflow. This is a static analysis demonstration; displayed deformation is magnified 250 times and is not a physical transient.

The original 10 × 1 × 1 mm beam is written directly as ASCII MSH: 80 hex20 elements, 621 sparse node labels, and eight boundary quad8 records. The boundary records become four root faces and four tip faces, with 21 nodes at each end; the imported mesh remains 80 solids, with no extra shells. `mesh.quality` certifies all 80 valid, minimum scaled Jacobian 1, volume 10 mm³.

With E = 1000 MPa, Poisson ratio 0, a fixed root and a total downward tip force of 0.001 N, the centre-tip displacement is **-0.004023474075715212 mm** (actual in-app probe node 4259), versus **-0.004024 mm** from the existing Timoshenko cantilever reference scaled to this load. The modest load keeps this demonstration within the linear small-strain assumption.

Video: `gmsh-orphan-import-91.mp4` (H.264, 1280×1080, 10 fps, 32.0 s). Native stills: `after-import.png` and `after-results.png`. Original captures, generated mesh/model, recipe and numeric report remain outside feature history. The three exact files were verified and transferred through ordinary Git in artifact commit `73e82f656bfdbd7c31ff773a46b27da707910a22`, under `objects/feature/gmsh-orphan-import-91/dc2b1d43c307b1c18fa728f99ffd2c67313c68e1/`. Received receipts and native GitHub attachment URLs are pending; `media=pending` until attachment completion is verified.

<details>
<summary>Original mesh generator and reproducible command recipe</summary>

Run this original Python standard-library generator in an empty folder. It writes only a clean-room mesh and command recipe; it never calls Gmsh.

```python
"""Original structured serendipity mesh, written from MSH4.1 public format/node diagrams.
No Gmsh program/source or commercial-product file is used. Coordinates are unitless millimetres.
"""
from pathlib import Path
import json

out=Path(__file__).resolve().parent
points={}
def node(p):
    p=tuple(p)
    if p not in points: points[p]=101+7*len(points)
    return points[p]
def midpoint(a,b): return tuple((x+y)//2 for x,y in zip(a,b))
gmsh_edges=[(0,1),(0,3),(0,4),(1,2),(1,5),(2,3),(2,6),(3,7),(4,5),(4,7),(5,6),(6,7)]
volumes=[]; root=[]; tip=[]
def quad(corners):
    return [node(p) for p in corners]+[node(midpoint(corners[a],corners[b])) for a,b in [(0,1),(1,2),(2,3),(3,0)]]
for i in range(20):
 for j in range(2):
  for k in range(2):
   a,b,c=2*i,2*j,2*k
   corners=[(a,b,c),(a+2,b,c),(a+2,b+2,c),(a,b+2,c),(a,b,c+2),(a+2,b,c+2),(a+2,b+2,c+2),(a,b+2,c+2)]
   nodes=[node(p) for p in corners]+[node(midpoint(corners[x],corners[y])) for x,y in gmsh_edges]
   volumes.append((1000+10*len(volumes),nodes))
   if i==0: root.append((20000+len(root),quad([corners[x] for x in [3,0,4,7]])))
   if i==19: tip.append((20004+len(tip),quad([corners[x] for x in [1,2,6,5]])))
parts=['$MeshFormat','4.1 0 8','$EndMeshFormat','$PhysicalNames','3','3 1 "all"','2 1 "root"','2 2 "tip"','$EndPhysicalNames',
       '$Entities','0 0 2 1','1 0 0 0 0 1 1 1 1 0','2 10 0 0 10 1 1 1 2 0','1 0 0 0 10 1 1 1 1 2 1 2','$EndEntities',
       '$Nodes',f'1 {len(points)} {min(points.values())} {max(points.values())}',f'3 1 0 {len(points)}']
ordered=sorted(points.items(),key=lambda x:x[1])
parts += [str(tag) for _,tag in ordered]
parts += [f'{p[0]/4:g} {p[1]/4:g} {p[2]/4:g}' for p,_ in ordered]
parts += ['$EndNodes','$Elements',f'3 {len(volumes)+len(root)+len(tip)} 1000 20007']
for dim,entity,kind,records in [(3,1,17,volumes),(2,1,16,root),(2,2,16,tip)]:
 parts.append(f'{dim} {entity} {kind} {len(records)}')
 parts += [str(tag)+' '+ ' '.join(map(str,nodes)) for tag,nodes in records]
parts += ['$EndElements']
(out/'cantilever.msh').write_text('\n'.join(parts)+'\n',encoding='ascii')
recipe=[
 {'command':'model.units','params':{'system':'mm-t-s-N-MPa'}},
 {'command':'mesh.import.orphan','params':{'path':'cantilever.msh','unmatched':'refuse'}},
 {'command':'model.material.create','params':{'name':'Elastic','elastic':{'type':'isotropic','young':1000,'poisson':0}}},
 {'command':'model.section.create','params':{'name':'Solid','set':'all','material':'Elastic','kind':{'type':'solid'}}},
 {'command':'model.boundary_condition.create','params':{'name':'Clamp','type':'fixed','target':{'set':'root'}}},
 {'command':'model.load.create','params':{'name':'TipForce','type':'force','target':{'set':'tip'},'vector':[0,0,-0.001]}},
 {'command':'model.step.create','params':{'name':'Bend','procedure':{'type':'static'},'boundary_conditions':['Clamp'],'loads':['TipForce']}},
 {'command':'solve.static','params':{'step':'Bend'}},
 {'command':'solve.probe','params':{'step':'Bend','at':[[10,0.5,0.5]]}},
]
(out/'recipe.json').write_text(json.dumps(recipe,indent=2)+'\n',encoding='utf-8')
print(json.dumps({'nodes':len(points),'hex20':len(volumes),'root_faces':len(root),'tip_faces':len(tip)}))

```

Then run `fieldcraft-cli run recipe.json`, or execute its commands through the native app/control channel. The native recording uses this same input and recipe, followed by `results.contour {"step":"Bend","field":"displacement_z"}` and `results.deformed {"step":"Bend","scale":250}`.

</details>

## Validation

At tested head `dc2b1d43c307b1c18fa728f99ffd2c67313c68e1`, rebased without source changes onto the latest fork `upstream-develop` mirror `7d9c027d71b2c1ea3395e2552e95f0455a7244db`:

- `cargo check` for fieldcraft-io, fieldcraft-engine and fieldcraft-ui-egui: passed.
- Full I/O tests: 54 unit and 18 integration tests passed; one existing measurement ignored; doc tests 0. Nine new tests cover all supported kinds, independently constructed midpoint edges, sparse tags, physical volume/face/edge/point sets, surface-only curves, both sides of an internal interface, late/missing entity tables, parametric coordinates, named refusals, duplicate node/element tags, every-byte truncation, huge/negative counts, invalid tags, nonfinite coordinates, a 10 MiB line and small work caps.
- Full engine tests: 372 passed, 15 existing tests ignored, only the three documented Windows baseline failures below filtered; no additional exclusions. The first run caught a quoted-choice default and stale feature reference; both were corrected and the full final run is green.
- Five focused engine tests prove replacement/definition preservation/one undo step, SI and imperial conversion, unchanged model/journal on refusal (including same-tag collisions), original local-writer roundtrips for all five supported solid kinds with physical end faces and positive quality, and the 80-element imported hex20 cantilever against both the analytical tolerance and existing frozen fixture (difference below 1e-8 at the oracle load). No solver numerics or oracle data changed.
- Full UI tests: 187 unit and 3 doc tests passed, no skips. The new picker test exercises the actual file filter, background import, undo/redo and refused replacement preserving model/journal. Existing hostile-parameter and form tests cover the new command; session import caps are reduced for those tests.
- `cargo clippy -p fieldcraft-io -p fieldcraft-engine -p fieldcraft-ui-egui --all-targets -- -D warnings`, `cargo fmt --all --check`, and `git diff --check`: passed. The final added I/O-only regression also passed its touched-crate clippy/test rerun.
- `cargo build -p fieldcraft`: passed. The external original ASCII fixture/recipe also imported and solved through the CLI. Native screenshots are presented GPU frames, renderer/backend checked; the in-app clicked probe matches the engine probe's node. The app exited cleanly after capture.

Parent source and visual review are completed before publication. Upstream CI is pending. No local full workspace gate, heavy suite, wasm gate or oracle regeneration is claimed.

## Known limitations

ASCII MSH 4.1 only. Binary, partitioned and unsupported element families (including pyramids and complete quad9/hex27) refuse explicitly; ignored postprocessing/periodic metadata is listed rather than applied. The parser uses bounded text/line lengths and model/session resource limits. Boundary matching follows corner labels; no tolerance-based welding, geometry healing or orientation repair is performed.

Replacement intentionally refuses existing mesh-dependent selections rather than silently reusing labels or replacing same-named physical sets. Geometry itself and unrelated definitions are preserved. This is an orphan-mesh reader, not a CAD interchange or general Gmsh physics importer. Units must be chosen deliberately because MSH has no length-unit declaration.

The three filtered Windows baseline failures are `bug_report::tests::a_long_session_and_the_model_are_attachments` (JSON-escaped home redaction), `cmd::file::tests::every_migrated_journal_replays_to_its_migrated_model` (imperial density last bit), and `cmd::render::tests::labels_and_command_lines_read_well` (path separator). They are not hidden or fixed here. Existing ignored tests remain ignored.

The native case is an original analytical beam, not a new full corpus or commercial-tool comparison. Run-specific QA/media stay outside feature history. Native GitHub attachment completion remains pending; no raw-repository media fallback is used.
