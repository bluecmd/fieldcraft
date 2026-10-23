import test from 'node:test';
import assert from 'node:assert/strict';
import {createHash} from 'node:crypto';
import {relayFeedback, putText, syncBase} from './relay.mjs';
import {manifestRecord, syncArtifacts} from './artifact-inbox.mjs';
const sha = 'a'.repeat(40);
const meta = {schema: 1, agent: 'ui-agent', branch: 'fix/legend-fit-610', head_sha: sha, name: 'demo.mp4', bytes: 1234, sha256: 'b'.repeat(64), description: 'Actual app recording'};
const path = `manifests/${meta.branch}/${sha}/demo.mp4.json`;
const pointer = `version https://git-lfs.github.com/spec/v1\noid sha256:${meta.sha256}\nsize 1234\n`;

test('manifest must match an actual LFS pointer, not an ordinary binary', () => {
  assert.equal(manifestRecord(path, JSON.stringify(meta), pointer).sha256, meta.sha256);
  for (const p of ['binary data', pointer.replace('1234', '1235'), pointer.replace('b'.repeat(64), 'c'.repeat(64))]) assert.throws(() => manifestRecord(path, JSON.stringify(meta), p));
});
test('rejects traversal, private source paths, huge files, mismatched paths and invalid heads', () => {
  for (const change of [{branch: '../../x'}, {name: '../demo.mp4'}, {name: 'C:\\private.mp4'}, {bytes: 101 * 1024 * 1024}, {head_sha: 'HEAD'}]) assert.throws(() => manifestRecord(path, JSON.stringify({...meta, ...change}), pointer));
  assert.throws(() => manifestRecord('manifests/other.json', JSON.stringify(meta), pointer));
});
test('artifact is not marked received until LFS bytes are retrieved and verified', () => {
  const writes = []; const data = (content, size = content.length) => ({type: 'blob', sha: content, size});
  const api = (route, method, value) => {
    if (method) { writes.push({route, value}); return {}; }
    if (route.includes('/git/ref/')) return {object: {sha: 'commit'}};
    if (route.includes('/git/trees/')) return {tree: [{...data('manifest'), path}, {...data('pointer'), path: `objects/${meta.branch}/${sha}/demo.mp4`}]};
    if (route.endsWith('/git/blobs/manifest')) return {encoding: 'base64', content: Buffer.from(JSON.stringify(meta)).toString('base64')};
    if (route.endsWith('/git/blobs/pointer')) return {encoding: 'base64', content: Buffer.from(pointer).toString('base64')};
    const e = Error('missing'); e.notFound = true; throw e;
  };
  syncArtifacts(api, '/unused', () => { throw Error('LFS object unavailable'); });
  assert.ok(writes.some(w => w.route.includes('artifact-errors'))); assert.ok(!writes.some(w => w.route.includes('receipts%2Fartifacts%2F')));
  writes.length = 0; let fetched = false;
  syncArtifacts(api, '/unused', () => { fetched = true; });
  assert.ok(fetched); const received = writes.find(w => w.route.includes('receipts%2Fartifacts%2F'));
  const record = JSON.parse(Buffer.from(received.value.content, 'base64').toString());
  assert.equal(record.status, 'received'); assert.equal(record.github_attachment, 'pending_maintainer'); assert.equal(record.verified_sha256, meta.sha256);
});
test('unchanged feedback does not create polling commits, including files above the contents API inline limit', () => {
  let writes = 0; const text = 'same';
  const hash = createHash('sha1').update(`blob ${Buffer.byteLength(text)}\0`).update(text).digest('hex');
  const api = (route, method) => method ? writes++ : {encoding: 'none', sha: hash};
  assert.equal(putText(api, 'feedback/test.txt', text), false); assert.equal(writes, 0);
});
test('upstream-develop mirror is fast-forward only', () => {
  const writes = [];
  const api = (route, method, data) => {
    if (method) { writes.push({route, method, data}); return {}; }
    if (route.includes('tangency-labs')) return {object: {sha: 'new'}};
    if (route.includes('/git/ref/')) return {object: {sha: 'old'}};
    const e = Error('missing'); e.notFound = true; throw e;
  };
  syncBase(api); assert.equal(writes.find(w => w.method === 'PATCH').data.force, false);
});
test('completed failed job logs are relayed while the rest of the workflow is still running', () => {
  const writes = []; const calls = [];
  const api = (route, method, data) => {
    if (method) { writes.push({route, data}); return {}; }
    if (route.endsWith('/pulls/709')) return {head: {sha}, state: 'open', title: 'Legend', body: 'body', updated_at: 'today'};
    if (route.includes('/check-runs')) return {total_count: 2, check_runs: [
      {id: 1, name: 'CI', status: 'completed', conclusion: 'failure', details_url: 'https://github.com/tangency-labs/fieldcraft/actions/runs/40023518993/job/120654123187'},
      {id: 2, name: 'heavy', status: 'in_progress', conclusion: null}
    ]};
    if (route.includes('/contents/')) { const e = Error('missing'); e.notFound = true; throw e; }
    return [];
  };
  const publisher = {api, receipts: new Map([['receipts/prs/fix/legend-fit-610.json', {pr: 'https://github.com/tangency-labs/fieldcraft/pull/709', branch: 'fix/legend-fit-610'}]])};
  relayFeedback(publisher, route => { calls.push(route); return 'test failed: cancellation took 5.79 seconds'; });
  assert.deepEqual(calls, ['repos/tangency-labs/fieldcraft/actions/jobs/120654123187/logs']);
  const log = writes.find(w => w.route.includes('feedback%2Flogs%2F')); assert.ok(Buffer.from(log.data.content, 'base64').toString().includes('5.79 seconds'));
  const snapshot = writes.find(w => w.route.includes('feedback%2Fprs%2F'));
  const feedback = JSON.parse(Buffer.from(snapshot.data.content, 'base64').toString());
  assert.ok(feedback.checks[0].log.endsWith('.txt')); assert.equal(feedback.checks[1].status, 'in_progress');
});
