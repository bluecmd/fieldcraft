# FieldCraft agent handoff queue

This branch is an operational inbox in **bluecmd/fieldcraft**, not application source. The backup
maintainer scans it every 15 minutes. Agents submit issue claims here first, wait for a granted
receipt, do the work in their feature branch, then submit a PR description here for publication.

Agents have **only a fork deploy key**. They must not log into GitHub, install `gh` credentials,
read Actions logs through the website, or upload PR attachments themselves. Only the maintainer
runs authenticated GitHub operations. Everything agents need is exchanged through git and LFS.

Target: **tangency-labs/fieldcraft:develop**. Feature branches live in **bluecmd/fieldcraft**.
No queue action merges a PR, closes an issue, changes CI settings, or executes feature-branch code.
Posting a PR is not a code review or a claim that CI or release acceptance is complete.

## 1. Ask the maintainer to claim an issue

Choose a stable agent ID (for example `gpu-zones-agent`) and a feature branch. Use one primary
implementation issue per branch. Refer to larger parent issues in the PR body; do not reserve an
entire epic for a small child task.

Commit **claims/$branch.md** on `pr-queue`. A branch containing slashes keeps those subdirectories:
`feature/ports` uses `claims/feature/ports.md`. Copy [CLAIM_TEMPLATE.example](CLAIM_TEMPLATE.example).
The metadata is JSON between `---` lines, followed by a short plan in Markdown:

```markdown
---
{
  "schema": 1,
  "kind": "claim",
  "action": "claim",
  "request_id": "claim-698-gpu-zones-agent-20261023",
  "agent": "gpu-zones-agent",
  "branch": "gpu-fan-porous",
  "issue": 698
}
---
Port the existing CPU fan/porous scheme to the GPU, with CPU parity and hardware checks.
```

**Wait for `receipts/claims/$branch.json` to say `granted` for your agent and issue before coding.**
A queue commit alone is not a claim. The publisher assigns the GitHub issue to **bluecmd**, adds
`claimed`, and comments with your agent/branch. The receipt distinguishes agents sharing that
GitHub account. Do not independently `/claim`, assign issues, or remove another agent's claim.

Existing assignments, a different agent's reservation, another open PR covering the issue, and
core-team/paused work block automatic claims. An existing bluecmd assignment without a matching
receipt requires reconciliation, except when an existing PR already establishes this same branch's
ownership. A handoff from core-team work must be handled explicitly by the maintainer; metadata
claiming permission is not an override. A `granting` receipt reserves the issue during an API
interruption but does **not** authorize starting. Read the error and wait for `granted`.

## 2. Ask the maintainer to post your PR

Push your feature branch to the fork, rebase on the maintainer’s `upstream-develop` mirror in the fork, and run the checks
appropriate to the change. Then commit **prs/$branch.md** on `pr-queue`, using
[PR_TEMPLATE.example](PR_TEMPLATE.example):

```markdown
---
{
  "schema": 1,
  "kind": "pr",
  "action": "publish",
  "request_id": "pr-698-gpu-zones-agent-20261023",
  "agent": "gpu-zones-agent",
  "branch": "gpu-fan-porous",
  "issue": 698,
  "head_sha": "0123456789abcdef0123456789abcdef01234567",
  "base": "develop",
  "draft": false,
  "title": "CFD: run fan curves and porous resistance on the GPU",
  "media": "pending"
}
---
## What and why

Closes #698. Describe the final change and its user-visible behavior.

## Demonstration

Describe the actual app behavior. Push the recording/still through the separate artifact-queue LFS handoff; the maintainer handles GitHub attachments.

## Validation

List exact checks/results and relevant hardware. Say which checks were not run.

## Known limitations

Record existing test failures, approximations, incomplete acceptance cases and remaining issues.
```

`head_sha` is the full **tested feature-branch commit**, not the queue commit. The publisher
verifies it twice before publishing; a moved head is blocked until the agent reruns affected
checks and updates the request. Do not push feature changes while publication is in progress.
The body after the metadata is posted literally. Use `Refs #N` for issues that must stay open;
use `Closes #N` only for work actually completed. Preserve known failures and hardware limits.

PRs are non-draft unless `draft` is explicitly true. Normal upstream CI triggers when the PR is
created; the publisher never bypasses approval or asserts that queued/skipped checks passed.
An existing open PR for the same branch/base is updated rather than duplicated. A closed or
merged PR is recorded, never automatically reopened or replaced. Draft readiness changes require
an explicit manual action. Changes to an open PR's title/body use the same file/request ID with
the new tested SHA and content.

Read **receipts/prs/$branch.json** for `posted`, the verified head, and the PR URL. A `blocked`
receipt explains what to fix. Blocked requests are retried on later scans, including when an
external condition changes. Posted, unchanged requests are not reposted. Archive/delete your
request files after completion if desired; retain the receipts.

## Media policy: GitHub attachments, not git

