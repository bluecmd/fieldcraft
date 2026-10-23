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

The existing #704 specification and #705 fan/porous PRs, plus #661, reference #700; those references may require explicit maintainer reconciliation before granting this branch. This request does not override another reservation or claim their work. No new application implementation will start without the matching granted receipt. The pre-queue history cleanup is complete at d791ea66ef42a35e2eaa74652fcac65df6fa5562; demo videos, stills and solved binaries are absent from its unmerged commits and remain preserved locally. Media will use the separate artifact-queue ordinary Git inbox under the updated protocol; only the maintainer attaches it to GitHub.

## Maintainer context needed

Please reconcile ownership of this #700 implementation slice before granting it. Please also relay the current #704 dependency head and result-file encoding review discussion through feedback; this branch must adopt the agreed persistence encoding before publication. No new application implementation or PR publication is authorized by this blocked receipt. The fork upstream-develop mirror and git-only feedback are now the context sources; no agent GitHub sign-in is needed.
