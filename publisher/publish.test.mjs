import test from 'node:test';
import assert from 'node:assert/strict';
import {Publisher, parseRequest, FORK, UPSTREAM} from './publish.mjs';
const sha = 'a'.repeat(40);
const request = {schema: 1, kind: 'pr', action: 'publish', request_id: 'pr-1', agent: 'solver-a', branch: 'feature/flow', issue: 698, head_sha: sha, base: 'develop', draft: false, title: 'CFD: fan curves', media: 'not_needed'};
const body = '## What and why\nFlow diagnostics.\n\n## Validation\nTests pass.\n\n## Known limitations\nNo two-fluid coupling.\n';
const text = (r = request, b = body) => `---\n${JSON.stringify(r, null, 2)}\n---\n${b}`;
const r = () => parseRequest('prs/feature/flow.md', text());
function world({assignees = [], labels = [], head = sha, commits = [], existing = []} = {}) {
  const writes = [];
  const api = (route, method = 'GET', data) => {
    if (method !== 'GET') {
      writes.push({route, method, data});
      if (route.endsWith('/pulls')) return {number: 77, state: 'open', html_url: 'https://github.com/tangency-labs/fieldcraft/pull/77', head: {sha}};
      return {};
    }
    if (route === `repos/${UPSTREAM}/issues/698`) return {number: 698, state: 'open', assignees: assignees.map(login => ({login})), labels: labels.map(name => ({name}))};
    if (route.startsWith(`repos/${UPSTREAM}/issues/698/comments?`)) return [];
    if (route.startsWith(`repos/${FORK}/branches/`)) return {commit: {sha: head}};
    if (route.includes('/compare/')) return {ahead_by: 1, behind_by: 0, commits: [{sha}]};
    if (route.includes('/commits/')) return {files: commits};
    if (route.includes('/pulls?')) return existing;
    if (route.includes('/contents/prs%2F') || route.includes('/contents/claims%2F')) return {sha: 'blob'};
    if (route.includes('/contents/receipts%2F')) { const e = Error('missing'); e.notFound = true; throw e; }
    throw Error(`unexpected read ${route}`);
  };
  const p = new Publisher(api);
  return {p, writes};
}
const claim = () => ({schema: 1, kind: 'claim', action: 'claim', request_id: 'claim-1', agent: 'solver-a', branch: 'feature/flow', issue: 698, path: 'claims/feature/flow.md'});
const grant = p => p.receipts.set('receipts/claims/feature/flow.json', {...claim(), request_blob: 'claim-blob', status: 'granted'});

