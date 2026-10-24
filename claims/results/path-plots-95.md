---
{
  "schema": 1,
  "kind": "claim",
  "action": "claim",
  "request_id": "claim-95-windows-dxc-agent-20261024-1",
  "agent": "windows-dxc-agent",
  "branch": "results/path-plots-95",
  "issue": 95
}
---
Implement the full path-plot contract in #95 after a matching granted receipt and the relayed discussion: real bounded inverse-isoparametric point location and shape-function interpolation, a read-only typed polyline query, null stations across gaps/outside geometry, atomic confined CSV export, actual results-panel plotting and typed/picked points with a viewport path/station overlay. No nearest-node approximation or UI-only partial implementation.

Use a bounded per-result spatial index with conservative higher-order geometry bounds (Bernstein/control-point enclosure for curved tet10/hex20; not nodal AABBs alone), scale-relative Newton residuals, explicit iteration/candidate/work/memory budgets and deterministic shared-face selection. Cover every required solid family (tet4/tet10/wedge6/hex8/hex20), and the existing pyramid5 family if its exact positive-weight map is used; unsupported non-solid kinds are explicitly refused. Preserve existing stale-result, mode/frame and field availability checks; do not present final nonlinear state as an intermediate-frame field.

Verify all required element linear patches, exact nodal values, curved geometry outside the nodal box, distorted/degenerate/outside/hostile cases, holes and mixed meshes, beam-theory top-fibre stress and display-unit/CSV behavior. Add a genuine native solid-temperature path-plot/export demonstration on a frozen tested head, with CPU/Vulkan provenance and scientific limits. Keep media external for the ordinary-Git artifact handoff; no media binaries in feature history, no proprietary/GPL data, no ROADMAP edit. Research primary finite-element and conservative-enclosure sources before implementing the numerical method.

Please relay the complete issue discussion and any current maintainer scope decisions in feedback/issues/95.json. This is a separate source branch from the frozen #99 material-library publication and does not reserve #96 stress linearisation or #205 nonlinear state storage.
