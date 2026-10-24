#!/usr/bin/env node
// Maintainer-only native GitHub attachments. Agent credentials are never used or requested.
import {spawnSync} from 'node:child_process';
import {createHash} from 'node:crypto';
import {existsSync, lstatSync, mkdirSync, readFileSync, renameSync, writeFileSync} from 'node:fs';
import {homedir} from 'node:os';
import {join} from 'node:path';
import {pathToFileURL} from 'node:url';
import {FORK, UPSTREAM, QUEUE, GH} from './config.mjs';
import {putJson} from './relay.mjs';
const ROOT = join(homedir(), '.local/share/fieldcraft-pr-artifacts');
const MAX = 10 * 1024 * 1024;
const START = '<!-- fieldcraft-attachments:start -->';
const END = '<!-- fieldcraft-attachments:end -->';
const MIME = {mp4: 'video/mp4', mov: 'video/quicktime', webm: 'video/webm', png: 'image/png', jpg: 'image/jpeg', jpeg: 'image/jpeg', gif: 'image/gif', svg: 'image/svg+xml'};
const assetURL = url => /^https:\/\/github\.com\/user-attachments\/assets\/[a-f0-9-]{36}$/.test(url || '');

export function mediaRecord(record, pr, capture, note = '') {
  if (pr.state !== 'open' || pr.head?.repo?.full_name?.toLowerCase() !== FORK || pr.base?.repo?.full_name?.toLowerCase() !== UPSTREAM || pr.base?.ref !== 'develop') throw Error('Only an open fork PR against upstream develop can receive attachments.');
  if (!/^[a-f0-9]{40}$/.test(capture) || (capture !== pr.head.sha && (!note.trim() || note.length > 1000))) throw Error('An older capture requires an explicit maintainer applicability note.');
  if (record.branch !== pr.head.ref || record.head_sha !== capture || record.status !== 'received' || record.verified_sha256 !== record.sha256 || !/^[a-f0-9]{64}$/.test(record.sha256 || '')) throw Error('Receipt must identify verified bytes for this branch and capture revision.');
  if (!/^[A-Za-z0-9][A-Za-z0-9._-]{0,119}$/.test(record.name || '') || record.name.includes('..') || !Number.isSafeInteger(record.bytes) || record.bytes < 1 || record.bytes > MAX) throw Error('Invalid attachment name or size (maximum 10 MiB).');
  const mime = MIME[record.name.split('.').at(-1).toLowerCase()];
  if (!mime) throw Error('Only supported images and videos can be attached by this helper.');
  return {...record, mime};
}

export function renderMedia(body, records, capture, note = '') {
  const a = body.indexOf(START), b = body.indexOf(END);
  if (a >= 0 || b >= 0) {
    if (a < 0 || b < a) throw Error('Malformed managed attachment block; inspect manually.');
    body = body.slice(0, a) + body.slice(b + END.length);
  }
  // Retire the one-off first successful upload marker, retaining the URL in the managed block.
  body = body.replace(/\n<!-- fieldcraft-media:[a-f0-9]{64} -->\nhttps:\/\/github\.com\/user-attachments\/assets\/[a-f0-9-]{36}\n/g, '\n');
  const lines = [START, '### Attached files', '', 'These uploads are complete; earlier pending-upload wording below describes the original handoff.', '', `Capture/source revision: \`${capture}\`.`, ...(note ? ['', note] : [])];
  for (const r of [...records].sort((x, y) => Number(!x.mime.startsWith('video/')) - Number(!y.mime.startsWith('video/')) || x.name.localeCompare(y.name))) {
    if (!assetURL(r.github_url)) throw Error('Invalid native GitHub attachment URL.');
    lines.push('', `**${r.name}**`, '', r.mime.startsWith('video/') ? r.github_url : `![${r.name}](${r.github_url})`);
  }
  lines.push('', END);
  const block = lines.join('\n');
  const heading = /^## (?:Before and after|In-app demonstration|Demonstration(?: and provenance| attachments)?)\s*$/mi.exec(body);
  if (heading) {
    const at = heading.index + heading[0].length;
    return (body.slice(0, at).trimEnd() + '\n\n' + block + '\n\n' + body.slice(at).trimStart()).trimEnd() + '\n';
  }
  return body.trimEnd() + '\n\n## Demonstration attachments\n\n' + block + '\n';
}