test('JSON front matter leaves the PR Markdown literal, including shell syntax', () => {
  const b = body + '\nLiteral `$HOME` and `$(whoami)` are documentation.\n';
  assert.equal(parseRequest('prs/feature/flow.md', text(request, b)).body, b);
});
test('rejects wrong branch, missing tested SHA, foreign base, and missing limitations', () => {
  for (const changes of [{branch: 'wrong'}, {head_sha: 'latest'}, {base: 'main'}, {title: 'one\ntwo'}]) assert.throws(() => parseRequest('prs/feature/flow.md', text({...request, ...changes})));
  assert.throws(() => parseRequest('prs/feature/flow.md', text(request, '## Validation\nTests pass.')));
});
test('branch path traversal and queue branch cannot be requested', () => {
  for (const branch of ['../flow', 'x/../flow', 'pr-queue', 'develop', 'x.lock']) assert.throws(() => parseRequest(`prs/${branch}.md`, text({...request, branch})));
});
test('media must use attachments; pending may publish for subsequent PR upload', () => {
  assert.throws(() => parseRequest('prs/feature/flow.md', text({...request, media: 'attached'})));
  assert.throws(() => parseRequest('prs/feature/flow.md', text(request, body + 'https://raw.githubusercontent.com/bluecmd/fieldcraft/flow/docs/video.mp4')));
  assert.equal(parseRequest('prs/feature/flow.md', text({...request, media: 'pending'})).media, 'pending');
  assert.equal(parseRequest('prs/feature/flow.md', text({...request, media: 'attached'}, body + '\nhttps://github.com/user-attachments/assets/abc')).media, 'attached');
});
test('grants an unassigned issue and records the agent, with a visible issue comment', () => {
  const {p, writes} = world(); p.claim(claim(), 'blob');
  assert.equal(p.receipts.get('receipts/claims/feature/flow.json').status, 'granted');
  assert.ok(writes.some(w => w.route.endsWith('/assignees') && w.data.assignees[0] === 'bluecmd'));
  assert.ok(writes.some(w => w.route.endsWith('/comments') && w.data.body.includes('solver-a')));
});
test('another agent reservation blocks a claim even though GitHub uses the same account', () => {
  const {p, writes} = world(); p.receipts.set('receipts/claims/other.json', {status: 'granted', agent: 'other', branch: 'other', issue: 698});
  assert.throws(() => p.claim(claim(), 'blob'), /reserved by other/); assert.equal(writes.length, 0);
});
test('a different GitHub assignee or core-team label blocks automatic takeover', () => {
  for (const options of [{assignees: ['jyviko']}, {labels: ['in progress']}, {labels: ['paused']}, {assignees: ['bluecmd']}]) {
    const {p, writes} = world(options); assert.throws(() => p.claim(claim(), 'blob')); assert.equal(writes.length, 0);
  }
});
test('an open PR on another branch blocks overlapping work', () => {
  const {p, writes} = world(); p.pulls = [{title: 'Fix (#698)', head: {ref: 'other', repo: {full_name: FORK}}}];
  assert.throws(() => p.claim(claim(), 'blob'), /Another open PR/); assert.equal(writes.length, 0);
});
test('cannot change the agent of an active branch reservation', () => {
  const {p, writes} = world(); grant(p);
  assert.throws(() => p.claim({...claim(), agent: 'other'}, 'blob'), /Release the existing/); assert.equal(writes.length, 0);
});
test('release requires original owner and refuses while its PR is open', () => {
  const {p, writes} = world({assignees: ['bluecmd']}); grant(p);
  assert.throws(() => p.claim({...claim(), action: 'release', agent: 'other'}, 'blob'));
  p.pulls = [{head: {ref: 'feature/flow', repo: {full_name: FORK}}}];
  assert.throws(() => p.claim({...claim(), action: 'release'}, 'blob'), /open PR/); assert.equal(writes.length, 0);
});
test('release removes only our account and records a receipt', () => {
  const {p, writes} = world({assignees: ['bluecmd'], labels: ['claimed']}); grant(p);
  p.claim({...claim(), action: 'release'}, 'blob');
  assert.equal(p.receipts.get('receipts/claims/feature/flow.json').status, 'released');
  assert.ok(writes.some(w => w.method === 'DELETE' && w.data?.assignees[0] === 'bluecmd'));
});
test('no PR can be posted before its issue claim is granted', () => {
  const {p, writes} = world({assignees: ['bluecmd']}); assert.throws(() => p.publish(r(), 'blob'), /matching granted/); assert.equal(writes.length, 0);
});
test('stale tested SHA blocks publication before any write', () => {
  const {p, writes} = world({assignees: ['bluecmd'], head: 'b'.repeat(40)}); grant(p);
  assert.throws(() => p.publish(r(), 'blob'), /tested head_sha/); assert.equal(writes.length, 0);
});
test('checks every commit, so a later deletion cannot hide committed video', () => {
  const {p, writes} = world({assignees: ['bluecmd'], commits: [{filename: 'docs/demo.mp4', status: 'added'}]}); grant(p);
  assert.throws(() => p.publish(r(), 'blob'), /every unmerged commit/); assert.equal(writes.length, 0);
});
test('publishes a non-draft PR with the exact title/body and writes its URL', () => {
  const {p, writes} = world({assignees: ['bluecmd']}); grant(p); p.publish(r(), 'blob');
  const post = writes.find(w => w.route.endsWith('/pulls'));
  assert.equal(post.data.body, body); assert.equal(post.data.title, request.title); assert.equal(post.data.draft, false);
  assert.equal(p.receipts.get('receipts/prs/feature/flow.json').status, 'posted');
});
test('existing open PR is updated, not duplicated', () => {
  const existing = [{number: 77, state: 'open', draft: false, title: 'old', body: 'old', html_url: 'url', head: {sha}}];
  const {p, writes} = world({assignees: ['bluecmd'], existing}); grant(p);
  const api = p.api; p.api = (route, method, data) => method === 'PATCH' ? (writes.push({route, method, data}), {...existing[0], ...data}) : api(route, method, data);
  p.publish(r(), 'blob'); assert.ok(writes.some(w => w.method === 'PATCH')); assert.ok(!writes.some(w => w.method === 'POST' && w.route.endsWith('/pulls')));
});
test('closed PR is recorded without reopening or recreating it', () => {
  const {p, writes} = world({assignees: ['bluecmd'], existing: [{state: 'closed', html_url: 'closed-url'}]}); grant(p); p.publish(r(), 'blob');
  assert.equal(p.receipts.get('receipts/prs/feature/flow.json').status, 'already_closed');
  assert.ok(!writes.some(w => w.route.endsWith('/pulls')));
});
test('dry-run never assigns, comments, creates a PR or writes a receipt', () => {
  const {p, writes} = world(); p.dryRun = true; p.claim(claim(), 'blob'); assert.equal(writes.length, 0);
});

