import test from 'node:test';
import assert from 'node:assert/strict';
import {mediaRecord, renderMedia, postMedia, postReadyMedia} from './post-media.mjs';
const sha = 'a'.repeat(40), old = 'b'.repeat(40), hash = 'c'.repeat(64);
const url = 'https://github.com/user-attachments/assets/11111111-2222-3333-4444-555555555555';
const pr = {state: 'open', body: 'Problem\n\n## Demonstration\n\nCapture provenance.\n', head: {ref: 'fix/example', sha, repo: {full_name: 'bluecmd/fieldcraft'}}, base: {ref: 'develop', repo: {full_name: 'tangency-labs/fieldcraft'}}};
const record = {status: 'received', branch: pr.head.ref, head_sha: sha, name: 'demo.mp4', bytes: 1234, sha256: hash, verified_sha256: hash, github_attachment: 'pending_maintainer'};
test('only verified media for the destination/capture can be uploaded', () => {
  assert.equal(mediaRecord(record, pr, sha).mime, 'video/mp4');
  for (const change of [{name: '../demo.mp4'}, {name: 'qa.json'}, {bytes: 11 * 1024 * 1024}, {verified_sha256: 'd'.repeat(64)}, {branch: 'another'}, {status: 'blocked'}]) assert.throws(() => mediaRecord({...record, ...change}, pr, sha));
  assert.throws(() => mediaRecord(record, {...pr, state: 'closed'}, sha));
});
test('historical captures require a maintainer applicability review', () => {
  const historic = {...record, head_sha: old};
  assert.throws(() => mediaRecord(historic, pr, old));
  assert.equal(mediaRecord(historic, pr, old, 'Only tests changed after this capture.').head_sha, old);
});
test('native player URLs and image embeds are idempotent and retain original limitations', () => {
  const files = [{...record, mime: 'video/mp4', github_url: url}, {...record, name: 'still.png', mime: 'image/png', github_url: url}];
  for (const body of [pr.body, 'Problem\n\n## Known limitations\n\nNot a transient.\n']) {
    const result = renderMedia(body, files, sha, 'Actual source pin.');
    assert.equal(renderMedia(result, files, sha, 'Actual source pin.'), result);
    assert.ok(result.includes('\n\n'+url+'\n\n'));
    assert.ok(result.includes('![still.png]('+url+')'));
    assert.ok(result.includes(body.includes('limitations') ? 'Not a transient.' : 'Capture provenance.'));
  }
});
function fixture({closed = false, moved = false} = {}) {
  let state = {...pr, state: closed ? 'closed' : 'open'}, reads = 0; const writes = [];
  const api = (route, method, data) => {
    if (method) {
      writes.push({route, method, data});
      if (method === 'PATCH') state = {...state, body: data.body};
      return {};
    }
    if (route.includes('/pulls/704')) { reads++; return {...state, head: {...state.head, sha: moved && reads > 1 ? old : sha}}; }
    if (route.endsWith('/git/ref/heads/pr-queue')) return {object: {sha: 'queue'}};
    if (route.includes('/git/trees/')) return {tree: [{type: 'blob', path: `receipts/artifacts/${pr.head.ref}/${sha}/demo.mp4.json`, sha: 'blob', size: 500}]};
    if (route.endsWith('/git/blobs/blob')) return {encoding: 'base64', content: Buffer.from(JSON.stringify(record)).toString('base64')};
    if (route === 'repos/bluecmd/fieldcraft') return {id: 123, private: false, permissions: {push: true}};
    const e = Error('not found'); e.notFound = true; throw e;
  };
  return {api, writes};
}
test('receipts become attached only after downloaded bytes and PR publication are verified', () => {
  const failed = fixture();
  assert.throws(() => postMedia(failed.api, 704, {upload: () => { throw Error('hash mismatch'); }}));
  assert.equal(failed.writes.length, 0);
  const ok = fixture(); const result = postMedia(ok.api, 704, {upload: () => url});
  assert.equal(result.videos, 1);
  assert.equal(ok.writes[0].method, 'PATCH');
  const receipt = JSON.parse(Buffer.from(ok.writes[1].data.content, 'base64'));
  assert.equal(receipt.github_attachment, 'attached'); assert.equal(receipt.github_url, url);
});
test('closed PRs and moving heads cannot acquire published media or attached receipts', () => {
  const closed = fixture({closed: true});
  assert.throws(() => postMedia(closed.api, 704, {upload: () => assert.fail('closed PR uploaded')}));
  assert.equal(closed.writes.length, 0);
  const moved = fixture({moved: true});
  assert.throws(() => postMedia(moved.api, 704, {upload: () => url}));
  assert.equal(moved.writes.length, 0);
});

test('automatic upload accepts current-head media only and skips already embedded files', () => {
  const f = fixture(); let uploads = 0;
  postReadyMedia(f.api, [{...pr, number: 704}], () => { uploads++; return {pr: 704}; });
  assert.equal(uploads, 1);
  const history = fixture();
  postReadyMedia(history.api, [{...pr, number: 704, head: {...pr.head, sha: old}}], () => assert.fail('historical capture uploaded automatically'));
  const done = fixture();
  const api = (...args) => {
    if (args[0].endsWith('/git/blobs/blob')) return {encoding: 'base64', content: Buffer.from(JSON.stringify({...record, github_attachment: 'attached', github_url: url})).toString('base64')};
    return done.api(...args);
  };
  postReadyMedia(api, [{...pr, number: 704, body: pr.body+url}], () => assert.fail('attached asset uploaded twice'));
});
