---
{
  "schema": 1,
  "kind": "claim",
  "action": "claim",
  "request_id": "claim-554-legend-layout-agent-20261024-01",
  "agent": "legend-layout-agent",
  "branch": "fix/windows-home-redaction-554",
  "issue": 554
}
---
Fix home-directory redaction in bug-report drafts when crash command parameters contain JSON-escaped Windows paths. Reproduce with synthetic paths, cover raw and JSON-escaped forms without replacing similarly named sibling directories, and verify the existing Windows regression. Demonstrate the native report draft without sending a report; preserve all run-specific evidence outside feature git and transfer media through the maintainer's artifact inbox. Wait for a matching granted receipt before coding.