The maintainer's [request on #704](https://github.com/tangency-labs/fieldcraft/pull/704#issuecomment-6372535663)
applies to new handoffs: **do not commit demonstration videos or stills**. Removing them in a later
commit is insufficient: amend/rebase the unmerged commits so the binaries are absent from the
branch history. Agents hand recordings to the maintainer through the separate LFS inbox below;
only the maintainer performs GitHub PR attachment upload. Keep
small original recipes, numerical report JSON, settings, hardware and reproduction notes in git.
Remove attribution rows for binaries removed from the repository; preserve provenance in the
reproduction notes. Existing assets already in upstream are outside this queue's cleanup scope.

The publisher inspects every unmerged commit (bounded to 50) and refuses new video files or
raster demo images under docs/tools/demo-video. Large changes outside those bounds require a
manual handoff. There is no raw-repository-video fallback.

`media` has three values:

- `pending`: post the PR with an honest textual demonstration and a pending attachment receipt.
  The agent transfers recordings/stills through the LFS artifact inbox below. The maintainer
  retrieves the files and handles GitHub attachment upload. Agents never need GitHub sign-in.
  `pending` does not block PR creation, and a received artifact does not mean it is attached.
- `attached`: the maintainer has supplied real GitHub attachment URLs; the body contains them (`github.com/user-attachments/...` or
  `user-images.githubusercontent.com/...`). Use this when the PR already exists and the upload is done.
- `not_needed`: explain when a visual demonstration is not applicable. This is not an assertion
  that a required UI review or screenshot has been completed.

Clips must identify physical transients versus accelerated steady iterations, assumptions and
resolution. A single air-side example must not be described as a coupled two-fluid exchanger.

## 3. Release a reservation

When abandoning a branch with no open PR, change its claim file to `"action": "release"`, retaining
the **original** request ID, agent, branch and issue. Wait for `released`. An open PR keeps its
reservation; coordinate withdrawal first. Closed primary issues retire reservations as `completed`.
No automatic inactivity expiry is imposed by this queue. Do not reuse another agent's reservation.
If the repository's own claim sweep or a maintainer changes the assignment, reconcile it before
publication; the publisher rechecks assignment and will block the PR.

## Sharing pr-queue without losing anyone's work

Use a separate worktree for queue edits; never merge `pr-queue` into a feature branch or upstream.
Use the existing SSH deploy-key remote named `fork` (`git@github.com:bluecmd/fieldcraft.git`,
or your configured SSH host alias), never an HTTPS login. Only edit your own `claims/...` and `prs/...` files. The publisher owns `receipts/`; maintainers
own this protocol and `publisher/`. Do not commit credentials, private filesystem paths or media.

```sh
git fetch fork pr-queue
git worktree add -b queue/YOUR-AGENT /tmp/fieldcraft-queue-YOUR-AGENT FETCH_HEAD
# In that worktree, edit only your own request file, then:
git add claims/YOUR-BRANCH.md             # or prs/YOUR-BRANCH.md
git commit -m 'queue: request issue claim for YOUR-AGENT'
git push fork HEAD:pr-queue
```

If the push is rejected, fetch `pr-queue` again and rebase onto its new `FETCH_HEAD`, resolve only
your own file, and retry. **Never force-push `pr-queue`**, reset it, delete another agent's files,
or overwrite receipts. A request is ready as soon as it is pushed; no chat ping is required.

## Git-only feedback and CI logs

The maintainer copies upstream context into `pr-queue` every scan. Agents fetch it with their fork
key, not GitHub API credentials:

```sh
git fetch fork pr-queue upstream-develop
git show fork/pr-queue:feedback/issues/index.json
git show fork/pr-queue:feedback/prs/fix/legend-fit-610.json
```

`feedback/issues/<issue>.json` contains claimed/requested issue details and discussion.
`feedback/prs/<branch>.json` contains the PR head, comments, reviews, inline review comments and
check status. Failed completed jobs include a `log` path under `feedback/logs/`; use `git show
fork/pr-queue:<that-path>` to read it. The maintainer downloads completed job logs even when the
rest of the workflow is still running. Logs over 2 MiB are explicitly truncated with a notice.
Agents should request missing context in their queue handoff instead of attempting a login.

`upstream-develop` in the fork is a fast-forward-only mirror of upstream develop. Agents fetch and
rebase onto `fork/upstream-develop`; they need no upstream credentials. No force update or change
to the fork’s normal develop branch is performed.

## Artifact transfer: separate Git LFS inbox

Recordings, screenshots and other binary evidence go to **bluecmd/fieldcraft:artifact-queue**,
not a feature branch or pr-queue. Its `objects/**` files are Git LFS pointers; small manifests
remain normal JSON. Never merge this branch upstream. Use the same fork deploy key, with no new
SSH server access, GitHub login or token. Install Git LFS on the agent machine if necessary.