test('a head moving during inspection is blocked before creating a PR', () => {
  const {p, writes} = world({assignees: ['bluecmd']}); grant(p); const api = p.api; let reads = 0;
  p.api = (route, method, data) => route.includes('/branches/') ? {commit: {sha: ++reads === 1 ? sha : 'b'.repeat(40)}} : api(route, method, data);
  assert.throws(() => p.publish(r(), 'blob'), /changed during inspection/); assert.equal(writes.length, 0);
});
test('identical receipts do not create empty polling commits', () => {
  const {p, writes} = world(); const record = {status: 'blocked', branch: 'feature/flow', error: 'wait'};
  p.save('claim', 'feature/flow', record); p.save('claim', 'feature/flow', record);
  assert.equal(writes.length, 1);
});
test('run rejects a different authenticated publisher account', () => {
  const p = new Publisher(() => ({login: 'other'})); assert.throws(() => p.run(), /authenticate as bluecmd/);
});
test('a rejected ownership edit cannot overwrite the granted agent in the error receipt', () => {
  const {p, writes} = world({assignees: ['bluecmd']});
  const old = {...claim(), status: 'granted', request_blob: 'old-blob'};
  const incoming = {...claim(), agent: 'intruder', request_id: 'other-claim'};
  const api = p.api;
  p.api = (route, method, data) => {
    if (method && method !== 'GET') return api(route, method, data);
    if (route === 'user') return {login: 'bluecmd'};
    if (route.includes('/git/ref/')) return {object: {sha: 'tree'}};
    if (route.includes('/git/trees/')) return {truncated: false, tree: [
      {type: 'blob', path: 'receipts/claims/feature/flow.json', sha: 'receipt'},
      {type: 'blob', path: 'claims/feature/flow.md', sha: 'incoming'}
    ]};
    if (route.endsWith('/git/blobs/receipt')) return {encoding: 'base64', content: Buffer.from(JSON.stringify(old)).toString('base64')};
    if (route.endsWith('/git/blobs/incoming')) return {encoding: 'base64', content: Buffer.from(text(incoming, 'plan')).toString('base64')};
    if (route.includes('/pulls?state=open')) return [];
    return api(route, method, data);
  };
  p.run(); const receipt = p.receipts.get('receipts/claims/feature/flow.json');
  assert.equal(receipt.agent, 'solver-a'); assert.equal(receipt.request_id, 'claim-1'); assert.equal(receipt.status, 'granted');
  assert.ok(!writes.some(w => w.route.includes(`/repos/${UPSTREAM}`)));
});
