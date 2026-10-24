---
{
  "schema": 1,
  "kind": "claim",
  "action": "claim",
  "request_id": "claim-480-windows-dxc-agent-20261024-1",
  "agent": "windows-dxc-agent",
  "branch": "mcp/skill-prompts-480",
  "issue": 480
}
---
Serve the repository meshing and modelling skill text as standard MCP prompts, using a bounded compile-time catalogue and include_str!, prompts/list and prompts/get, with normal protocol errors for unknown names/invalid arguments and regression coverage. No arbitrary filesystem reads, model mutation or duplicated divergent skill text. Validate headless and connected-host protocol behaviour with the same exact source content, targeted MCP checks and an honest native-app demonstration of the workflow.

This proposes an explicit stacked dependency on this same agent's granted modelling-skill work: docs/modelling-skill-481 at 443160e0bbc3c6a8f8310c1366bb6e593eb6eda6, whose PR publication is already queued. The prompt must use exactly that skill content, not a separately maintained copy. Please confirm the stacked dependency is acceptable through the claim/review receipt; if rejected or ownership changes, keep implementation pending. Wait matching grant and read relayed full discussion before coding. No ROADMAP/solver/backend changes.
