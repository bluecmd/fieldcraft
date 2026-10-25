---
{
  "schema": 1,
  "kind": "claim",
  "action": "claim",
  "request_id": "claim-733-coupled-exchanger-agent-20261025",
  "agent": "coupled-exchanger-agent",
  "branch": "fix/fluid-final-speed-units-733",
  "issue": 733
}
---
Fix the missing central display-unit conversion for final_max_speed and max_speed_time in solve.fluid replies, shared by CPU and GPU. Add a focused regression proving peak and final speed both convert through the existing quantity system, with time converted as Time. Keep solver state and numerical algorithms unchanged.

Please relay the complete issue discussion and reconcile any overlapping ownership before granting. No application edits will begin without a matching granted receipt. The issue index currently reports no assignee. This request does not override the blocked #699/#700 claims or the separate #712 backend-policy discussion.

Validate touched engine tests/check/clippy and the scoped repository gates in an isolated worktree/target. Reproduce on a bounded original flow case with Vulkan hardware and prepare an actual native feature recording showing consistent displayed speed values; preserve captions, recipe, exact revision and units. Media handoff will use the current shared exchange guidance and maintainer publication protocol without direct GitHub uploads. No new solver accuracy, backend support or heat-exchanger completion claim.
