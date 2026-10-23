// Maintainer-side GitHub -> git feedback relay. Agents need only their fork deploy key.
import {spawnSync} from 'node:child_process';
import {createHash} from 'node:crypto';
import {FORK, UPSTREAM, QUEUE} from './publish.mjs';

export function putText(api, path, text) {
  const route = `repos/${FORK}/contents/${encodeURIComponent(path)}`;
  let old;
  try { old = api(`${route}?ref=${QUEUE}`); } catch (e) { if (!e.notFound) throw e; }
  const blob = createHash('sha1').update(`blob ${Buffer.byteLength(text)}\0`).update(text).digest('hex');
  if (old?.sha === blob || (old?.encoding === 'base64' && Buffer.from(old.content, 'base64').toString('utf8') === text)) return false;
  api(route, 'PUT', {branch: QUEUE, message: `queue: refresh ${path}`, content: Buffer.from(text).toString('base64'), ...(old ? {sha: old.sha} : {})});
  return true;
}
export const putJson = (api, path, value) => putText(api, path, JSON.stringify(value, null, 2) + '\n');
export function pages(api, route, max = 10) {
  const all = [];
  for (let page = 1; page <= max; page++) {
    const part = api(`${route}${route.includes('?') ? '&' : '?'}per_page=100&page=${page}`);
    if (!Array.isArray(part)) throw Error('Expected a paginated GitHub array.');
    all.push(...part);
    if (part.length < 100) return all;
  }
  throw Error('Feedback pagination bound reached; ask the maintainer for the remaining context.');
}
export function jobLog(route) {
  const p = spawnSync('/usr/bin/gh', ['api', route], {encoding: 'utf8', timeout: 60000, maxBuffer: 16 * 1024 * 1024});
  if (p.error) throw p.error;
  if (p.status !== 0) throw Error((p.stderr || 'Cannot download completed job log').slice(0, 1000));
  return p.stdout;
}
const person = c => ({author: c.user?.login, body: c.body, created_at: c.created_at, updated_at: c.updated_at, url: c.html_url});
export function syncBase(api) {
  const upstream = api(`repos/${UPSTREAM}/git/ref/heads/develop`);
  let fork;
  try { fork = api(`repos/${FORK}/git/ref/heads/upstream-develop`); } catch (e) { if (!e.notFound) throw e; }
  if (!fork) api(`repos/${FORK}/git/refs`, 'POST', {ref: 'refs/heads/upstream-develop', sha: upstream.object.sha});
  else if (fork.object.sha !== upstream.object.sha) api(`repos/${FORK}/git/refs/heads/upstream-develop`, 'PATCH', {sha: upstream.object.sha, force: false});
  putJson(api, 'feedback/base.json', {upstream: `${UPSTREAM}:develop`, mirror: `${FORK}:upstream-develop`, sha: upstream.object.sha});
}
export function relayFeedback(publisher, readLog = jobLog) {
  const api = publisher.api;
  const issues = new Set();
  const pulls = new Map();
  for (const [path, receipt] of publisher.receipts) {
    if (path.startsWith('receipts/claims/') && receipt.issue) issues.add(receipt.issue);
    if (path.startsWith('receipts/prs/') && receipt.pr) {
      const n = /^https:\/\/github\.com\/tangency-labs\/fieldcraft\/pull\/(\d+)$/.exec(receipt.pr);
      if (n) pulls.set(receipt.branch, Number(n[1]));
    }
  }
  let errors = 0;
  try {
    const catalog = pages(api, `repos/${UPSTREAM}/issues?state=open`).filter(i => !i.pull_request).map(i => ({number: i.number, title: i.title, body: i.body, state: i.state, updated_at: i.updated_at, labels: i.labels.map(l => l.name), assignees: i.assignees.map(a => a.login), url: i.html_url}));
    putJson(api, 'feedback/issues/index.json', catalog);
  } catch (e) { errors++; putJson(api, 'feedback/errors/issue-catalog.json', {error: e.message}); }
  for (const issue of issues) {
    try {
      const i = api(`repos/${UPSTREAM}/issues/${issue}`);
      const comments = pages(api, `repos/${UPSTREAM}/issues/${issue}/comments`).map(person);
      putJson(api, `feedback/issues/${issue}.json`, {number: issue, title: i.title, body: i.body, state: i.state, updated_at: i.updated_at, assignees: i.assignees.map(a => a.login), labels: i.labels.map(l => l.name), url: i.html_url, comments});
    } catch (e) { errors++; putJson(api, `feedback/errors/issue-${issue}.json`, {error: e.message}); }
  }
  for (const [branch, n] of pulls) {
    try {
      const pr = api(`repos/${UPSTREAM}/pulls/${n}`);
      const comments = pages(api, `repos/${UPSTREAM}/issues/${n}/comments`).map(person);
      const reviews = pages(api, `repos/${UPSTREAM}/pulls/${n}/reviews`).map(r => ({...person(r), state: r.state, submitted_at: r.submitted_at, commit_id: r.commit_id}));
      const inline = pages(api, `repos/${UPSTREAM}/pulls/${n}/comments`).map(c => ({...person(c), path: c.path, line: c.line, original_line: c.original_line, side: c.side, commit_id: c.commit_id, in_reply_to_id: c.in_reply_to_id}));
      const response = api(`repos/${UPSTREAM}/commits/${pr.head.sha}/check-runs?per_page=100`);
      if (response.total_count > 100) throw Error('Check-run list is truncated; manual relay required.');
      const checks = [];
      for (const check of response.check_runs) {
        const item = {id: check.id, name: check.name, status: check.status, conclusion: check.conclusion, started_at: check.started_at, completed_at: check.completed_at, url: check.details_url, output: check.output};
        const job = /\/actions\/runs\/(\d+)\/job\/(\d+)/.exec(check.details_url || '');
        if (check.status === 'completed' && ['failure', 'timed_out', 'cancelled'].includes(check.conclusion) && job) {
          const path = `feedback/logs/${branch}/${pr.head.sha}/${job[1]}-${job[2]}.txt`;
          try {
            const raw = readLog(`repos/${UPSTREAM}/actions/jobs/${job[2]}/logs`);
            const limit = 2 * 1024 * 1024;
            let text = `PR #${n}; head ${pr.head.sha}\nCheck: ${check.name}\nSource: ${check.details_url}\nCompleted: ${check.completed_at}\n\n`;
            if (Buffer.byteLength(raw) > limit) {
              const bytes = Buffer.from(raw); const half = limit / 2;
              text += bytes.subarray(0, half).toString('utf8') + '\n\n[LOG TRUNCATED: middle omitted to keep git text below 2 MiB; ask the maintainer for the full log.]\n\n' + bytes.subarray(-half).toString('utf8');
              item.log_truncated = true;
            } else { text += raw; item.log_truncated = false; }
            putText(api, path, text); item.log = path;
          } catch (e) { item.log_error = e.message; }
        }
        checks.push(item);
      }
      putJson(api, `feedback/prs/${branch}.json`, {number: n, branch, head_sha: pr.head.sha, state: pr.state, merged: pr.merged, draft: pr.draft, title: pr.title, body: pr.body, updated_at: pr.updated_at, url: pr.html_url, comments, reviews, inline_comments: inline, checks});
    } catch (e) { errors++; putJson(api, `feedback/errors/pr-${n}.json`, {error: e.message}); }
  }
  console.log(JSON.stringify({feedback: {issues: issues.size, pulls: pulls.size, errors}}));
}
