---
{
  "schema": 1,
  "kind": "claim",
  "action": "claim",
  "request_id": "claim-653-contour-animation-agent-20261024-cpu",
  "agent": "contour-animation-agent",
  "branch": "cfd/cpu-checkpoint-653",
  "issue": 653
}
---
Implement a bounded first CPU-fluid checkpoint/resume slice, explicitly Refs #653: periodic step-based checkpoints at an explicit confined path and explicit resume with strict model/settings/backend/thread identity. Preserve full dynamic momentum and thermal population buffers, convective outlet history, relaxed fan state, conducting-solid thermal state, absolute step/frame/history counters and steady-monitor windows. Rebuild static topology from verified original model/setup identity; reject unsupported configurations or mismatches rather than approximate them.

Use a versioned, capped, bit-preserving streaming state format with validation before allocation and atomic same-directory replacement, retaining an earlier valid checkpoint if writing fails. Verify interrupted/resumed versus uninterrupted full results bit-for-bit on the same CPU backend/thread count, including steady, thermal/conjugate, outlet and fan histories where supported; reject corrupt/truncated or oversized input and changed settings. Keep solver arithmetic and event ordering unchanged. Include a real native in-app demonstration with honest resolution/time semantics; hand media through the ordinary-Git artifact protocol.

GPU checkpointing, explicit/nonlinear/transient-heat solvers, wall-clock scheduling and automatic discovery remain separate work, so the broad issue stays open. This is a scoped implementation request, not a claim that the entire multi-solver issue is complete. Wait for the matching granted receipt before application coding.
