# LES progress — 2026-11-06

Verdict: retain the LBM core; repair boundaries and validity, then selected
architecture changes. The public les-design branch has the slice 8 proposal and
the unpublished-audit link correction. Slices 5–7 belong to codex-les3.
Never merge these PRs ourselves. No wall-layer PR/push until jyviko agrees on #785.

## Existing PRs

#826 received CHANGES_REQUESTED at 21:15Z; the sole citation move is now pushed
at `f9f37d006959dd3c318f241a8a0ab95628adcd11`, awaiting renewed review and CI.
#827 is APPROVED at its unchanged head. **Do not push to #827 before merge.**
#832 remains open/ready; its existing scope question is still recorded below.

- **#826 / #819**, head `f9f37d00`: original failure at step 1480 now survives
  30,000 steps at roundoff. CPU 192-case max departure 2.45e-14; GB10 both
  layouts, 64 cases at 30k, max 2.98e-7. Controls/oracle pass at the prior reviewed head. Citation moved from meshing
  literature to solver literature Part C, C3.3a beside Zou–He. Folded into the
  original commit and rebased onto current upstream/develop `6b613217`; solver
  and test sources are identical to prior head `3c268c4c`. `cargo fmt --all
  --check` passed on sc-builder-2 under its flock; diff checks passed. No solver
  reruns for this documentation-only review correction. New CI is running.
  Reply: https://github.com/tangency-labs/fieldcraft/pull/826#issuecomment-6696013537
  Release-note acknowledgement: all existing uniform-pressure-outlet models
  change numerically; TRT outlet-adjacent error .047% → .057%, RR 3.2% → .23%;
  oracle unchanged.
- **#827 / #794**, head `78c79791`: rejects unstable halfway wall before backend
  allocation/callbacks. Historical sweep: 40 on-node cases decay, 40 halfway
  cases grow. Focused tests, oracle and final GB10 controls pass. Approved by jyviko;
  no push or branch change made in this review turn.
  Reply: https://github.com/tangency-labs/fieldcraft/pull/827#issuecomment-6695993104
  **After merge, separate follow-up PR:** describe current behaviour in the
  user-facing error/manual without history or issue numbers, and remove the
  `FIELDCRAFT_HAN_WALL=halfway` option instead of documenting it.
- **#832 / #820**, head `45d3ac6994529e56c114e3f12e791e1dc35660f1`, worktree
  fc-les2-820 / branch fix/820-inlet-slip. TRT inlet stress-trace extrapolation
  and fourth-order filtering of density residual. Original guard at 2838 now
  survives 30k; final density 7.84e-9, velocity 2.27e-9. Nine CPU cases, eight
  controls, GB10 both layouts/eight cases and 96 parity cases pass. CI 41926505284.
  Sphere survives 120 D/U but lift RMS .01958 fails .001 target; Cd 1.15780.
  Tau .505/.507/.518 pass; .5005 with Smag .1 grows. Scope confirmation pending:
  https://github.com/tangency-labs/fieldcraft/pull/832#issuecomment-6644180092

If #826 lands before #832, reconcile pressure/inlet scratch deliberately: #826
uses 20 floats; #832 uses 16 with residuals 14/15. Reserve >=22, move residuals,
update native/shader strides and memory accounting; rerun both regressions.
No active GPU lease. Fresh GPU-REQUEST is required before any further GPU work.

## Slice 8 local prototype

Worktree `/home/agent/work/fc-les2-wall`, branch `prototype/786-stress-wall`,
base upstream/develop `6b613217`, current local commit
`f96b5fbd03ffa4ef539690ffe8e28a4d6c3f1654` (previous `ae710001`). Not pushed.
Existing Han on-node traction with Musker sampled at an explicit physical height;
interpolation retained. No gradient/physical-viscosity coupling implemented yet.

Mechanism matrix: 32 short cases; 48 longer/reversed cases. N10/20/30,
TRT/RR/MRT/HRR, all axes/signs, h=.20/.25, rest/refusals. Requested/applied gap
<=2.45e-15; independent momentum budget <=3.71e-14 of momentum; mass drift
<=2.86e-13. These are mechanism controls, not turbulent drag acceptance.

