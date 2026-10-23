# Optional DXC setup for Windows DX12

This helps only when FieldCraft's status bar shows **Dx12** beside the GPU name. Windows normally
tries Vulkan first; if the status bar shows Vulkan, no setup is needed. The Windows zip does not
include DXC. Installing Microsoft's compiler yourself lets wgpu select it automatically on DX12,
instead of the older FXC compiler.

1. Close FieldCraft and `fieldcraft-cli` before changing files in their installation folder.
2. Download **`dxc_2026_09_29.zip`** from Microsoft's
   [DirectXShaderCompiler v1.9.2609 release](https://github.com/microsoft/DirectXShaderCompiler/releases/tag/v1.9.2609).
3. In PowerShell, verify the downloaded archive:

   ```powershell
   Get-FileHash .\dxc_2026_09_29.zip -Algorithm SHA256
   ```

   The hash must be
   `AD31B1FC8443175D204F77A611FDB3EF2EC42759BDC2F1167368DE24A4A7E7F1`.
   If it differs, do not install that file; download this exact release again.
4. Extract the archive into a separate folder. Copy **only `bin\x64\dxcompiler.dll`** next to
   `fieldcraft.exe` and `fieldcraft-cli.exe` in the FieldCraft installation folder. Do not copy
   `dxil.dll`; it is not needed. No global compiler installation, PATH change or compiler
   environment override is required.
5. Restart FieldCraft and run a GPU fluid solve. On DX12, wgpu's automatic compiler selection
   loads the adjacent DLL. To check its location in PowerShell while the app is running:

   ```powershell
   (Get-Process fieldcraft).Modules |
     Where-Object ModuleName -eq 'dxcompiler.dll' |
     Select-Object FileName
   ```

   The path should name the DLL in your FieldCraft installation folder. The status bar still
   shows Dx12: it identifies the graphics backend, not the compiler. A measured RTX 4090 example
   in [#561](https://github.com/tangency-labs/fieldcraft/issues/561) reduced first-solve pipeline
   creation from about 8 seconds with FXC to about 0.5 seconds with DXC. These are that machine's
   measurements; timing depends on the GPU, driver and workload.

## Updates and removal

The in-app updater replaces only the files listed in the release zip. Because that zip does not
contain `dxcompiler.dll`, it preserves your manually added DLL. It does not update DXC for you.
If you replace the entire installation folder yourself, copy the DLL into the new folder again.

To undo this setup, close FieldCraft and its CLI, then delete the manually added `dxcompiler.dll`.
The next start returns to automatic compiler selection without that local DLL. Vulkan is
unaffected by installing or removing DXC.

DXC is Microsoft's open-source compiler, with NCSA/MIT and other third-party notices in its
[tagged source licence](https://github.com/microsoft/DirectXShaderCompiler/blob/v1.9.2609/LICENSE.TXT)
and release archive. You download it directly from Microsoft; FieldCraft does not redistribute it.
