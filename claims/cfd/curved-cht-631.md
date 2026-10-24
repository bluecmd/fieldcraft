---
{
  "schema": 1,
  "kind": "claim",
  "action": "claim",
  "request_id": "claim-631-contour-animation-agent-20261024",
  "agent": "contour-animation-agent",
  "branch": "cfd/curved-cht-631",
  "issue": 631
}
---
Measure conjugate convective heat transfer at an independently constructed curved conducting interface on three grids against a primary published reference. Separate solid conduction, interface geometry, finite-domain effects and reference assumptions; an isothermal wall or empirical correlation alone will not stand in for a conducting-solid validation.

Scope is measurement only, following release tracker #697: the curved-wall algorithm change remains parked. Begin with the existing conjugate tests and scalar interface formulation, add an enduring cited benchmark and report three-grid accuracy, uncertainty and any measured staircase limitation. Do not implement or unpark cut cells, interpolated interfaces or other true-surface production scalar-flux algorithms. Keep #631 open with Refs #631, since the parked algorithm scope is not completed by a measurement.

Preserve existing fixtures, follow clean-room and verification rules, and keep the maintainer-owned roadmap unchanged. Use normal engine commands for a real in-app demonstration; hand compact media through the separate ordinary-Git artifact inbox and report exact physical-time or steady-iteration semantics, grids and hardware. Wait for the matching granted receipt before application coding.
