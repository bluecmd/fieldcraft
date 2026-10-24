---
{
  "schema": 1,
  "kind": "claim",
  "action": "claim",
  "request_id": "claim-174-contour-animation-agent-20261024-results",
  "agent": "contour-animation-agent",
  "branch": "results/buckling-type-174",
  "issue": 174
}
---
Implement the complete dedicated buckling-result type follow-up: return buckling load factors and normalized mode shapes in their own solver result/mode types and StepResult variant instead of FrequencyResult with dummy frequency, participation and mass fields. Keep frequency results distinct. Preserve the existing numerical eigenproblem, ordering, signs, normalization and supported element scope; this does not implement the separate initial-stress stiffness work in #694.

Carry the distinction through solved-result storage/validation, engine commands and replies, result selection/probes, imperfect-geometry input and native mode display. Handle the old saved-result representation explicitly, with backward-reading regressions rather than silently losing existing buckling results. Keep mode/load-factor quantities dimensionless through the central units API and do not alter frequency units or mass normalization.

Verify with original small buckling and frequency models, exact unchanged numerical outputs, current/legacy result encode/decode and hostile-input tests, command/UI regressions, touched-crate tests/clippy, targeted wasm and the cheap assets/layers/unit-factors/fmt gates. Use isolated source/build directories with bounded jobs, original durable recipe and an actual native Vulkan in-app buckling demonstration; all video/stills remain external and go through the separate artifact handoff. Keep ROADMAP unchanged under the current maintainer policy. Read the full relayed issue discussion and wait for the matching granted receipt before application coding. If ownership or core-team work blocks this claim, preserve the receipt and stop rather than bypass it.
