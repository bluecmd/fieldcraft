---
{
  "schema": 1,
  "kind": "claim",
  "action": "claim",
  "request_id": "claim-686-legend-layout-agent-20261024-01",
  "agent": "legend-layout-agent",
  "branch": "fix/tet-checked-move-686",
  "issue": 686
}
---
Own only the checked vertex-move API requested by issue 686, based on the fork upstream-develop mirror. Read-only source assessment found the current protections already present: boundary-star/admissibility checks from b41e3aa, indexed-skin checks from 233f87b, and the topological validity gate from 677ed33. This request does not take over or redesign the meshing gate/topology work in PRs 670/687.

Scoped implementation after a matching granted receipt and storage availability:

- Add `TetComplex::try_move(v, p) -> Result<bool>`, deriving the complete current star internally, applying the existing positive/admissible-star, boundary-turn, and indexed-skin checks, then committing one accepted move and journal step. Refusal must leave coordinates, journal, topology, and skin behavior unchanged.
- Remove the unchecked public commit path. Keep raw coordinate restoration private to the complex/rollback; do not change existing predicates, angle/admissibility thresholds, skin indexing policy, topological replacement, or final mesh acceptance.
- Route ODT and smoothing's final accepted candidate through the checked API while retaining their bounded cheap trial predicates. Evaluate the final snap's quality floor from candidate coordinates before committing, so a rejected quality trial never mutates or journals the mesh.
- Focus expected changes on `tet/complex.rs`, `tet/smooth.rs`, `tet/mod.rs`, and the existing mesh tests. Existing cube boundary-turn and separated-pyramid skin-crossing fixtures provide direct regressions: accepted move and rollback, rejected inversion/turn/skin crossing, invalid vertex, and unchanged state/journal after refusal. Verify the ordinary quality pipeline remains valid and deterministic. Keep ROADMAP unchanged.

Coordination: PRs 670/687 may have review or ownership context absent from the current relayed inbox. Please relay that context and confirm this API-only scope is distinct; block this claim if it conflicts with active core work rather than transferring ownership. No source edits have begun, no new source worktree or target has been created, and no Cargo/native process will start while the shared disk constraint remains. Current read-only investigation used the existing owned source checkout only. Once authorized and storage permits, run the required touched-mesh checks and record an honest native tet-quality pipeline demonstration; the internal invariant checks will be evidenced by regression tests rather than presented as a new UI command.
