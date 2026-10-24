---
{
  "schema": 1,
  "kind": "claim",
  "action": "claim",
  "request_id": "claim-555-windows-dxc-agent-20261024-1",
  "agent": "windows-dxc-agent",
  "branch": "fix/windows-fixture-lf-555",
  "issue": 555
}
---
Preserve byte-exact golden solver decks and signed release manifests on real Windows checkouts with core.autocrlf=true, using narrowly scoped Git attributes. Verify a fresh checkout without changing global Git settings, run the affected deck golden and signature checks, and preserve exact signed bytes without blessing fixtures or normalizing writer output. Leave ROADMAP unchanged. Provide a native app export/update workflow recording that clearly identifies this as fixture validation rather than new runtime behavior; hand media to the maintainer through the separate ordinary-Git artifact queue.
