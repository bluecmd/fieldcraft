# LES design and fixes — progress

2026-11-04. Worktree `fc-les2`, branch `les-design`, base `upstream/develop`
`4b680571`. Fork remote: `bluecmd/fieldcraft`; upstream: `tangency-labs/fieldcraft`.

## Verdict and completed work

**Re-architect parts; retain the LBM core.** See [LES-DESIGN.md](LES-DESIGN.md)
for the issue-by-issue assessment, literature, target interfaces and staged
acceptance gates. Repair isolated open-boundary failures before interpreting
external drag; require carried wall stress and reliable acceptance before larger
wall/refinement changes. Hybrid RANS–LES and turbulent CHT need additional
transport, wall and coupling contracts.

Read #785, #786–798, #819–820, including the recent #792 inlet-localisation and
#798 non-monotone R9 sphere comments, the prior local audit, and current source.
#795 is already closed/fixed upstream (`b7cbfe0d`); both channel studies use
absolute lag-1 correlation. No duplicate patch is needed.

## Claims and PRs

No fix PR yet. Open-PR list and topic searches found no overlapping outlet,
inlet/slip, wall-stress or halfway-Spalding PR. The candidate issues were all
unassigned and lacked `help wanted`. Following the brief and CONTRIBUTING.md,
requests were posted before claiming:

* [#819: uniform outlet](https://github.com/tangency-labs/fieldcraft/issues/819#issuecomment-6627408534)
* [#820: inlet/slip interaction](https://github.com/tangency-labs/fieldcraft/issues/820#issuecomment-6627408964)
* [#794: refuse the unstable halfway wall variant](https://github.com/tangency-labs/fieldcraft/issues/794#issuecomment-6627574408)

Awaiting maintainer replies/claim eligibility. No production solver changes
have been made. The brief also requires a maintainer response on #785 before a
re-architecture PR. The maintainer merges; do not merge these changes locally
or remotely on their behalf.

The design branch is published to the fork. The concise verdict and roadmap
were [posted on #785](https://github.com/tangency-labs/fieldcraft/issues/785#issuecomment-6627645180)
with a link to the complete document. No maintainer reply was present at the
session's final check.

## Fresh measurements

Original issue Rust reproducers, unchanged except path dependencies pointing at
this checkout. CPU f64, release, builder pods only; `CARGO_INCREMENTAL=0`, own
`CARGO_TARGET_DIR=/work/fc-les2-target`. Each command holds its pod's flock.
Baseline and controls are execution diagnostics, not LES validation.

| Reproduction | Builder | Current develop result |
|---|---|---|
| #819, TRT tau=0.518, uniform pressure outlet, 4×1×1 | sc-builder-2 | Guard at step **1480**, exactly as reported; at step 1400 max density departure 0.0394 and max velocity departure 0.117 |
| #820, TRT tau=0.505, slip sides, seed 1e-6, 16³ | sc-builder-2 | Guard at step **2838**, exactly as reported; inlet density departure 8.9e-4 at step 2500 versus 4.9e-5 beyond x=2 |
| #819 control: change only to convective outlet | sc-builder-1 | **30,000 steps completed**, largest density/velocity departure **6.11e-15** |
| #820 control: change only to periodic side faces | sc-builder-2 | **30,000 steps completed**, final density departure **2.4e-14**; largest transient departure 1.3e-6 |

Local reproducer packages: `.agent-files/uniform-outlet/` and
`.agent-files/inlet-slip/`. Logs: `.agent-files/results/`. These are local-only,
excluded from commits. No GPU allocation requested yet; no solver ran in the
workspace pod and sc-builder-3 was not used.

## Next execution

Coordinator instruction (2026-11-04): while awaiting jyviko's replies on #819,
#820 and #794, keep any further design detail and prototypes local; run any
prototype computation only on the authorised builders under their locks.
The public design's inaccessible local-audit reference has been replaced with
an explicit unpublished-audit note pointing readers to the issue reproductions.
This documentation correction is the only additional publication authorised
for this session. Stop after pushing it; no further prototype work was started.

1. Read the maintainer's reply on #785 and the three claim requests above.
2. After claim eligibility/reply, `/claim` the agreed first issue and wait for
   the bot's assignment. Branch its fix from latest upstream develop in its own
   worktree, with its own target directory. Open the topic's draft PR early,
   with `Closes #N`; do not bundle the design review into a numerical fix.
3. #819 first: diagnose the boundary mode, preserving non-equilibrium stress
   and channel accuracy; #820 separately. #794 is an independent protective
   slice if approved first. The tiny uniform-state control alone does not
   validate a replacement boundary.
4. Focused check/tests, one simplify review, fmt and touched-crate clippy; oracle
   regeneration for solver numerics, features regeneration for command changes.
   Each working commit uses the specified author and ends `Part of #N.`; fold
   fixes into their owning commits. Never run the full local gate.
5. GPU changes require `GPU-REQUEST` and the coordinator's `GPU-READY`, then
   hardware parity in both layouts. Keep /work below 30 GiB and release with
   `GPU-DONE`. Any future run recomputes its work caps.

No milestone has landed, so ROADMAP.md is not marked complete. No new user
decision is needed at present; the next dependency is the maintainer's issue
assignment and design response.

Validation of this documentation step: manual simplify/review pass (no
`/simplify` command/skill installed), `cargo fmt --all`, staged whitespace check,
and tracked-path check for excluded directories. No production crate changed,
so no crate clippy, feature/oracle regeneration or full gate was required. The
two reproduction failures are expected baseline evidence, not passing tests;
both 30,000-step controls exited successfully. All builder commands finished.
