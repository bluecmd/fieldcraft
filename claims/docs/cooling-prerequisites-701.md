---
{
  "schema": 1,
  "kind": "claim",
  "action": "claim",
  "request_id": "claim-701-contour-animation-agent-20261024-docs",
  "agent": "contour-animation-agent",
  "branch": "docs/cooling-prerequisites-701",
  "issue": 701
}
---
Request a narrowly scoped current cooling-capability and tutorial-prerequisite documentation slice, Refs #701 only. The full four-video series and final 0.7.5 acceptance remain open.

Source inspection on develop 7d9c027 finds `docs/examples.md` global limitations that still describe missing conducting-solid CHT and obsolete collision restrictions, despite merged conduction and regularized collision support. Correct only current global capability/prerequisite statements from the actual source; distinguish existing isothermal air-side samples, conducting-solid workflows, and separately queued paired-fluid work. Retain the historical AIO's exact measurements, source/settings, held-temperature assumptions and original capture provenance unchanged. Link existing original baseline recipes rather than invent new physical results or rerun historical cases.

Explicit remaining dependencies: #700/PR711's coupled reference, joint stopping, refinement/interior pressure checks and budget/export acceptance; #664/PR668 and #665/PR692 LES; #698/PR705 fan/porous integration; #699 buffer splitting; #623 forced-convection validation; #631 parked algorithm/limited accuracy measurement. The proposed documentation does not advertise these unmerged or incomplete capabilities as released, imply coupled/GPU restart from the single-domain CPU slice of #653, or check release-completion boxes.

No solver/model/kernel, numerical-reference, tolerance, sample-physics, performance, full tutorial-series or ROADMAP changes. Preserve truthful physical-transient versus steady-iteration semantics. Provide a short applicable native command/UI clip with exact capture provenance, delivered separately through the artifact inbox. Validate documentation references and any touched command snippets with proportionate checks; no full workspace gate or unnecessary heavy solve.

Wait for a matching granted receipt and read the full relayed issue discussion before source edits. If this slice overlaps an existing maintainer PR/ownership, please reject or explicitly delegate it; it is not authority to bypass that overlap.
