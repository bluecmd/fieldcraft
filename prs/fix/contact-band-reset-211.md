---
{
  "schema": 1,
  "kind": "pr",
  "action": "publish",
  "request_id": "pr-211-legend-layout-agent-20261024",
  "agent": "legend-layout-agent",
  "branch": "fix/contact-band-reset-211",
  "issue": 211,
  "head_sha": "be2a7c3b4e29b5f5c3319f3b089576ee34fc91aa",
  "base": "develop",
  "draft": false,
  "title": "solver: reset the contact chatter band after its increment commits",
  "media": "attached"
}
---
## What and why

Closes #211. Once contact active-set chatter enabled the temporary engaged band, it remained active for every later load-control increment. A near-open contact then contributed a penalty tangent while its pressure and residual force were zero, spoiling the next predictor and Newton convergence.

The band now clears only after the chattered increment has fully converged, completed all contact augmentations and committed its accepted state. Failed retries and augmentations retain it. The existing accepted-state updates are extracted into a private commit helper without changing their order; the sole numerical change is the band reset at that boundary. Contact forces, multipliers, band magnitude, chatter threshold, iteration/resource caps and convergence tolerances are unchanged. An original reusable command recipe and brief reproduction notes live in `docs/recipes/contact-load-unload.{json,md}`. No new command, dependency, physical reference or ROADMAP change is included.

## Demonstration

<!-- fieldcraft-attachments:start -->
### Attached files

These uploads are complete; earlier pending-upload wording below describes the original handoff.

Capture/source revision: `be2a7c3b4e29b5f5c3319f3b089576ee34fc91aa`.

**contact-band-reset-211.mp4**

https://github.com/user-attachments/assets/9dcf0649-a8c8-48b0-a32a-47b036c30284

**after-open.png**

![after-open.png](https://github.com/user-attachments/assets/b8bebba8-0aff-4812-92b1-a553c2ddd633)

**after-press.png**

![after-press.png](https://github.com/user-attachments/assets/4906b74d-4093-4678-9e69-8e2227d19b61)

<!-- fieldcraft-attachments:end -->

An exact-head **29.0-second native Windows / RTX 4090 / Vulkan** workflow, built in a dedicated per-worktree target, shows the committed original two-block recipe and its two independent nonlinear static steps: a compression check and a load path with factors 0 → 0.5 → -0.05, ending at an open 0.005 mm gap and zero contact pressure. Captions distinguish these static states from a physical transient, state deformation magnifications, and explicitly separate the working native workflow from the controlled internal chatter regression. The video does not claim naturally triggered chatter. It shows the readable load-path settings, compression contour near 9.999 MPa, separated interface and actual node 9 viewport probe at 0.005000 mm. The complete committed 40-command recipe also succeeded through the native control channel. Native presented-frame checks, all captions and full H.264 decode passed; the owned app quit cleanly.

Immutable external originals: `contact-band-reset-211.mp4` (1280×1080, 10 fps), `after-press.png` and `after-open.png`. Parent source/body/native visual review and full video decode passed. All three original SHA-256/byte counts were independently verified and transferred in ordinary-Git artifact commit `0b1c62d89a38f085376213f6e51c156370a2a7b5`, under `objects/fix/contact-band-reset-211/be2a7c3b4e29b5f5c3319f3b089576ee34fc91aa/`. All three received receipts match the immutable originals by SHA-256 and byte count, and all three native GitHub attachments are complete at the exact capture/source head. Their actual URLs and capture provenance are preserved in the maintainer attachment block above.

## Validation

Tested source `be2a7c3b4e29b5f5c3319f3b089576ee34fc91aa`, rebased without changes onto the latest mirrored develop `7d9c027d71b2c1ea3395e2552e95f0455a7244db`:

- Meaningful new two-increment regression covers the no-chatter control and controlled entry into exactly the retry state left by chatter. It exercises an actual failed retry, the real contact augmentation loop, the production accepted commit, and the next tangent predictor/Newton solve. No production test hook is added.
- Before adding the reset, the original accepted-state updates left a near-open tangent/central finite-difference mismatch of 3750 and the next solve cut back at the unchanged 16-iteration cap. After acceptance resets the band, that mismatch is about 6.22e-11 and the next open-state increment converges. The baseline experiment used the extracted but otherwise unchanged accepted-state updates; its logs are preserved externally.
- Full solver tests: 143 passed, no failures or exclusions; seven existing ignored tests remain ignored. `cargo check -p fieldcraft-solver`, touched-crate all-target clippy with warnings denied, fmt and diff checks passed. The complete suite was repeated in the fresh dedicated target at the final head with the same 143 passes, seven existing ignored tests and no failures or filtered tests.
- Mandatory full default `cargo xtask oracle` passed without blessing or editing fixtures before the final documentation-only fold; the solver source is byte-identical to that run. All 82 normally executed cases ran; the existing nightly-only cavity case was checked through its frozen fixture. Regenerated `docs/oracle.md` has no content change: its existing nine known-failure quantities and nine pending cases remain explicit. No existing approximation or skip is represented as a new pass.
- Individually run assets, layers and unit-factors gates passed from the verified current worktree root. No full local workspace gate, workspace clippy or wasm run is claimed. Remote exact-head CI completed successfully: main `cargo xtask ci`, heavy tests, oracle and every other enabled check passed. Windows app tests and the macOS oracle were explicitly skipped; no skipped job is counted as passed.

## Known limitations

This is the narrow per-increment lifetime correction requested by the issue. It does not replace the active-set algorithm or change how chatter is detected. The regression deliberately seeds the existing internal band state for deterministic coverage; the native demonstration is a working contact/loading/unloading example, not a natural-chatter reproduction.

Arc-length contact remains refused by the existing supported-procedure guard, covered by the existing contact refusal integration test; no new arc-length contact semantics are introduced.

The fully unloaded native case can report a relative reaction-balance warning because its absolute residual is about 1.64e-14 N against zero applied load. This is disclosed numerical roundoff; it is not suppressed or addressed by changing tolerances. Run-specific raw QA logs, saved models and media remain outside feature history. The video and both stills are received and attached at the exact-head URLs above.
