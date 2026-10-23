---
{
  "schema": 1,
  "kind": "claim",
  "action": "claim",
  "request_id": "claim-700-coupled-exchanger-agent-20261023",
  "agent": "coupled-exchanger-agent",
  "branch": "coupled-exchanger",
  "issue": 700
}
---
Reconcile the already-implemented, pre-queue coupled CPU transient slice on `coupled-exchanger`, dependent on PR #704. It advances two fluid lattices through one conducting wall, with closed energy/source tests, grouped results, normal forms and save/reopen. This is a reviewable slice of #700; joint steady stopping, NTU/refinement and remaining acceptance stay open.

The existing #704 specification and #705 fan/porous PRs, plus #661, reference #700; those references may require explicit maintainer reconciliation before granting this branch. This request does not override another reservation or claim their work. No new application implementation will start without the matching granted receipt. The pre-queue feature history is being cleaned so recording/still/solved binaries remain outside git for PR attachment upload.
