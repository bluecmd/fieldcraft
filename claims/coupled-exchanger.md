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

## Read-only reporting finding for a focused follow-up

Windows RTX 4090 QA of Vulkan-only PR #713 at `337ee655330a251c60e0e4fe6155e800f7c7561f` exposed a pre-existing fluid-reply unit defect. `fluid_summary` returns `final_max_speed` and `max_speed_time` in SI, but the `solve.fluid` reply conversion table in `crates/engine/src/units.rs` maps only `max_speed`. The native heated-flow reply labels velocity as `mm/s` while returning `final_max_speed = 0.0023931903312362536` (m/s); the corresponding value in the declared units is about `2.3931903312362536 mm/s`. The same table omissions exist on mirrored develop `7d9c027`; they are not introduced by #713.

Please relay an existing focused issue for these two missing conversions, or have the maintainer create one, so it can receive a normal claim before implementation. A useful regression would exercise the public command boundary in two display systems, including a non-second time unit, rather than testing only the internal SI summary. No unit-conversion fix is bundled into another agent's approved issue. The immutable Windows QA report and two native stills were transferred through `artifact-queue` commit `9879654`, under `gpu/vulkan-only/337ee655330a251c60e0e4fe6155e800f7c7561f/`, using agent `windows-dxc-agent`; receipt verification is pending. This is runtime evidence only, with no numerical-accuracy, throughput or macOS claim.
