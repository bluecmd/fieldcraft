---
{
  "schema": 1,
  "kind": "pr",
  "action": "publish",
  "request_id": "pr-561-windows-dxc-agent-20261023",
  "agent": "windows-dxc-agent",
  "branch": "windows/dxc-package-561",
  "issue": 561,
  "head_sha": "3145551f7a63c966cdb36bd7e846691061329f1a",
  "base": "develop",
  "draft": false,
  "title": "Windows: ship verified official DXC in the x64 package",
  "media": "pending"
}
---
## What and why

Closes #561. The Windows zip now places the official x64 `dxcompiler.dll` beside both FieldCraft executables, allowing wgpu's existing automatic DX12 compiler selection to use DXC. Packaging pins Microsoft's immutable v1.9.2609 archive URL and SHA-256, extracts only that DLL, and appends the complete MIT/NCSA notices to the Windows package's licence file. `dxil.dll`, SDK tools/headers and the proprietary licence stay out.

A missing input, download failure or invalid cached/supplied checksum stops packaging with recovery instructions. `-DxcArchive` supports offline packaging with the same mandatory checksum. The updater regression verifies installing a previously absent DLL and replacing an old one while preserving user-added files. Backend selection and solver numerics are unchanged.

## Demonstration

A 27.5-second native-window recording shows the extracted package opening the original heated-flow sample, choosing GPU in Solve Fluid, and displaying temperature frames and signed wall heat. The window reports RTX 4090 (Dx12); process inspection confirms the loaded DLL is adjacent to the packaged executable, version 1.9.2609.5 with SHA-256 `0689e2fee0a1c9df8662a59680469f24e2a3ad7ce6bbbcc9284f7a83172bfc59`. The compiler override is unset, DXC directories are removed from PATH, and the working directory contains no DLL.

The sample is a single-air-domain physical transient over a held 80 °C bracket: 5 mm cells, 5,824 total cells, 180 steps and 45 s. It is not a coupled two-fluid exchanger. The recording/stills are preserved outside source git. Transfer through the separate fork-only `artifact-queue` Git LFS inbox is prepared and awaits the required upstream-root LFS initialization; no successful transfer or received receipt is claimed yet. The maintainer retrieves and verifies the bytes, then handles GitHub PR attachments. **Media remains pending until a real attachment URL is supplied.** Agents use only the fork deploy key and git feedback, without GitHub login or UI upload. No videos, raster demo images or LFS pointers are committed to this feature branch. No throughput or pipeline-time improvement is claimed from these runs.

[Reproduction notes](https://github.com/bluecmd/fieldcraft/blob/3145551f7a63c966cdb36bd7e846691061329f1a/docs/windows-dxc.md) and the linked small JSON report pin the source, package hash, compiler, settings and results.

## Validation

- Built both release-profile x64 executables and ran the production Windows packaging script. Download/checksum, GUI icon/PE checks, CLI version check and extraction of the resulting zip passed. No `dxil.dll` or tools are packaged; complete notices are present.
- `packaging/windows/test-dxc.ps1`: real upstream download and verified offline archive passed; missing input and corrupt supplied/cached archive refusals passed.
- Real RTX 4090, Windows 11 Pro build 26200, NVIDIA driver 616.56: all **22 existing GPU verification integration tests pass, no skips**, with DX12, automatic compiler selection and the packaged DLL adjacent to the test executable. Includes CPU/analytic flow and thermal parity, both collisions/layouts, conjugate solids and contacts. Hardware GPU required; no throughput benchmark.
- Updater: 17 tests and doc tests pass with two explicitly excluded existing Windows failures. Both failures were independently reproduced on unchanged develop source: `verify::tests::the_release_key_verifies_the_release_tooling` (signature text fixture) and `stage::tests::app_images_and_packages` (Linux package-command path assertion). The new DLL install/replacement regression passes.
- `cargo clippy -p fieldcraft-update --all-targets -- -D warnings`, `cargo fmt --all -- --check`, `cargo xtask assets` (377 assets attributed), and `git diff --check` pass. Rebased on current upstream develop and repeated affected checks before the tested head was pushed.

## Known limitations

Older Windows 10/driver combinations were not available for testing; this does not establish their unsigned-DXIL compatibility. The standalone release licence attachment still describes the Rust crates; the Windows zip adds the complete DXC notices. The local developer package retains the workspace's 0.7.0 label and is not a published release. No full local CI gate or numerical oracle was run for this packaging-only change.
