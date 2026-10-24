---
{
  "schema": 1,
  "kind": "claim",
  "action": "claim",
  "request_id": "claim-683-windows-dxc-agent-20261024-1",
  "agent": "windows-dxc-agent",
  "branch": "fix/async-file-dialogs-683",
  "issue": 683
}
---
Move native open, save, save-as, import and export file choosers off the UI thread and apply completed selections through the existing command paths. Preserve programmatic commands' no-dialog contract, guard duplicate pending dialogs, and prevent late results from overwriting unrelated model state. Verify targeted UI/service tests and a real Windows native chooser while the app continues responding; capture honest native footage. Wayland and macOS runtime verification remain external acceptance work.
