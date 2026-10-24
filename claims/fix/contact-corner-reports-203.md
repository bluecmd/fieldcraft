---
{
  "schema": 1,
  "kind": "claim",
  "action": "claim",
  "request_id": "claim-203-legend-layout-agent-20261024-01",
  "agent": "legend-layout-agent",
  "branch": "fix/contact-corner-reports-203",
  "issue": 203
}
---
Request the bounded reporting correction described by issue 203. The current upstream-develop mirror still constructs each secondary node's reporting normal from every secondary face's full area vector, including faces with no contact overlap at that node. `Interface::result` projects the actual nodal force onto that normal for pressure and normal_force. The existing plane-strain/one-layer C-ring test checks displacements and closed-node count, but not the reported pressure/normal-force mismatch described in the issue.

The same `State::normal` also feeds the friction return map. Therefore this request is explicitly **reporting only**: derive the reporting normal from the actual face-pair overlap supports at the node, preserving the existing physical normal/friction assembly, nodal forces, tangent, multipliers, accepted-state updates, contact detection, convergence settings and all numerical/resource caps. Do not silently change the friction law by replacing its existing normal. Check the associated reported normal/shear force decomposition coherently without changing assembled forces.

After a matching grant and review of the full discussion/ownership context:

- Reproduce the original C-ring reporting mismatch and independently confirm the existing displacement/contact-force agreement. Preserve baseline evidence externally.
- Use the existing overlap/support integration in deterministic face order; no new search, closest-node normal, global heuristic, tolerance or contact algorithm. Verify finite and zero-support behavior conservatively.
- Add a simple original corner fixture showing that a non-overlapping secondary face cannot tilt the reported normal. Extend the real plane-strain versus one-layer solid ring control with mapped pressure/normal-force checks, while proving physical force/displacement parity remains unchanged. Include a flat friction control and thread-count determinism; do not weaken existing tolerances to obtain a pass.
- Run the touched solver checks/tests/clippy/fmt, cheap assets/layers/unit-factors gates and the mandatory numerical-output oracle regeneration. Do not bless or edit frozen physical references/fixtures; if corrected reporting moves a frozen result, relay the measured baseline/new/reference comparison for explicit maintainer review first.
- Commit a small original reusable recipe plus brief reproduction notes; keep run-specific raw QA, saved models and native media external. Record an actual native Vulkan working contact/reporting workflow at the final tested head, with units, static-state interpretation and any deformation scale explicit. The coordinator handles ordinary-Git media transport; no ROADMAP changes.

Please relay the complete issue 203 discussion and any current contact-reporting branch or PR ownership before implementation. The local index says unassigned, and no current local claim or relayed open PR references issue 203; this is not permission to supersede an unseen owner. Issue 189 explicitly depends on amplitudes/multiple-step work and is not claimed; quadratic contact faces (188), the active-set algorithm and nonlinear result-history format (205) are outside this request. Existing #211 source/media remain frozen for their queued publication. No application edits or new source/build tree have begun for issue 203.
