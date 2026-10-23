#!/usr/bin/env node
// Operational PR/claim bot: request text is data, never a command or executable checkout.
import {spawnSync} from 'node:child_process';
import {pathToFileURL} from 'node:url';

export const FORK = 'bluecmd/fieldcraft';
export const UPSTREAM = 'tangency-labs/fieldcraft';
export const QUEUE = 'pr-queue';
const ACTIVE = new Set(['granting', 'granted', 'release_failed', 'suspended']);
const stamp = () => new Date().toISOString();

export function parseRequest(path, text) {
  if (Buffer.byteLength(text) > 65536) throw Error('Request exceeds 64 KiB.');
  const m = /^---\r?\n([\s\S]*?)\r?\n---\r?\n([\s\S]*)$/.exec(text);
  if (!m) throw Error('Use JSON front matter between --- lines, then the Markdown body.');
  const r = JSON.parse(m[1]);
  const kind = path.startsWith('claims/') ? 'claim' : 'pr';
  const prefix = kind === 'claim' ? 'claims/' : 'prs/';
  const branch = path.slice(prefix.length, -3);
  if (!/^[A-Za-z0-9][A-Za-z0-9._/-]{0,179}$/.test(branch) || branch.includes('..') ||
      branch.includes('//') || /[/.]$|\.lock(?:\/|$)/.test(branch) ||
      ['main', 'develop', QUEUE, 'artifact-queue', 'upstream-develop'].includes(branch)) throw Error('Invalid feature branch name.');
  if (r.schema !== 1 || r.kind !== kind || r.branch !== branch) throw Error('Schema, kind or branch does not match the request path.');
  if (!/^[A-Za-z0-9][A-Za-z0-9_.-]{0,79}$/.test(r.agent || '')) throw Error('Give a stable agent identifier.');
  if (!/^[A-Za-z0-9][A-Za-z0-9_.-]{0,127}$/.test(r.request_id || '')) throw Error('Give a unique request_id.');
  if (!Number.isSafeInteger(r.issue) || r.issue <= 0) throw Error('One primary issue number is required.');
  if (kind === 'claim' && !['claim', 'release'].includes(r.action)) throw Error('Claim action must be claim or release.');
  if (kind === 'pr') {
    if (r.action !== 'publish' || !/^[a-f0-9]{40}$/.test(r.head_sha || '')) throw Error('PR needs action publish and the full tested head_sha.');
    if (r.base !== 'develop' || typeof r.draft !== 'boolean') throw Error('Base must be develop; draft must be explicit.');
    if (typeof r.title !== 'string' || !r.title.trim() || r.title.length > 256 || /[\r\n]/.test(r.title)) throw Error('Invalid title.');
    if (!['attached', 'not_needed', 'pending'].includes(r.media)) throw Error('Declare media: attached, not_needed or pending.');
    if (!/^## Validation\s*$/mi.test(m[2]) || !/^## Known limitations\s*$/mi.test(m[2])) throw Error('Body needs Validation and Known limitations sections.');
    if (r.media === 'attached' && !/https:\/\/(?:github\.com\/user-attachments\/|user-images\.githubusercontent\.com\/)/.test(m[2])) throw Error('Include the GitHub attachment URL in the body.');
    if (/https:\/\/(?:raw\.githubusercontent\.com\/bluecmd\/fieldcraft\/|github\.com\/bluecmd\/fieldcraft\/raw\/)[^\s)]+\.(?:mp4|webm|mov|png|jpe?g|gif)(?:[?#\s)]|$)/i.test(m[2])) throw Error('Do not link demo binaries committed to the feature branch; use GitHub attachments.');
  }
  return {...r, body: m[2].trim() + '\n', path};
}

export function ghApi(route, method = 'GET', data) {
  const args = ['api', route, '--method', method];
  if (data !== undefined) args.push('--input', '-');
  const p = spawnSync('/usr/bin/gh', args, {input: data === undefined ? undefined : JSON.stringify(data), encoding: 'utf8', timeout: 60000, maxBuffer: 8 * 1024 * 1024});
  if (p.error) throw p.error;
  if (p.status !== 0) {
    const e = Error((p.stderr || p.stdout || 'GitHub API failed').slice(0, 1800));
    e.notFound = /HTTP 404|"status":\s*"404"/.test(e.message);
    throw e;
  }
  return p.stdout.trim() ? JSON.parse(p.stdout) : null;
}

export class Publisher {
  constructor(api = ghApi, dryRun = false) { this.api = api; this.dryRun = dryRun; this.receipts = new Map(); this.pulls = []; }
  get(route) { return this.api(route); }
  mutate(route, method, data) {
    if (this.dryRun) throw Error('Dry-run attempted a mutation.');
    return this.api(route, method, data);
  }
  content(path, ref = QUEUE) {
    return this.get(`repos/${FORK}/contents/${encodeURIComponent(path)}?ref=${encodeURIComponent(ref)}`);
  }
  decode(blob) {
    if (blob.encoding !== 'base64' || typeof blob.content !== 'string') throw Error('Expected a small base64 GitHub blob.');
    return Buffer.from(blob.content, 'base64').toString('utf8');
  }
  save(kind, branch, value) {
    const path = `receipts/${kind === 'claim' ? 'claims' : 'prs'}/${branch}.json`;
    const old = this.receipts.get(path);
    const clean = x => { const {updated_at, ...v} = x || {}; return v; };
    if (JSON.stringify(clean(old)) === JSON.stringify(clean(value))) return;
    if (this.dryRun) { console.log(JSON.stringify({receipt: path, ...value})); return; }
    let current;
    try { current = this.content(path); } catch (e) { if (!e.notFound) throw e; }
    const record = {...value, updated_at: stamp()};
    this.mutate(`repos/${FORK}/contents/${encodeURIComponent(path)}`, 'PUT', {
      branch: QUEUE, message: `queue: ${branch}: ${value.status}`,
      content: Buffer.from(JSON.stringify(record, null, 2) + '\n').toString('base64'),
      ...(current ? {sha: current.sha} : {}),
    });
    this.receipts.set(path, record);
  }
  fresh(r, blob) {
    if (this.content(r.path).sha !== blob) throw Error('Request changed while processing; it will be retried next scan.');
  }
  references(pr, n) {
    return new RegExp(`(?:^|[^\\w])#${n}(?!\\d)|github\\.com/tangency-labs/fieldcraft/issues/${n}(?!\\d)`, 'i').test(pr.title + '\n' + (pr.body || ''));
  }
  sameBranch(pr, branch) {
    return pr.head?.ref === branch && pr.head?.repo?.full_name?.toLowerCase() === FORK;
  }
  issue(r) {
    const i = this.get(`repos/${UPSTREAM}/issues/${r.issue}`);
    if (i.pull_request || i.state !== 'open' || i.locked) throw Error('Primary issue must be an open, unlocked issue.');
    return i;
  }
  commentOnce(issue, marker, body) {
    let found = false;
    for (let page = 1; page <= 20; page++) {
      const comments = this.get(`repos/${UPSTREAM}/issues/${issue}/comments?per_page=100&page=${page}`);
      if (comments.some(c => c.body?.includes(marker))) { found = true; break; }
      if (comments.length < 100) break;
      if (page === 20) throw Error('Comment history exceeds scan bound; manual handoff required.');
    }
    if (!found) this.mutate(`repos/${UPSTREAM}/issues/${issue}/comments`, 'POST', {body: `${marker}\n${body}`});
  }
  claim(r, blob) {
    const key = `receipts/claims/${r.branch}.json`;
    const old = this.receipts.get(key);
    const record = {request_id: r.request_id, request_blob: blob, agent: r.agent, branch: r.branch, issue: r.issue};
    if (r.action === 'release') {
      if (!old || old.agent !== r.agent || old.issue !== r.issue || old.request_id !== r.request_id || !ACTIVE.has(old.status)) throw Error('Release must match the recorded reservation and original request_id.');
      const i = this.issue(r);
      if (this.pulls.some(p => this.sameBranch(p, r.branch))) throw Error('An open PR still uses this branch; keep its issue reserved until the PR is closed/merged.');
      this.fresh(r, blob);
      if (!this.dryRun) {
        if (i.assignees.some(a => a.login === 'bluecmd')) this.mutate(`repos/${UPSTREAM}/issues/${r.issue}/assignees`, 'DELETE', {assignees: ['bluecmd']});
        if (i.assignees.every(a => a.login === 'bluecmd') && i.labels.some(l => l.name === 'claimed')) this.mutate(`repos/${UPSTREAM}/issues/${r.issue}/labels/claimed`, 'DELETE');
        this.commentOnce(r.issue, `<!-- fieldcraft-pr-queue:release:${r.request_id} -->`, `Released the queue reservation for agent \`${r.agent}\`, branch \`${r.branch}\`.`);
      }
      this.save('claim', r.branch, {...record, status: 'released'});
      return;
    }
    if (old && ACTIVE.has(old.status) && (old.issue !== r.issue || old.agent !== r.agent || old.request_id !== r.request_id)) throw Error('Release the existing branch reservation before changing its owner, issue or request_id.');
    for (const [path, receipt] of this.receipts) {
      if (path !== key && path.startsWith('receipts/claims/') && ACTIVE.has(receipt.status) && receipt.issue === r.issue) throw Error(`Issue reserved by ${receipt.agent} on ${receipt.branch}.`);
    }
    const i = this.issue(r);
    if (i.labels.some(l => l.name === 'in progress' || l.name === 'paused') && !old) throw Error('Core-team or paused work requires an explicit maintainer handoff, not an automatic takeover.');
    if (i.assignees.some(a => a.login !== 'bluecmd')) throw Error('Issue belongs to another GitHub assignee.');
    if (this.pulls.some(p => this.references(p, r.issue) && !this.sameBranch(p, r.branch))) throw Error('Another open PR references this issue; coordinate before claiming it.');
    if (i.assignees.length && !old && !this.pulls.some(p => this.sameBranch(p, r.branch) && this.references(p, r.issue))) throw Error('Issue already belongs to our account without an agent receipt; manual reconciliation required.');
    this.fresh(r, blob);
    if (!this.dryRun) {
      this.save('claim', r.branch, {...record, status: 'granting'});
      this.mutate(`repos/${UPSTREAM}/issues/${r.issue}/assignees`, 'POST', {assignees: ['bluecmd']});
      this.mutate(`repos/${UPSTREAM}/issues/${r.issue}/labels`, 'POST', {labels: ['claimed']});
      this.commentOnce(r.issue, `<!-- fieldcraft-pr-queue:claim:${r.request_id} -->`, `Claimed for agent \`${r.agent}\` on \`bluecmd/fieldcraft:${r.branch}\` through the [PR queue](https://github.com/${FORK}/tree/${QUEUE}). GitHub assignee: @bluecmd. The agent may start once its queue receipt says \`granted\`. This is a reservation, not a claim of completion.`);
    }
    this.save('claim', r.branch, {...record, status: this.dryRun ? 'would_grant' : 'granted'});
  }
  checkMedia(compare) {
    if (compare.ahead_by > 50 || compare.commits.length !== compare.ahead_by) throw Error('More than 50 commits or incomplete comparison: manual handoff required.');
    for (const commit of compare.commits) {
      const detail = this.get(`repos/${FORK}/commits/${commit.sha}?per_page=100`);
      if (detail.files.length >= 100) throw Error('Commit file list reaches the inspection bound; manual handoff required.');
      for (const f of detail.files) {
        if (f.status === 'removed') continue;
        if (/\.(?:mp4|webm|mov|mkv|avi)$/i.test(f.filename) || /^(?:docs\/|tools\/demo-video\/).*\.(?:png|jpe?g|gif|webp)$/i.test(f.filename)) throw Error(`Remove demo media from every unmerged commit: ${f.filename}. Attach it to GitHub instead; a later deletion still leaves the binary in history.`);
      }
    }
  }
  publish(r, blob) {
    const receipt = this.receipts.get(`receipts/claims/${r.branch}.json`);
    if (!receipt || receipt.status !== 'granted' || receipt.agent !== r.agent || receipt.issue !== r.issue) throw Error('Wait for a matching granted issue claim before requesting publication.');
    const i = this.issue(r);
    if (!i.assignees.some(a => a.login === 'bluecmd') || i.assignees.some(a => a.login !== 'bluecmd')) throw Error('GitHub issue assignment no longer matches the granted claim.');
    const head = this.get(`repos/${FORK}/branches/${encodeURIComponent(r.branch)}`);
    if (head.commit.sha !== r.head_sha) throw Error('Branch head differs from the tested head_sha; update validation and resubmit.');
    const compare = this.get(`repos/${UPSTREAM}/compare/develop...bluecmd:${encodeURIComponent(r.branch)}`);
    if (!compare.ahead_by) throw Error('Branch has no changes against upstream develop.');
    if (compare.behind_by) throw Error('Rebase onto current upstream develop, rerun the affected checks, and resubmit the tested SHA.');
    this.checkMedia(compare);
    const existing = this.get(`repos/${UPSTREAM}/pulls?state=all&head=bluecmd:${encodeURIComponent(r.branch)}&base=develop&per_page=100`);
    if (existing.length >= 100) throw Error('PR history exceeds inspection bound.');
    this.fresh(r, blob);
    if (this.get(`repos/${FORK}/branches/${encodeURIComponent(r.branch)}`).commit.sha !== r.head_sha) throw Error('Branch changed during inspection; resubmit its tested SHA.');
    let pr = existing.find(p => p.state === 'open') || existing[0];
    if (pr?.state === 'closed') {
      this.save('pr', r.branch, {request_id: r.request_id, request_blob: blob, agent: r.agent, branch: r.branch, issue: r.issue, status: 'already_closed', pr: pr.html_url});
      return;
    }
    if (this.dryRun) {
      this.save('pr', r.branch, {request_id: r.request_id, request_blob: blob, status: pr ? 'would_update' : 'would_create', branch: r.branch});
      return;
    }
    if (pr) {
      if (pr.draft !== r.draft) throw Error('Existing PR draft state differs; change readiness manually before resubmitting.');
      if (pr.title !== r.title || pr.body !== r.body) pr = this.mutate(`repos/${UPSTREAM}/pulls/${pr.number}`, 'PATCH', {title: r.title, body: r.body});
    } else {
      pr = this.mutate(`repos/${UPSTREAM}/pulls`, 'POST', {title: r.title, body: r.body, head: `bluecmd:${r.branch}`, base: 'develop', draft: r.draft, maintainer_can_modify: true});
    }
    if (pr.head?.sha !== r.head_sha) throw Error(`PR ${pr.html_url} exists but its branch moved during publication. Inspect it; the next scan will not create a duplicate.`);
    this.save('pr', r.branch, {request_id: r.request_id, request_blob: blob, agent: r.agent, branch: r.branch, issue: r.issue, status: 'posted', head_sha: r.head_sha, pr: pr.html_url, media: r.media});
  }
  run() {
    if (this.get('user').login !== 'bluecmd') throw Error('Queue publisher must authenticate as bluecmd.');
    const ref = this.get(`repos/${FORK}/git/ref/heads/${QUEUE}`);
    const tree = this.get(`repos/${FORK}/git/trees/${ref.object.sha}?recursive=1`);
    if (tree.truncated) throw Error('Queue tree is truncated; refusing an incomplete scan.');
    const blobs = tree.tree.filter(x => x.type === 'blob');
    for (const b of blobs.filter(b => /^receipts\/(?:claims|prs)\/.+\.json$/.test(b.path))) {
      this.receipts.set(b.path, JSON.parse(this.decode(this.get(`repos/${FORK}/git/blobs/${b.sha}`))));
    }
    // Retire reservations once the primary issue is closed; never close or reopen issues here.
    for (const [path, r] of this.receipts) {
      if (path.startsWith('receipts/claims/') && ACTIVE.has(r.status)) {
        const i = this.get(`repos/${UPSTREAM}/issues/${r.issue}`);
        if (i.state === 'closed') this.save('claim', r.branch, {...r, status: 'completed'});
        else if (r.status === 'granted' && (!i.assignees.some(a => a.login === 'bluecmd') || i.assignees.some(a => a.login !== 'bluecmd'))) {
          this.save('claim', r.branch, {...r, status: 'suspended', error: 'GitHub assignment changed. Stop and coordinate; explicitly update the claim request only after reconciliation.'});
        }
      }
    }
    this.pulls = [];
    for (let page = 1; page <= 10; page++) {
      const pulls = this.get(`repos/${UPSTREAM}/pulls?state=open&per_page=100&page=${page}`);
      this.pulls.push(...pulls);
      if (pulls.length < 100) break;
      if (page === 10) throw Error('Too many open PRs to check ownership safely.');
    }
    const requests = blobs.filter(b => /^(?:claims|prs)\/.+\.md$/.test(b.path)).sort((a, b) => a.path.localeCompare(b.path));
    if (requests.length > 100) throw Error('More than 100 requests; archive completed requests first.');
    let blocked = 0;
    for (const b of requests) {
      let r;
      try {
        r = parseRequest(b.path, this.decode(this.get(`repos/${FORK}/git/blobs/${b.sha}`)));
        const kind = r.kind;
        const old = this.receipts.get(`receipts/${kind === 'claim' ? 'claims' : 'prs'}/${r.branch}.json`);
        if (old?.request_blob === b.sha && ['released', 'completed', 'posted', 'already_closed'].includes(old.status)) continue;
        if (kind === 'claim' && r.action === 'claim' && ['granted', 'suspended'].includes(old?.status) && old.request_blob === b.sha) continue;
        if (kind === 'claim') this.claim(r, b.sha); else this.publish(r, b.sha);
      } catch (e) {
        blocked++;
        const kind = b.path.startsWith('claims/') ? 'claim' : 'pr';
        const branch = b.path.slice(kind === 'claim' ? 7 : 4, -3);
        const old = this.receipts.get(`receipts/${kind === 'claim' ? 'claims' : 'prs'}/${branch}.json`);
        console.error(`${b.path}: ${e.message}`);
        if (/^[A-Za-z0-9][A-Za-z0-9._/-]*$/.test(branch) && !branch.includes('..')) this.save(kind, branch, {
          ...(old && ACTIVE.has(old.status) ? {...old, rejected_request_blob: b.sha} : {
            ...(old || {}), ...(r ? {request_id: r.request_id, agent: r.agent, issue: r.issue} : {}), branch, request_blob: b.sha, status: 'blocked',
          }), error: e.message,
        });
      }
    }
    console.log(JSON.stringify({scanned: requests.length, blocked, dry_run: this.dryRun, at: stamp()}));
  }
}

if (process.argv[1] && import.meta.url === pathToFileURL(process.argv[1]).href) {
  try {
    const dry = process.argv.includes('--dry-run');
    const p = new Publisher(ghApi, dry);
    const {syncBase, relayFeedback} = await import('./relay.mjs');
    if (!dry) syncBase(ghApi);
    p.run();
    if (!dry) {
      relayFeedback(p);
      const {syncArtifacts} = await import('./artifact-inbox.mjs');
      syncArtifacts(ghApi);
    }
  }
  catch (e) { console.error(e.message); process.exitCode = 1; }
}
