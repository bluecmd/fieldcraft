---
{
  "schema": 1,
  "kind": "pr",
  "action": "publish",
  "request_id": "pr-561-windows-dxc-agent-20261023",
  "agent": "windows-dxc-agent",
  "branch": "windows/dxc-package-561",
  "issue": 561,
  "head_sha": "2ab2a59e96b27f3fd9e4b046c97e29d57d8933b4",
  "base": "develop",
  "draft": false,
  "title": "Windows: document optional manual DXC setup for DX12",
  "media": "pending"
}
---
## What and why

Closes #561. Windows users whose status bar shows Dx12 now have a short guide for optionally installing Microsoft's DXC themselves. Vulkan users need no setup. The guide links the official v1.9.2609 archive, gives its SHA-256, explains copying only the x64 `dxcompiler.dll` beside the executables, and shows how to confirm the loaded module. It also explains updater retention, manual updates and removal. DXC is downloaded by the user from Microsoft; FieldCraft does not redistribute it.

The Windows zip remains unchanged. This PR contains only `docs/windows-dxc.md` and links from the GPU performance documentation and Windows release notes. No packaging, runtime, updater, licence-policy, attribution or ROADMAP change remains in the feature history. The approximate 8-second/0.5-second pipeline times are explicitly attributed to the measured RTX 4090 example in #561, not asserted as new or universal results.

## Demonstration

A fresh 27.5-second native-window recording, `manual-windows-dxc.mp4`, shows the original heated-flow sample solving on RTX 4090 (Dx12), temperature frames and signed wall heat after **manual** DXC installation. Its captions state that the standard Windows zip is unchanged and that the verified Microsoft DLL was added by the user.

The demo uses a freshly extracted original seven-file Windows package containing no DXC. The downloaded archive matched `AD31B1FC8443175D204F77A611FDB3EF2EC42759BDC2F1167368DE24A4A7E7F1`; only `bin/x64/dxcompiler.dll` was manually copied into the installation folder. Module inspection confirmed that adjacent user-added DLL loaded, with the compiler override unset and DXC directories excluded from PATH. The app reports GPU execution on RTX 4090. Removing the DLL restored the original seven-file layout, and the app restarted successfully afterward.

The sample is a single-air-domain physical transient over a held 80 °C bracket: 5 mm cells, 5,824 total cells, 180 steps and 45 s. It is not a coupled two-fluid exchanger. The fresh video and `manual-temperature.png` / `manual-wall-heat.png` are prepared outside source git for the separate ordinary-Git `artifact-queue` handoff at this new tested head. **Media remains pending until the maintainer supplies real GitHub attachment URLs.** Earlier bundled-package artifacts at head `3145551` were received but are immutable, superseded evidence and must not be attached as the demonstration of this revised PR.

## Validation

- Reviewed the three-file docs-only diff, archive/hash/module PowerShell instructions, local documentation target and release-note link. `git diff --check` passes. No code or package changes remain.
- Ran the unchanged production packaging script with the existing release-profile binaries; verified the zip has seven original files and no `dxcompiler.dll` or `dxil.dll` before manual setup.
- Followed manual installation on Windows 11 Pro build 26200, RTX 4090 driver 616.56; verified the user-added module path and hash, completed the native GPU/thermal sample, then removed the DLL and restarted. No new throughput or pipeline-time measurement is claimed.
- Verified updater retention with the existing Windows file-by-file mock updater fixture temporarily naming the user-added file `dxcompiler.dll`: one targeted test passed, source restored afterward. No updater code or test changes are included.
- Rebased onto the maintainer's current `upstream-develop` mirror before the exact-lease rewrite of this owned feature branch. No full local gates were rerun for documentation-only changes. Earlier CI results and bundled-package verification belong to the superseded head and are not asserted for this new head.

## Known limitations

Manual setup is optional and specific to DX12; Vulkan users need nothing. Timing varies with hardware, driver and workload. Older Windows 10/driver combinations were unavailable for this manual smoke test. The local app used the current unchanged developer binaries labelled 0.7.0; this is not a published release. Artifact receipt and GitHub attachment remain separate maintainer steps; no GitHub login or UI upload is required of agents.
