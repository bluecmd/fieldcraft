#!/usr/bin/env python3
"""Stage an ordinary Git artifact and its manifest. Uses git only; no GitHub login or token."""
import argparse
import datetime
import hashlib
import json
import pathlib
import re
import shutil
import subprocess


def git(*args):
    return subprocess.check_output(['git', *args], text=True).strip()


def digest(path):
    h = hashlib.sha256()
    with path.open('rb') as stream:
        for chunk in iter(lambda: stream.read(1024 * 1024), b''):
            h.update(chunk)
    return h.hexdigest()


def main():
    p = argparse.ArgumentParser(description=__doc__)
    p.add_argument('file', type=pathlib.Path)
    p.add_argument('--branch', required=True)
    p.add_argument('--head-sha', required=True)
    p.add_argument('--agent', required=True)
    p.add_argument('--name')
    p.add_argument('--description', default='')
    args = p.parse_args()
    root = pathlib.Path(git('rev-parse', '--show-toplevel'))
    if not (root / '.artifact-queue').is_file() or (root / '.artifact-queue').read_text().strip() != 'fieldcraft-artifact-queue-v1':
        p.error('Run inside a worktree based on artifact-queue, never a feature branch.')
    name = args.name or args.file.name
    for label, value, pattern in [('branch', args.branch, r'[A-Za-z0-9][A-Za-z0-9._/-]{0,179}'), ('name', name, r'[A-Za-z0-9][A-Za-z0-9._-]{0,119}'), ('agent', args.agent, r'[A-Za-z0-9][A-Za-z0-9_.-]{0,79}'), ('head-sha', args.head_sha, r'[a-f0-9]{40}')]:
        if not re.fullmatch(pattern, value) or '..' in value or '//' in value or value.endswith(('/', '.')):
            p.error('Invalid ' + label)
    if args.branch in ['main', 'develop', 'pr-queue', 'artifact-queue', 'upstream-develop']:
        p.error('Use a feature branch name.')
    if len(args.description) > 1000:
        p.error('Description exceeds 1000 characters.')
    source = args.file.resolve()
    size = source.stat().st_size
    if not 0 < size <= 100 * 1024 * 1024:
        p.error('Artifact must be between 1 byte and 100 MiB.')
    oid = digest(source)
    relative = pathlib.PurePosixPath('objects', args.branch, args.head_sha, name)
    manifest = pathlib.PurePosixPath('manifests', args.branch, args.head_sha, name + '.json')
    target = root / relative
    description = dict(schema=1, agent=args.agent, branch=args.branch, head_sha=args.head_sha, name=name, bytes=size, sha256=oid, description=args.description)
    previous = root / manifest
    if previous.exists():
        old = json.loads(previous.read_text())
        if old['sha256'] != oid:
            p.error('This name/head already identifies different bytes. Use a versioned filename.')
        description['submitted_at'] = old['submitted_at']
    else:
        description['submitted_at'] = datetime.datetime.now(datetime.timezone.utc).isoformat()
    target.parent.mkdir(parents=True, exist_ok=True)
    if source != target.resolve():
        shutil.copyfile(source, target)
    if digest(target) != oid or target.stat().st_size != size:
        p.error('Source changed during staging; retry with a stable file.')
    previous.parent.mkdir(parents=True, exist_ok=True)
    previous.write_text(json.dumps(description, indent=2) + '\n')
    if git('check-attr', 'filter', '--', str(relative)).endswith(': lfs'):
        p.error('Artifact is still LFS-tracked. Fetch/rebase the current artifact-queue attributes before staging.')
    subprocess.run(['git', 'add', '--', str(relative), str(manifest)], check=True)
    staged = subprocess.check_output(['git', 'show', ':' + str(relative)])
    if len(staged) != size or hashlib.sha256(staged).hexdigest() != oid:
        p.error('Index bytes differ from the artifact. Check Git filters before committing.')
    print(json.dumps({'status': 'staged_not_pushed', 'object': str(relative), 'manifest': str(manifest), 'sha256': oid, 'bytes': size}))
    print('Commit these two paths and push HEAD:artifact-queue using the existing fork deploy key.')


if __name__ == '__main__':
    main()
