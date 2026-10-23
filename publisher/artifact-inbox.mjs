// Ordinary Git artifact inbox. No SSH receiver or GitHub credentials on the agent side.
import {spawnSync} from 'node:child_process';
import {createHash} from 'node:crypto';
import {mkdirSync, readFileSync, statSync, readdirSync, writeFileSync, chmodSync, existsSync} from 'node:fs';
import {join} from 'node:path';
import {homedir} from 'node:os';
import {FORK} from './config.mjs';
import {putJson} from './relay.mjs';
const ROOT = join(homedir(), '.local/share/fieldcraft-pr-artifacts');
const MAX = 100 * 1024 * 1024;
export function manifestRecord(path, text, blob) {
  const m = JSON.parse(text);
  if (m.schema !== 1 || !/^[A-Za-z0-9][A-Za-z0-9._/-]{0,179}$/.test(m.branch || '') || m.branch.includes('..') || m.branch.includes('//') || /[/.]$/.test(m.branch)) throw Error('Invalid artifact branch.');
  if (!/^[a-f0-9]{40}$/.test(m.head_sha || '') || !/^[A-Za-z0-9][A-Za-z0-9_.-]{0,79}$/.test(m.agent || '')) throw Error('Invalid artifact head or agent.');
  if (!/^[A-Za-z0-9][A-Za-z0-9._-]{0,119}$/.test(m.name || '') || m.name.includes('..')) throw Error('Invalid artifact basename.');
  if (!Number.isSafeInteger(m.bytes) || m.bytes < 1 || m.bytes > MAX || !/^[a-f0-9]{64}$/.test(m.sha256 || '')) throw Error('Invalid artifact size or SHA-256.');
  if (path !== `manifests/${m.branch}/${m.head_sha}/${m.name}.json`) throw Error('Artifact manifest path does not match metadata.');
  if (!blob || blob.type !== 'blob' || blob.size !== m.bytes || !/^[a-f0-9]{40}$/.test(blob.sha || '')) throw Error('Artifact Git blob is missing or its size does not match the manifest.');
  return m;
}
function git(args, dir) {
  const p = spawnSync('/usr/bin/git', ['--git-dir', dir, ...args], {encoding: 'utf8', timeout: 180000, maxBuffer: 4 * 1024 * 1024});
  if (p.error || p.status !== 0) throw Error('Maintainer Git retrieval failed; agents should not sign in. The maintainer must inspect local access.');
  return p.stdout;
}
function retrieve(m, commit, root) {
  const objects = join(root, 'objects'); mkdirSync(objects, {recursive: true, mode: 0o700});
  const dest = join(objects, m.sha256);
  if (existsSync(dest) && statSync(dest).size === m.bytes && createHash('sha256').update(readFileSync(dest)).digest('hex') === m.sha256) return;
  const used = readdirSync(objects).filter(n => /^[a-f0-9]{64}$/.test(n)).reduce((sum, n) => sum + statSync(join(objects, n)).size, 0);
  if (used + m.bytes > 2 * 1024 * 1024 * 1024) throw Error('Maintainer artifact cache quota reached; maintainer must archive received files.');
  const bare = join(root, 'git-cache.git');
  if (!existsSync(bare)) {
    mkdirSync(root, {recursive: true, mode: 0o700});
    const p = spawnSync('/usr/bin/git', ['init', '--bare', bare], {encoding: 'utf8'});
    if (p.status !== 0) throw Error('Cannot initialize maintainer Git cache.');
    git(['remote', 'add', 'origin', `https://github.com/${FORK}.git`], bare);
  }
  git(['fetch', '--depth=1', 'origin', commit], bare);
  const object = `objects/${m.branch}/${m.head_sha}/${m.name}`;
  const spec = `${commit}:${object}`;
  const size = Number(git(['cat-file', '-s', spec], bare).trim());
  if (size !== m.bytes) throw Error('Downloaded Git blob size does not match the manifest.');
  const result = spawnSync('/usr/bin/git', ['--git-dir', bare, 'cat-file', 'blob', spec], {timeout: 180000, maxBuffer: MAX + 1024});
  if (result.error || result.status !== 0 || result.stdout.length !== m.bytes || createHash('sha256').update(result.stdout).digest('hex') !== m.sha256) throw Error('Downloaded artifact bytes/hash do not match the manifest.');
  writeFileSync(dest, result.stdout, {mode: 0o600}); chmodSync(dest, 0o400);
}
export function syncArtifacts(api, root = ROOT, fetchObject = retrieve) {
  let ref;
  try { ref = api(`repos/${FORK}/git/ref/heads/artifact-queue`); } catch (e) { if (e.notFound) return; throw e; }
  const commit = ref.object.sha;
  const tree = api(`repos/${FORK}/git/trees/${commit}?recursive=1`);
  if (tree.truncated) throw Error('Artifact tree is truncated; maintainer intervention required.');
  const blobs = new Map(tree.tree.filter(x => x.type === 'blob').map(x => [x.path, x]));
  const manifests = [...blobs.keys()].filter(p => /^manifests\/.+\.json$/.test(p));
  if (manifests.length > 1000) throw Error('Archive old manifests before adding more than 1000 artifacts.');
  const read = path => {
    const b = blobs.get(path); if (!b || b.size > 16384) throw Error('Missing or oversized artifact metadata.');
    const v = api(`repos/${FORK}/git/blobs/${b.sha}`);
    if (v.encoding !== 'base64') throw Error('Unexpected artifact blob encoding.');
    return Buffer.from(v.content, 'base64').toString('utf8');
  };
  let received = 0, blocked = 0;
  for (const path of manifests) {
    let m;
    try {
      const text = read(path); const raw = JSON.parse(text);
      const object = `objects/${raw.branch}/${raw.head_sha}/${raw.name}`;
      m = manifestRecord(path, text, blobs.get(object));
      fetchObject(m, commit, root);
      const receiptPath = `receipts/artifacts/${m.branch}/${m.head_sha}/${m.name}.json`;
      let old;
      try {
        const value = api(`repos/${FORK}/contents/${encodeURIComponent(receiptPath)}?ref=pr-queue`);
        if (value.encoding === 'base64') old = JSON.parse(Buffer.from(value.content, 'base64').toString('utf8'));
      } catch (e) { if (!e.notFound) throw e; }
      const same = old?.verified_sha256 === m.sha256 && old.manifest_blob === blobs.get(path).sha;
      const attached = same && /^https:\/\/(?:github\.com\/user-attachments\/|user-images\.githubusercontent\.com\/)/.test(old.github_url || '');
      putJson(api, receiptPath, {...m, status: 'received', artifact_commit: same ? old.artifact_commit : commit, manifest: path,
        manifest_blob: blobs.get(path).sha, object, verified_sha256: m.sha256,
        github_attachment: attached ? 'attached' : 'pending_maintainer', ...(attached ? {github_url: old.github_url} : {})});
      received++;
    } catch (e) {
      blocked++; console.error(`${path}: ${e.message}`);
      const id = createHash('sha256').update(path).digest('hex');
      putJson(api, `receipts/artifact-errors/${id}.json`, {status: 'blocked', manifest: path, error: e.message});
    }
  }
  putJson(api, 'feedback/artifacts.json', {
    transport: 'fork artifact-queue via ordinary Git', status: received ? 'verified_uploads_received' : blocked ? 'retrieval_blocked' : 'awaiting_first_verified_upload', received, blocked
  });
  console.log(JSON.stringify({git_artifacts: {received, blocked}}));
}
