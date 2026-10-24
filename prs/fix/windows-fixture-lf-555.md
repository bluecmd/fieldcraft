---
{
  "schema": 1,
  "kind": "pr",
  "action": "publish",
  "request_id": "pr-555-windows-dxc-agent-20261024-1",
  "agent": "windows-dxc-agent",
  "branch": "fix/windows-fixture-lf-555",
  "issue": 555,
  "head_sha": "312f26e2c943994408df83108e265179916899ec",
  "base": "develop",
  "draft": false,
  "title": "Windows: preserve golden decks, geometry names and signed fixture bytes",
  "media": "attached"
}
---
## What and why

Closes #555. A Windows checkout with `core.autocrlf=true` changes byte-exact fixture files from their committed LF bytes to CRLF. Unchanged deck writers, the persistent-geometry-name regression and the signature verifier then correctly reject those altered expectations. The geometry-name failure misleadingly reports zero names gone/new because its diagnostic compares lines while its compatibility assertion compares the full string.

Eleven lines of narrowly scoped root `.gitattributes` pin the Abaqus, LS-DYNA and Nastran golden decks and persistent geometry-name expectations to LF, and disable Git text transformation for the signed manifest and signature. All 13 protected fixture blobs remain unchanged. No writer, comparison, signature logic, test, workflow or ROADMAP changes are included; no fixtures were blessed or re-signed. Other files keep their existing checkout policy.

## Demonstration

The existing 25.9-second native-window recording shows the normal cantilever deck-export workflow for LS-DYNA, Nastran and Abaqus, with full-integration hex8 elements and the static Load step. Its captions explicitly identify this as a Git-fixture correction with unchanged runtime behavior. The three normal exports contain LF and no CRLF (61,074, 49,451 and 16,454 bytes respectively). This is not a recording of automated fixture tests or a claim of new exporter behavior.

This is historical footage from original capture revision `20a9e0fd58119bbdec9c602f93dd87b9dcc1c5c8`, not a new-head QA recording. The subsequent three-line change adds only the geometry-name LF attribute; production app source and the demonstrated export workflow are unchanged. The developer executable has production source equivalent to the `7d9c027` branch base and runs on Windows 11/RTX 4090 Vulkan; it was not rebuilt at the attributes commit. An explicit maintainer applicability review is requested before treating these historical captures as evidence for the updated head; no automatic approval is inferred from this description.

The coordinator transferred the immutable originals in ordinary-Git artifact-queue commit `f81d6455b41d636909c4317764d765d5c2afd74c`, under `objects/fix/windows-fixture-lf-555/20a9e0fd58119bbdec9c602f93dd87b9dcc1c5c8/`. Maintainer receipts verify all three SHA-256 hashes/byte sizes and their actual native GitHub attachment URLs. The original capture revision remains unchanged. No videos, raster images, fixture exports or LFS pointers are committed to feature history; originals and provenance remain external.

Native GitHub attachments (original capture revision `20a9e0fd58119bbdec9c602f93dd87b9dcc1c5c8`):

- [lsdyna-export.png](https://github.com/user-attachments/assets/51961d80-8478-475e-961e-e94dc2d04f57)
- [nastran-export.png](https://github.com/user-attachments/assets/aeb7c022-6842-4e97-acc6-bc9c37bb9afd)
- [windows-fixture-export.mp4](https://github.com/user-attachments/assets/c0862633-a8c5-4078-8131-418674fe9935)

## Validation

- Exact updated head: fresh independent Git clone, only that clone's local `core.autocrlf=true`, clean status, and all 13 protected files match their unchanged baseline Git blobs by SHA-256. The 870-line geometry-name file has 870 LF and zero CRLF. `README.md` still has 169 CRLF, demonstrating the policy remains narrow. Global Git configuration was not changed.
- Geometry-name regression before the follow-up: `cargo test -p fieldcraft-geometry --features scripts --test design_names` fails with zero names gone/new in the actual CRLF checkout. With only the new LF attribute and Git re-checkout, the same executable/test passes. The same command also passes **1/1** from the fresh clean checkout at the updated head. `FIELDCRAFT_BLESS` was unset throughout; golden/test/geometry source and committed fixture bytes are unchanged.
- Earlier deck/signature validation at predecessor `20a9e0fd58119bbdec9c602f93dd87b9dcc1c5c8`: all 12 original fixtures differed through CRLF before their rules, then matched unchanged blobs in a fresh fixed autocrlf checkout. Full `fieldcraft-decks`: **69 tests plus doc tests pass**. Full updater: **18 pass and one unrelated existing Windows failure**, `stage::tests::app_images_and_packages` (Linux package-command path assertion); all 18 and doc tests pass with only that exact test excluded. The signed-release-key test passes, including changed-payload rejection. These checks were not rerun for the later geometry-only attribute; their source and inputs remain unchanged.
- Earlier targeted decks/updater clippy and workspace format checks passed at that predecessor. The final three-line attributes addition passes `git diff --check`; no Rust source changed. Rebase on the current fork `upstream-develop` mirror was a no-op. No full local CI gate or numerical oracle was run for this Git-attributes-only change.
- Relayed main CI was green at predecessor `20a9e0fd`; heavy, oracle and Windows/macOS jobs were skipped. This is historical CI, not a green-check claim for the updated head; updated-head CI is pending publication.

## Known limitations

Attributes apply when Git checks files out. An existing worktree can retain old CRLF fixture bytes even after an attribute change; preserve local edits and use a fresh checkout of affected files. The fresh-clone proof above avoids relying on index timestamps or rewriting expectations. This correction changes neither exports, model behavior nor solver numerics. Historical native media demonstrates the existing export workflow, while the geometry-name fix is validated by the unchanged byte-exact compatibility test and fresh checkout hashes.
