---
{
  "schema": 1,
  "kind": "claim",
  "action": "claim",
  "request_id": "claim-654-contour-animation-agent-20261024",
  "agent": "contour-animation-agent",
  "branch": "diagnostics/solver-errors-654",
  "issue": 654
}
---
Add bounded local diagnostic records for ordinary solve-stopping failures, solver caps and divergent outcomes, separate from panic/crash reports. Retain actual command/parameters, model size, selected step and last available progress, actual backend/adapter/driver/relevant limits, managed process GPU context counts and any real checkpoint path. Explicitly mark unavailable context, especially refusals before GPU allocation; never infer a device or progress.

List/read these as kind solver_error through existing diagnostics commands without adding startup app-crashed prompts. Preserve the original solve error/result if diagnostic persistence fails; do not record malformed command parameters, unrelated validation refusals or deliberate cancellation as solver failures. Nothing uploads automatically or becomes an automatic public attachment.

Respect FIELDCRAFT_CRASH_DIR and existing privacy/retention/confinement boundaries. Advertise written persistent report paths on CLI stderr and MCP replies for headless workers; document persistent container/cloud output. Add focused deterministic small-cap/nonconvergence, report-write-failure and headless path regressions plus a real native intentional-cap-refusal/diagnostics-listing demo. Keep backend selection, solver algorithms, #554 and the maintainer-owned roadmap unchanged; report untestable real device-loss/OOM conditions honestly.

Wait for matching granted receipt before application coding. Use isolated source/target directories, follow current Vulkan-only native policy and hand media to the maintainer through the ordinary-Git artifact protocol.
