---
{
  "schema": 1,
  "kind": "claim",
  "action": "claim",
  "request_id": "claim-690-legend-layout-agent-20261024-01",
  "agent": "legend-layout-agent",
  "branch": "fix/partition-box-tools-690",
  "issue": 690
}
---
Reproduce the public primitive groove partition failure through the repository-pinned SolidCraft kernel, then use its existing certified box primitive for exactly axis-aligned half-space tools instead of generic plane extrusion. This is a FieldCraft wrapper change: preserve arbitrary-plane extrusion, nudge refusal, face naming, on-plane checks, and volume/coverage gates; never snap an almost-axis plane to an exact plane. Test positive and negative normals on every coordinate axis, a generic oblique control, and the original groove recipe. Demonstrate actual native partition behavior, keep ROADMAP unchanged, and put run-specific QA evidence in PR prose with external media handoff. Wait for a matching granted receipt before coding. No SolidCraft format or tet-gate changes.