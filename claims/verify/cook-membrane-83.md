---
{
  "schema": 1,
  "kind": "claim",
  "action": "claim",
  "request_id": "claim-83-windows-dxc-agent-20261024-1",
  "agent": "windows-dxc-agent",
  "branch": "verify/cook-membrane-83",
  "issue": 83
}
---
Own the finite-strain Cook membrane verification in #83 after a matching granted receipt and complete relayed discussion. Independently transcribe a primary published reference with its exact constitutive energy, material constants, total force versus traction, plane-strain/thickness conditions, location, table or figure, mesh and precision. Build an original recipe, perform a bounded refinement study, justify the tolerance from the source and convergence, then activate the oracle or retain an honest known failure. Never invent a value, widen a tolerance to fit, or bless a fixture before a justified reference comparison. No commercial-code or GPL-family output, example deck or source.

The proposed 2021 Schroeder/Wick/Reese et al. primary paper is accessible, but its two Cook strain-energy functions differ from the currently implemented decoupled neo-Hookean model. I will not treat those tables as an equivalent benchmark or silently add a constitutive law. First locate a primary reference matching the existing energy and boundary conditions; report an actual model mismatch if none is available. Please relay the full issue discussion and any maintainer decision about the intended reference/variant to feedback/issues/83.json.

Resource plan: isolated source worktree and owned target, three build jobs; sequential CPU solves starting at 4, 8, 16 and 32 divisions, considering 64 only if necessary and feasible. Bound each solve's degrees of freedom, factor storage and nonlinear iterations before running; record actual costs and stop on a cap rather than exhausting memory. Keep expensive convergence runs out of the fast gate. Preserve every unrelated oracle fixture; only bless this case after an independently justified match, and regenerate the affected reference documentation as required. No ROADMAP edit.

Provide genuine exact-head native Windows/Vulkan footage of the verified nonlinear case, explicitly separating solver physical increments from playback and stating remaining reference/discretization limitations. Keep videos, stills and generated projects outside feature history for the coordinator's ordinary-Git artifact handoff. This claim does not reserve #174, #96, #205, or the coupled cooling issues.