Proposal: https://github.com/tangency-labs/fieldcraft/issues/785#issuecomment-6651971536
Await jyviko's reply. Current results confirm the existing need for measured
wall-gradient/viscosity coupling; no proposal change or new #785 comment.

## Current follow-up

TRT N20: original .1 lattice setting amplifies the small seed, while bulk stays
near .097 and final peak reaches .222. Original Han and held-bulk controls also
grow; no-seed TRT and HRR stay smoother. At physical t=.4, peak physical speeds
44.758/44.705/44.559 for settings .1/.08/.04 show amplification persists under
timestep reduction. At .04, 10,000 steps/t=1 complete with every-step maximum
Mach **.177432194**. This remedies the bounded control's Mach envelope only;
it neither identifies a bulk eigenmode nor validates the fluctuations.

Full channel: nominal Re_tau 950, 63x21x63, wall separation 2, periodic 6.3x6.3,
nu=1/950, MRT, Cs=.08, h=.20, Dean interior-cell-bulk controller. Requested,
applied and independent momentum-carry stress are measured separately.
Speed .065 / tau=.5001053; burn 20, then 25 complete ~2-unit blocks through 70.
Completed all 209,965 steps through t=70.0035, including 25 post-burn blocks.
Carried Cf=.005780313 versus Dean=.005262521 (**+9.83925%; 5% target fails**).
Requested/applied/carried mean stress 1.09760621; achieved Re_tau 995.2837.
Force 2SE=.2671%, lag1=.113858, half drift=.0422%; maximum absolute lag through 8
is .3843. Doubling block length changes SE by −9.04% on the same first 24 blocks.
Global Mach .214822 / averaging Mach .213109: the .2 envelope also fails.
Density departure <=.007163. Independent budget residual <=1.323e-12 of wall
impulse, requested/applied gap <=1.221e-15. Accurate transfer does not establish
accurate drag. Approximate wall-law bulk integration gives Cf=.006008899 versus
Dean=.005288095 (+13.63069%); it is a sensitivity calculation, not validated
physical bulk quadrature. Keep both conventions visible.

Predeclared h=.25 pilot completed t=10: last block Cf=.005652059 versus
Dean=.005262427 (+7.40403%); maximum Mach .216166. Startup only, no acceptance
claim. No extension planned. Speed .1/h=.20 pilot also ended t=10; +5.3568% in
its last block, maximum Mach .287418. Do not infer converged timestep bias.

Checks: manual simplification, fmt, all-target clippy -D warnings and normal
lbm tests pass (184 passed, 29 ignored). Controlled-drive checkpoint continuation
and 1/4-thread diagnostic sampling pass bitwise. No full gate/oracle/GPU run in
this follow-up; source is clean. Acceleration bits are saved as checkpoint
metadata before restoring Flow; no production checkpoint format changed.

## Completed / handover

All follow-up solve and archive jobs are complete. Only sc-builder-2 was used,
under `/home/agent/work/.sc-builder-2.lock`; no GPU requested/used. No active job
or lock is held by this worker. Prototype source is clean and committed locally.
Documentation is local only; do not push it or the prototype before agreement.

Permanent evidence: `/home/agent/work/fc-les2-wall-results/2026-11-05-channel/`:
RESULTS.md, final raw logs, statistics.json, blocks.csv, and full checkpoint
snapshots with extracted metadata. Earlier detailed progress is archived there.
Frozen binary remains `/work/fc-les2-wall-channel-frozen`, SHA256
`c152e968831d9b05e1542ccb797efe25584dba3285e0ae3b717bd1d5038fea20`.
Builder checkpoints remain in `/work/fc-les2-wall-channel-mrt065` and its `-h25`
variant. Own target `/work/fc-les2-wall-target`. Reproduce/resume under the flock
using the exact settings and checkpoint signature documented in RESULTS.md.

Await #826 re-review/CI, #827 merge before its separate follow-up, jyviko's wall
direction and #832 scope reply; keep all three PRs responsive.
The local evidence leaves the proposal unchanged, so no #785 comment was posted.
Next wall work remains gradient/viscosity coupling and fixed-height refinement,
with full drag/profile/Mach and eventual real-GPU acceptance still required.