**Current activation blocker:** a real upload returned `@bluecmd can not upload new objects to
public fork bluecmd/fieldcraft`. GitHub requires an existing LFS object in the repository network
or root write permission. The maintainer must initialize it once as described below. Do not
claim files are transferred based on a pointer commit or failed push. Keep the originals locally.
The branch and client are ready; wait for a successful LFS push and `received` receipt.

In a separate worktree based on artifact-queue:

```sh
git fetch fork artifact-queue
git worktree add -b artifact/YOUR-AGENT /tmp/fieldcraft-artifacts-YOUR-AGENT fork/artifact-queue
cd /tmp/fieldcraft-artifacts-YOUR-AGENT
git lfs install --local --skip-smudge
python /path/to/pr-queue/publisher/queue-artifact.py /path/to/recording.mp4 \
  --agent YOUR-AGENT --branch YOUR-FEATURE-BRANCH --head-sha FULL_TESTED_SHA \
  --description 'Actual app capture; assumptions and reproduction are in the feature docs'
git commit -m 'artifacts: hand off recording for YOUR-FEATURE-BRANCH'
git push fork HEAD:artifact-queue
```

The staging helper validates that Git’s index contains an LFS pointer whose SHA-256 and size match
the actual file. It stages the object and its manifest, not the source path. The normal LFS
pre-push hook uploads the bytes through GitHub using the existing deploy key. Never skip the LFS
push/check, put base64 media in Markdown, or put full binaries in ordinary git blobs. On a race,
fetch/rebase artifact-queue and retry; never force-push it. The staging helper accepts at most
100 MiB per file; request larger transfers through the maintainer.

The maintainer pulls **actual LFS bytes**, verifies size/hash and retains them outside source git.
`pr-queue:receipts/artifacts/<branch>/<head-sha>/<filename>.json` says `received` only after that
verification. `github_attachment: pending_maintainer` means the file is in the maintainer’s hands,
not yet a GitHub PR attachment. The maintainer handles that separate step and provides the real
URL. `receipts/artifact-errors/` records missing objects or retrieval problems without asking an
agent to sign in. Small recipes/reports/provenance still belong in the feature PR.

### One-time root LFS initialization (core maintainer only)

[GitHub’s documented rule](https://docs.github.com/en/repositories/working-with-files/managing-large-files/collaboration-with-git-large-file-storage#pushing-large-files-to-forks)
explains the observed rejection. LFS storage/bandwidth for public forks is charged to the parent
repository network. A core maintainer with upstream write permission can run the following in a
clone whose `origin` is tangency-labs/fieldcraft, using **their own existing authentication**:

```sh
git remote get-url origin  # verify tangency-labs/fieldcraft
git lfs install --local
queue_bootstrap_oid=$(printf 'FieldCraft artifact transport bootstrap v1\n' | \
  git lfs clean --filename queue-bootstrap.txt | sed -n 's/^oid sha256://p')
git lfs push --object-id origin "$queue_bootstrap_oid"
```

This uploads one tiny object without a source file, staged change, branch push or merge. Once
it succeeds, retry the fork artifact upload and verify a received receipt. Our backup-maintainer
account has only upstream triage access and cannot perform this root-write step. No upstream
credential should be given to an agent. Until a real transfer succeeds, artifact transport is
not declared operational.

## Publisher operation

Source: [publisher/publish.mjs](publisher/publish.mjs), tests:
`node --test publisher/publish.test.mjs publisher/handoff.test.mjs`. The maintainer service uses
Node, git/Git LFS and authenticated `gh`; agents use only git/Git LFS and their deploy key. The
staging helper uses Python’s standard library.
Only the installed, reviewed copy runs; committing replacement executable instructions to a
request does not execute anything. The scanner uses GitHub APIs, not branch checkouts or shell
commands from request bodies. It authenticates as bluecmd and targets only the fixed repositories.

The user timer is `fieldcraft-pr-queue.timer` on the backup maintainer's Linux host, every quarter
hour (`:00`, `:15`, `:30`, `:45`, Europe/Stockholm). It uses a local lock, persists missed timer
activations and runs independently of this chat. The host must be powered on and online with
working GitHub authentication. Persistent does not replay every missed interval. No model session
or automatic code review is implied: this is the mechanical claim/publication handoff.

```sh
systemctl --user status fieldcraft-pr-queue.timer
systemctl --user start fieldcraft-pr-queue.service       # scan now
journalctl --user -u fieldcraft-pr-queue.service -n 60
systemctl --user disable --now fieldcraft-pr-queue.timer # stop future scans
```

If a tool session cannot reach the user bus, use `systemctl --user --machine=bluecmd@.host`
for the status/start/disable commands above. This host route was verified during installation;
do not restart the shared user manager or other agents’ services to work around a stale socket.

Receipts are the shared result; the journal records operational failures. No empty polling commits
are created. Network/API interruptions retain reservations, fail visibly, and retry next time.
No merges, branch deletion, arbitrary recipients or non-GitHub posting are authorized by a request.
