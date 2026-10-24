---
{
  "schema": 1,
  "kind": "claim",
  "action": "claim",
  "request_id": "claim-99-windows-dxc-agent-20261024-1",
  "agent": "windows-dxc-agent",
  "branch": "model/material-library-99",
  "issue": 99
}
---
Implement the cited common-metal material library with the nine requested starting entries, a read-only list/family filter, and a typed add command exposed in the material catalogue. Embed original curated JSON for wasm, with every included value carrying its unit, basis, exact primary document/edition/table/page citation, product/temper and temperature applicability. Missing properties remain unavailable; cited yield/tensile values are reported informationally without creating plasticity, damage or failure criteria. Never copy a proprietary material library or commercial CAE output.

The issue predates the SI migration. Current Model stores SI, and DisplaySystem/Quantity already support all five presets and explicit temperature units, including Fahrenheit. Reuse these public boundaries for SI insertion and display-unit reports, rather than adding duplicate conversion factors or restoring obsolete unspecified/imperial-thermal refusals. Please relay the full current discussion and reconcile these older acceptance assumptions with the current unit architecture before implementation. Keep ROADMAP unchanged under the coordinator's explicit instruction.

Verification: every curated property is finite/plausible and cited; nine unique stable IDs; created materials validate; list/filter/add refusal and hostile-parameter cases are bounded and atomic; round trips cover all five presets plus thermal/temperature overrides. Check a structural response against its analytic case and an actual CPU conduction patch using available cited thermal properties, with an honest native app recording. Run affected model/engine checks, clippy, format and wasm, regenerate command documentation, and preserve private source evidence/media outside feature history. Wait for this exact claim's granted receipt before application/data edits.
