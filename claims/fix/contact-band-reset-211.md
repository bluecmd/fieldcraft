---
{
  "schema": 1,
  "kind": "claim",
  "action": "claim",
  "request_id": "claim-211-legend-layout-agent-20261024-01",
  "agent": "legend-layout-agent",
  "branch": "fix/contact-band-reset-211",
  "issue": 211
}
---
Request the narrow per-increment lifetime fix expressly proposed by issue 211. Read-only audit on the current upstream-develop mirror confirms that `Run::band` starts at zero, is set to `contact::ENGAGED_BAND` on active-set chattering, flows into subsequent evaluations/tangent predictors, and is never cleared after an accepted increment. The documented lifetime is currently the rest of the step. Near-open nodes then keep a penalty tangent block with zero residual force in later increments.

Intended semantics after a matching granted receipt:

- Keep the existing engaged-band behavior during the increment that chattered, its cutback retries, Newton line searches and augmented-Lagrangian solves. Clear it only after that increment has fully converged and committed, before any later increment's tangent predictor/evaluation. Do not reset it at the start of each retry or midway through augmentation.
- Preserve active-set detection, the band magnitude, chatter threshold, contact forces, multipliers, committed material state, convergence tolerances and iteration/resource caps. This does not implement a held/primal-dual active set or redesign the contact law.
- Establish a meaningful two-increment behavioral regression: the first increment activates the real anti-chatter path (or a controlled internal retry state if a naturally chattering small case cannot be made reliable), retains it through retry/augmentation, then a later near-open state has a force-consistent tangent and normal convergence without inheriting the band. Supplement with a finite-difference check at a near-open contact state and an unaffected no-chatter control. A test of merely assigning zero to a field is insufficient. First reproduce the failure at unchanged baseline; do not publish a speculative clear without behavioral evidence.
- This changes a solver numerical path only after chattering. Run required touched-solver check/tests/clippy/fmt, relevant contact and multi-increment controls, and the required oracle regeneration/fixture comparison under repository rules. Keep existing oracle fixtures unchanged unless an actual reference discrepancy is understood and explicitly coordinated. No local full workspace gate. Coordinate heavy resource intervals and record exact checks/limits rather than asserting numerical parity from the small regression alone.
- Record an actual native Vulkan contact workflow at the final tested head, making any test-only internal state evidence explicit. Keep media/run-specific QA outside feature history, ROADMAP unchanged, and parent coordinates ordinary-Git artifact handoff.

Please relay the full issue discussion and current ownership/review context for the mentioned `contact-folding` work before granting. The present inbox contains only issue 211's index body; this request does not take over any existing active-set implementation, core branch or open PR. Block on overlap rather than transferring another owner's reservation. No application edits, new source worktree or compilation have begun for this claim. The winding certificate and Gmsh import branches remain frozen for their existing PR handoffs.
