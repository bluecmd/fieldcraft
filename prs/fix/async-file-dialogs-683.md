---
{
  "schema": 1,
  "kind": "pr",
  "action": "publish",
  "request_id": "pr-683-windows-dxc-agent-20261024-1",
  "agent": "windows-dxc-agent",
  "branch": "fix/async-file-dialogs-683",
  "issue": 683,
  "head_sha": "b485880d04f0005232e9e24fc7f5cfbd79f12e3f",
  "base": "develop",
  "draft": false,
  "title": "Desktop: keep native file choosers off the event loop",
  "media": "pending"
}
---
## What and why

Closes #683. Native Open, Save, Save As, embedded-model save, import, deck export and CSV export currently enter a blocking OS file dialog from the app event loop. The desktop host now returns an `rfd::AsyncFileDialog` future. The app polls only a pending chooser and schedules repaint while it is pending, including the first poll when a toolbar action creates the chooser after frame logic. Idle frames do not poll a chooser or gain a periodic repaint.

Only one chooser can be pending. A selection is discarded if the model, current document identity, path or revision changes, or if a background job has started. New/Open/sample/update continuations run only after the selected save actually succeeds; cancellation or a write failure keeps the current document. Existing explicit-path/programmatic commands retain their normal no-dialog result contract. The browser retains its existing picker behavior.

`ui.inspect` exposes `file_picker` so control clients can distinguish a pending host chooser from app forms; the control protocol documentation is updated. There are no dependency, solver, ROADMAP or GPU-backend changes.

## Demonstration

A 24-second Windows native evidence clip shows the genuine OS Save As window, then seven selected, actually presented app frames while that same chooser is pending and the camera changes. The chooser is captured separately because it is an OS window; an explicitly marked opaque redaction covers only the private folder/search breadcrumb. Its type/Save/Cancel controls are original pixels. The app frames are not presented as screenshots of the chooser.

On Windows 11 with an NVIDIA GeForce RTX 4090 Vulkan viewport, eight bounded `ui.inspect` calls returned while `file_picker` was true; the slowest roundtrip was 0.0110964 seconds with a two-second timeout. The app rendered all seven camera frames through the native presented-frame capture path. This is responsiveness evidence, not a throughput benchmark. At the unchanged base `7d9c027`, the same real Save As interaction caused two two-second control timeouts and responded again after native Cancel.

The executable was built after the exact requested commit. Its SHA-256 is `d4ed882e0b0510135fa8106015b9dd5d779f67bb0a73d4e26ff08274a5398e27`; the original 315-node/160-hex model remained byte-identical after the demonstration. The owned app and chooser were closed through `app.quit` after capture.

`async-native-file-chooser.mp4`, `native-chooser.png` and `responsive-viewport.png` were transferred as ordinary Git blobs by the coordinator in artifact-queue commit `c453484333dd6fecd51ac64396409d69cc647b51`, under `objects/fix/async-file-dialogs-683/b485880d04f0005232e9e24fc7f5cfbd79f12e3f/`. Index SHA-256 hashes and byte sizes were verified before the normal push. The maintainer's receipts confirm all three artifacts received, with SHA-256 hashes and byte counts matching their manifests; `github_attachment` remains `pending_maintainer`. Actual GitHub attachments remain pending. Private logs, unredacted chooser capture and the original capture/encoding recipe are retained externally. No demonstration binaries, raster images or LFS pointers are in the feature history.

## Validation

- `cargo test -p fieldcraft-ui-egui -p fieldcraft -p fieldcraft-web`: 195 UI tests, 8 desktop tests, 3 web tests and 3 UI doctests pass.
- Nine added deferred-chooser regressions cover pending frames, duplicate invocation, a toolbar-created chooser's first repaint, save/open/import, stale model/path/document results, background jobs, cancellation and write failure, deck/CSV naming and parameters, save-before-open, and save-before-update. Existing explicit-path/programmatic tests also assert that the host chooser is never called.
- `cargo clippy -p fieldcraft-ui-egui -p fieldcraft -p fieldcraft-web --all-targets -- -D warnings` passes.
- `cargo check -p fieldcraft-web -p fieldcraft-ui-egui --target wasm32-unknown-unknown` passes.
- `cargo fmt --all -- --check` and `git diff --check` pass. Native `cargo build -p fieldcraft` passes at the requested commit. Fresh fork `upstream-develop` fetch/rebase was a no-op before the feature push.
- Real native Save As pending responsiveness and presented viewport frames were checked as described above. No full local CI gate, numerical oracle, sustained GPU solve or performance study was run for this UI change.

## Known limitations

Windows native pending-dialog responsiveness is verified. Wayland/Hyprland and macOS runtime behavior have not been tested here; this does not claim reproduction or resolution of a compositor-specific warning.

The Windows input helper could capture the genuine chooser but returned `failed to activate captured window` after the permitted fresh-window retry. No unsupported native-input workaround was used. Consequently the new native chooser's Save/Cancel completion was not manually exercised in this run: successful selection, cancellation, failed write and guarded continuations are covered by the deferred-host unit tests, and those are explicitly distinct from native manual validation. The final capture closes the owned app through its control API rather than claiming a native Cancel result.

Media remains pending until the maintainer provides actual GitHub attachment URLs. An artifact transfer or received receipt is not a GitHub attachment.
