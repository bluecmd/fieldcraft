---
{
  "schema": 1,
  "kind": "pr",
  "action": "publish",
  "request_id": "pr-555-windows-dxc-agent-20261024-1",
  "agent": "windows-dxc-agent",
  "branch": "fix/windows-fixture-lf-555",
  "issue": 555,
  "head_sha": "20a9e0fd58119bbdec9c602f93dd87b9dcc1c5c8",
  "base": "develop",
  "draft": false,
  "title": "Windows: preserve golden decks and signed fixture bytes on checkout",
  "media": "attached"
}
---
## What and why

Closes #555. A Windows checkout with `core.autocrlf=true` changes the golden decks and signed checksum fixtures from their committed LF bytes to CRLF. The unchanged deck writers and signature verifier then correctly reject those altered expectations.

Eight lines of narrowly scoped root `.gitattributes` pin the Abaqus, LS-DYNA and Nastran golden decks to LF and disable Git text transformation for the signed manifest and signature. All 12 fixture blobs remain unchanged. No writer, comparison, signature logic, test, workflow or ROADMAP changes are included; no fixtures were blessed or re-signed. Other files keep their existing checkout policy.

## Demonstration

A 25.9-second native-window recording shows the normal cantilever deck-export workflow for LS-DYNA, Nastran and Abaqus, with full-integration hex8 elements and the static Load step. Its captions explicitly identify this as a Git-fixture correction with unchanged runtime behavior. The three normal exports contain LF and no CRLF (61,074, 49,451 and 16,454 bytes respectively). This is not a recording of the automated tests or a claim of new exporter behavior.

The app runs the unchanged developer executable from the same `7d9c027` application source as the branch base, on Windows 11/RTX 4090 Vulkan; the attributes-only change does not alter it. The root coordinator pushed `windows-fixture-export.mp4`, `lsdyna-export.png` and `nastran-export.png` as ordinary Git blobs in `artifact-queue` commit `f81d645`, with index hashes and byte sizes verified. They are under `objects/fix/windows-fixture-lf-555/20a9e0fd58119bbdec9c602f93dd87b9dcc1c5c8/` with matching manifests. The maintainer's receipts confirm all three artifacts `received`, with SHA-256 hashes and byte sizes matching their manifests; `github_attachment` is now `attached` with the verified native URLs below. No videos, raster images, fixture exports or LFS pointers are committed to this feature branch; private originals and provenance are retained outside source git.

Native GitHub attachments (original capture revision `20a9e0fd58119bbdec9c602f93dd87b9dcc1c5c8`; receipts verify the original SHA-256 and byte sizes):

- [lsdyna-export.png](https://github.com/user-attachments/assets/51961d80-8478-475e-961e-e94dc2d04f57)
- [nastran-export.png](https://github.com/user-attachments/assets/aeb7c022-6842-4e97-acc6-bc9c37bb9afd)
- [windows-fixture-export.mp4](https://github.com/user-attachments/assets/c0862633-a8c5-4078-8131-418674fe9935)

## Validation

- Reproduced before the fix in a fresh independent clone with only that clone's local `core.autocrlf=true`: all 12 fixture files differed from their committed blobs through CRLF; Abaqus/Nastran golden tests, all four LS-DYNA golden tests and the signed-release-key verification test failed.
- Made a second fresh independent checkout of the exact fixed head with local `core.autocrlf=true`, without changing global Git configuration. All 12 files exactly match their unchanged committed SHA-256 values, with LF checkout bytes. `README.md` still checks out as CRLF, confirming the rules are narrow. The signed manifest and signature use `-text`, preserving their committed bytes without text normalization.
- Full `fieldcraft-decks` suite: **69 tests pass**, plus doc tests, in that real Windows checkout. `FIELDCRAFT_BLESS` was unset; both checkouts remain clean.
- Full updater suite: **18 pass and one unrelated existing Windows failure**, `stage::tests::app_images_and_packages` (the Linux package-command path assertion). With that exact exclusion, all 18 tests and doc tests pass. The formerly failing `verify::tests::the_release_key_verifies_the_release_tooling` now passes, including rejection of a changed payload.
- `cargo clippy -p fieldcraft-decks -p fieldcraft-update --all-targets -- -D warnings`, `cargo fmt --all -- --check`, and `git diff --check` pass. Rebased on the maintainer's current `upstream-develop` mirror before pushing the tested head. No full local CI gate or numerical oracle was run for a Git-attributes-only change.

- Relayed CI at 01:00 UTC on the exact requested head: the main `cargo xtask ci` gate and dependency/source/licence checks succeeded. Heavy tests, the oracle, Windows app tests and the macOS oracle were skipped; they are not reported as runtime passes.

## Known limitations

Attributes apply when Git checks files out. An existing worktree that retains old CRLF fixture files may need a fresh checkout of those files after preserving any local edits. This does not change exported decks, model behavior or solver numerics; the native recording demonstrates the existing workflow whose byte-exact expectations the fix protects.
