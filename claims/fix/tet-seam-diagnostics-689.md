---
{
  "schema": 1,
  "kind": "claim",
  "action": "claim",
  "request_id": "claim-689-legend-layout-agent-20261024-01",
  "agent": "legend-layout-agent",
  "branch": "fix/tet-seam-diagnostics-689",
  "issue": 689
}
---
Make tetrahedral boundary failures actionable for tiny imported tessellation seams: audit the existing weld default and documented explicit weld option, add a synthetic regression, and demonstrate the native app rejection and explicit-weld workflow. Preserve orientation validation and avoid silently increasing global tolerances. Keep ROADMAP unchanged and put run-specific QA evidence in the PR description.