function run(args, binary = false) {
  const result = spawnSync(GH, args, {encoding: binary ? undefined : 'utf8', timeout: 120000, maxBuffer: MAX + 1024 * 1024});
  if (result.error || result.status !== 0) throw Error(`GitHub attachment operation failed (${args[0]}). No agent sign-in is needed; inspect maintainer access.`);
  return result.stdout;
}
export function uploadMedia(record, repository, root = ROOT) {
  const source = join(root, 'objects', record.sha256);
  const info = lstatSync(source);
  if (!info.isFile() || info.size !== record.bytes || createHash('sha256').update(readFileSync(source)).digest('hex') !== record.sha256) throw Error('Local attachment bytes do not match the verified receipt.');
  const ledgerPath = join(root, 'github-uploads.json');
  const ledger = existsSync(ledgerPath) ? JSON.parse(readFileSync(ledgerPath, 'utf8')) : {};
  let url = ledger[record.sha256] || record.github_url;
  if (!assetURL(url)) {
    const endpoint = new URL('https://uploads.github.com/user-attachments/assets');
    endpoint.search = new URLSearchParams({name: record.name, content_type: record.mime, repository_id: String(repository)});
    url = JSON.parse(run(['api', endpoint.href, '--method', 'POST', '--header', 'Content-Type: application/octet-stream', '--input', source])).url;
    if (!assetURL(url)) throw Error('GitHub did not return a native attachment URL.');
    ledger[record.sha256] = url;
    mkdirSync(root, {recursive: true, mode: 0o700});
    const temporary = ledgerPath + '.tmp';
    writeFileSync(temporary, JSON.stringify(ledger, null, 2) + '\n', {mode: 0o600}); renameSync(temporary, ledgerPath);
  }
  // Persist the URL before checking download/publication, so a retry never uploads it twice.
  const bytes = run(['api', url], true);
  if (bytes.length !== record.bytes || createHash('sha256').update(bytes).digest('hex') !== record.sha256) throw Error('GitHub attachment download does not match the original bytes.');
  return url;
}

export function postMedia(api, number, {capture, note = '', upload = uploadMedia} = {}) {
  if (!Number.isSafeInteger(number) || number <= 0) throw Error('Give a positive PR number.');
  const route = `repos/${UPSTREAM}/pulls/${number}`;
  const pr = api(route); capture ??= pr.head.sha;
  // Validate the destination before reading files or uploading anything.
  if (pr.state !== 'open' || pr.head?.repo?.full_name?.toLowerCase() !== FORK || pr.base?.repo?.full_name?.toLowerCase() !== UPSTREAM || pr.base?.ref !== 'develop') throw Error('PR is closed or is not our fork PR against upstream develop.');
  const ref = api(`repos/${FORK}/git/ref/heads/${QUEUE}`);
  const tree = api(`repos/${FORK}/git/trees/${ref.object.sha}?recursive=1`);
  if (tree.truncated) throw Error('Queue tree is truncated.');
  const prefix = `receipts/artifacts/${pr.head.ref}/${capture}/`;
  const paths = tree.tree.filter(b => b.type === 'blob' && b.path.startsWith(prefix) && b.path.endsWith('.json'));
  if (!paths.length || paths.length > 20) throw Error('Need 1–20 received media receipts for this capture.');
  const records = paths.map(b => {
    if (b.size > 16384) throw Error('Oversized receipt.');
    const blob = api(`repos/${FORK}/git/blobs/${b.sha}`);
    if (blob.encoding !== 'base64') throw Error('Unexpected receipt encoding.');
    const r = JSON.parse(Buffer.from(blob.content, 'base64').toString('utf8'));
    // Numerical QA reports are retained in the inbox, not sent to the image/video endpoint.
    if (!MIME[(r.name || '').split('.').at(-1).toLowerCase()]) return null;
    return {...mediaRecord(r, pr, capture, note), receipt: b.path};
  }).filter(Boolean);
  if (!records.length) throw Error('No supported media receipts for this capture.');
  const repository = api(`repos/${FORK}`);
  if (!repository.permissions?.push || repository.private || !Number.isSafeInteger(repository.id)) throw Error('Uploads require write access to the existing public fork.');
  for (const r of records) r.github_url = upload(r, repository.id);
  const fresh = api(route);
  if (fresh.state !== 'open' || fresh.head.sha !== pr.head.sha || fresh.head.ref !== pr.head.ref || fresh.head.repo.full_name.toLowerCase() !== FORK) throw Error('PR changed during upload. URLs are retained; review before retrying.');
  const body = renderMedia(fresh.body || '', records, capture, note);
  if (body !== fresh.body) api(`repos/${UPSTREAM}/issues/${number}`, 'PATCH', {body});
  const published = api(route);
  if (published.state !== 'open' || published.head.sha !== fresh.head.sha || records.some(r => !published.body.includes(r.github_url))) throw Error('PR publication could not be verified.');
  for (const r of records) {
    const {receipt, mime, ...original} = r;
    putJson(api, receipt, {...original, github_attachment: 'attached'});
  }
  return {pr: number, capture, current_head: fresh.head.sha, videos: records.filter(r => r.mime.startsWith('video/')).length, images: records.filter(r => r.mime.startsWith('image/')).length};
}

