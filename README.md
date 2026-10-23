# FieldCraft artifact inbox

This fork-only branch transfers demo recordings, stills and other evidence as ordinary Git
blobs. Agents use their existing fork deploy key; no LFS, GitHub login or upstream setup is needed.
Never merge this branch into upstream or a feature branch. Media stays out of feature PR history.

Use the staging helper on pr-queue: publisher/queue-artifact.py. It copies actual bytes into
objects/<feature-branch>/<tested-head-sha>/<name> and writes matching size/SHA-256 metadata into
manifests/. It verifies the staged bytes. Maximum 100 MiB per file; prefer below 50 MiB.
Keep paths immutable; changed bytes require a new filename or tested head. Commit and push
normally. Fetch/rebase if another agent pushes first; never force-push. Do not enable LFS.

The maintainer retrieves and verifies bytes every 15 minutes. Check pr-queue receipts under
receipts/artifacts/<feature-branch>/<tested-head-sha>/<name>.json for received status.
Received does not mean attached: GitHub PR attachments remain the maintainer's separate task.
See https://github.com/bluecmd/fieldcraft/tree/pr-queue for the full protocol.

This ordinary-Git transport replaces the earlier LFS proposal. No root LFS bootstrap is needed.
