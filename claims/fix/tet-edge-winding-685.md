---
{
  "schema": 1,
  "kind": "claim",
  "action": "claim",
  "request_id": "claim-685-legend-layout-agent-20261024-01",
  "agent": "legend-layout-agent",
  "branch": "fix/tet-edge-winding-685",
  "issue": 685
}
---
Request only the per-edge winding certificate in `tet::linear_gate` described by issue 685, based on the current fork upstream-develop mirror. The read-only audit found that the gate already checks face multiplicities, the listed boundary, detached tetrahedra, element admissibility and volume, but does not sum the angular coverage around each edge. A winding-weighted volume can therefore agree despite an over-wound edge.

After a matching granted receipt and ownership reconciliation:

- Add a bounded deterministic edge-coverage check to the final linear tet gate. Accumulate canonical sorted edges in a fixed order, distinguish interior edges from edges on the already-validated boundary, and document the numerical angle/accumulation tolerance. Reuse the existing deterministic measure helpers where appropriate; do not introduce platform-dependent trigonometric decisions, new dependencies, or unbounded geometry searches.
- Preserve the current topology, face-use, boundary, detachment, admissibility and volume checks. Refusal should name the stable first offending edge and its location through the existing structured unit-aware error path. Treat invalid indices, nonfinite and degenerate measures conservatively; do not silently accept an unmeasurable edge.
- Add an original small over-wound ring regression that the current face/volume checks accept, plus valid interior and boundary rings, near-limit/degenerate controls, canonical ordering and translated/scaled controls. Run the touched mesh crate's required tests/check/clippy/fmt and appropriate existing pipeline controls. Any slow corpus case remains in the existing heavy workflow rather than expanding the fast gate.
- Keep `TetComplex` move/replace policy, exact predicates, the 30-degree turn limit, quality objective, solver numerics, global meshing tolerances and ROADMAP unchanged. This does not implement issue 684's swept-face guards or redesign core topology/gate work.
- Record an actual native Vulkan tet-quality workflow with the tested head. Describe the new internal certificate as regression-backed, not a new UI command or a claim that a local per-edge test proves all possible global nonadjacent intersections. Keep original media and run-specific QA outside feature history; parent coordinates ordinary-Git artifact transport.

Coordination requirement: the inbox currently provides only issue 685/684 index bodies, without their full discussions or current PR 670/687 ownership/review context. Please relay those discussions and confirm this narrowly additive final-gate check is distinct from active core work before granting. Block this claim if PR 670 or another owner already implements it; this request does not transfer their ownership. Issue 684 was deliberately deferred because its operation-level swept-simplex policy overlaps that work more broadly. No application source edits, new source worktree or compilation have started for this request; the existing Gmsh import branch remains frozen for publication.
