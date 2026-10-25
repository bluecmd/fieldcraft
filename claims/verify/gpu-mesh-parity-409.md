---
{
  "schema": 1,
  "kind": "claim",
  "action": "claim",
  "request_id": "claim-409-coupled-exchanger-agent-20261025",
  "agent": "coupled-exchanger-agent",
  "branch": "verify/gpu-mesh-parity-409",
  "issue": 409
}
---
Request a bounded verification-only slice of #409, subject to reconciliation of its existing bluecmd assignment. The queue currently has no matching reservation for this branch. Please relay the complete issue discussion and confirm that this slice does not overlap another active owner before granting. This request does not authorize taking over the GPU meshing implementation or closing #409.

At mirrored develop 3751d3bb93676aa49d1a18c438424be88162ac03, crates/mesh-gpu/src/lib.rs documents minimum scaled-Jacobian agreement within 0.02 of the coloured CPU reference. The original sphere, disc and tapered-block agreement test checks repeatability and connectivity but only prints quality and normalized node gaps. Its zipped gap calculation does not independently establish matching node counts or finite coordinates.

After a matching granted receipt and complete discussion, make that existing promise executable: check equal-sized finite meshes and finite quality summaries, preserve repeatability/connectivity checks, and enforce the already documented 0.02 minimum-SJ bound on the original canonical cases. Keep node-gap output diagnostic unless the issue discussion supplies an independently justified bound. Do not tighten tolerances to match one device, change production algorithms, claim scientific mesh convergence, or claim the full million-node/performance scope complete.

Validate the touched mesh-GPU tests and crate checks, formatting, Clippy and applicable quick repository gates. Use an actual RTX4090 with explicit Vulkan and required hardware selection; record missing-device cases honestly. Add no new backend setup/removal and preserve other maintainers' backend paths. Rebase on the latest mirrored develop before publication. A tests-only change does not alter the native meshing UI; any demonstration will retain exact capture provenance and state its limited applicability rather than imply a new algorithm.

No feature branch or application source changes have been started for this request. #699 and #700 remain blocked and are outside this scope.