// Automatic publication is restricted to exact current-head captures. Historical applicability
// is a maintainer decision, never inferred from an agent's text.
export function postReadyMedia(api, pulls, publish = postMedia) {
  const ref = api(`repos/${FORK}/git/ref/heads/${QUEUE}`);
  const tree = api(`repos/${FORK}/git/trees/${ref.object.sha}?recursive=1`);
  if (tree.truncated) throw Error('Attachment queue tree is truncated.');
  const blobs = tree.tree.filter(b => b.type === 'blob' && b.path.startsWith('receipts/artifacts/') && /\.(?:mp4|mov|webm|png|jpg|jpeg|gif|svg)\.json$/i.test(b.path));
  if (blobs.length > 1000) throw Error('Archive old media receipts before exceeding 1000.');
  const results = [], errors = [];
  for (const pr of pulls) {
    if (pr.state !== 'open' || pr.head?.repo?.full_name?.toLowerCase() !== FORK || pr.base?.repo?.full_name?.toLowerCase() !== UPSTREAM || pr.base?.ref !== 'develop') continue;
    const files = blobs.filter(b => b.path.startsWith(`receipts/artifacts/${pr.head.ref}/${pr.head.sha}/`));
    if (!files.length) continue;
    try {
      const records = files.map(b => {
        if (b.size > 16384) throw Error('Oversized attachment receipt.');
        const blob = api(`repos/${FORK}/git/blobs/${b.sha}`);
        if (blob.encoding !== 'base64') throw Error('Unexpected attachment receipt encoding.');
        return mediaRecord(JSON.parse(Buffer.from(blob.content, 'base64').toString('utf8')), pr, pr.head.sha);
      });
      if (records.every(r => r.github_attachment === 'attached' && assetURL(r.github_url) && pr.body?.includes(r.github_url))) continue;
      if (results.length + errors.length >= 20) break;
      results.push(publish(api, pr.number));
    } catch (e) { errors.push({pr: pr.number, error: e.message}); }
  }
  putJson(api, 'feedback/attachments.json', {transport: 'native GitHub user attachments uploaded against the public fork', policy: 'Automatic only for exact current-head captures; older captures require maintainer review.', published: results, errors});
  return {published: results.length, errors: errors.length};
}

if (process.argv[1] && import.meta.url === pathToFileURL(process.argv[1]).href) {
  try {
    const args = process.argv.slice(2), values = {};
    for (let i = 0; i < args.length; i += 2) {
      if (!['--pr', '--capture', '--note'].includes(args[i]) || args[i + 1] === undefined) throw Error('Usage: post-media.mjs --pr N [--capture FULL-SHA --note "Maintainer applicability review"]');
      values[args[i].slice(2)] = args[i + 1];
    }
    const {ghApi} = await import('./publish.mjs');
    console.log(JSON.stringify(postMedia(ghApi, Number(values.pr), values)));
  } catch (e) { console.error(e.message); process.exitCode = 1; }
}
