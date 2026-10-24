---
{
  "schema": 1,
  "kind": "claim",
  "action": "claim",
  "request_id": "claim-699-coupled-exchanger-agent-20261024",
  "agent": "coupled-exchanger-agent",
  "branch": "gpu/population-buffers-699",
  "issue": 699
}
---
Implement the native Vulkan population-buffer layout required by #699, a critical dependency of the 0.7.5 cooling workflow. The issue index currently has no assignee, and the queue has no matching reservation; the existing fan/porous and Vulkan-only implementations remain owned by their current branches. Please relay the full issue discussion and reconcile any overlapping open PR before granting this request.

Choose the layout from actual adapter storage/binding limits, with checked addressing and explicit refusal of unsupported layouts. Preserve the existing small-grid path and arithmetic order where possible, and cover both collision schemes and both AA/AB streaming paths, open/obstacle boundaries, thermal fields and conducting solids. Deliberately small test caps will exercise split seams without large CI allocations. Compare against the original layout and CPU using the existing tolerances rather than weakening them.

Account for the actual storage layout, staging/readback, host state and requested result frames. Perform a bounded real-hardware case beyond the former per-buffer cell limit, with exact cells, memory, elapsed time and throughput plus a smaller CPU-reference case. Coordinate device use with the three existing workers. Fan/porous and LES compatibility depends on their owner branches; do not silently claim unmerged integration or duplicate those implementations. Keep original recipes and durable addressing documentation in source; run-specific reports and actual native feature videos use the separate artifact inbox.

The blocked #700 ownership reconciliation remains pending with the user/maintainer. This independent request does not release or override that reservation. No application implementation for #699 will start before a matching granted receipt; the first steps after a grant are reading the full relayed discussion and reviewing the current mirrored source and GPU limits.
