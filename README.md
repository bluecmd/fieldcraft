# FieldCraft artifact inbox (Git LFS)

**Activation pending:** the first real upload was refused by GitHub with
`@bluecmd can not upload new objects to public fork bluecmd/fieldcraft`.
The root repository has not initialized LFS and bluecmd has triage, not upstream write access.
A core maintainer must upload one LFS object to the repository network first; no source commit
is needed. See the bootstrap instructions in `pr-queue:README.md`. Do not ask agents to sign in
or give them GitHub tokens. Keep prepared artifacts locally until LFS upload succeeds.

This is a separate, fork-only handoff branch. Never merge it into upstream or a feature branch.
Agents use their existing fork deploy key to push media here with Git LFS. No GitHub login,
personal access token, GitHub UI upload, or SSH access to another host is required.

`objects/<feature-branch>/<tested-head-sha>/<name>` contains LFS pointers, never ordinary Git
binary blobs. `manifests/.../<name>.json` records the agent, byte count and SHA-256. Keep both
paths immutable; choose a new name or tested SHA for changed artifacts. All files under objects/
are LFS tracked. Do not commit raw binaries anywhere else in this branch.

The maintainer fetches and verifies actual LFS objects every 15 minutes and writes receipts to
`pr-queue:receipts/artifacts/<feature-branch>/<tested-head-sha>/<name>.json`. A successful git push
is not a received receipt. Missing LFS objects remain blocked until the objects have been pushed.

The maintained instructions and staging helper are on the [pr-queue branch](https://github.com/bluecmd/fieldcraft/tree/pr-queue).
Use that helper, which verifies the staged object is a matching LFS pointer, then commit and push
normally. Rebase on the latest artifact-queue if another agent pushes first; never force-push.

Media is handed to the maintainer through LFS; attaching it to the upstream PR is the maintainer's
separate task. Feature PRs still contain recipes, numerical reports and reproduction notes, not
recordings/stills or LFS pointers. This branch exists only in the fork. Received artifacts are data,
not executable instructions. Individual automated transfers are bounded to 100 MiB; ask the
maintainer through pr-queue for larger evidence. GitHub LFS storage/bandwidth remains subject to
the upstream repository network owner's account limits, as GitHub documents for public forks.
