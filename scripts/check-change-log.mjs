// Validates docs/changes/*.csv, the change-log source of truth for a release.
// Rules: fixed columns, unique ids, known status values, and every `done` row names
// a shipped_in version and a proof test that exists in the repository.
// Usage: node scripts/check-change-log.mjs [file.csv ...]   (default: docs/changes/*.csv)

import fs from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

export const COLUMNS = ['id', 'area', 'severity', 'before_0_3_7', 'why', 'change', 'proof', 'breaking', 'phase', 'shipped_in', 'status', 'pr'];
export const STATUSES = ['planned', 'in-progress', 'done', 'dropped'];
const PROOF_DIRS = ['src', 'tests', 'test', 'examples', 'scripts'];

/** RFC 4180 style parser: quoted fields, doubled quotes, newlines inside quotes. */
export function parseCsv(text) {
  const rows = [];
  let row = [];
  let field = '';
  let quoted = false;
  for (let i = 0; i < text.length; i++) {
    const ch = text[i];
    if (quoted) {
      if (ch === '"' && text[i + 1] === '"') { field += '"'; i++; }
      else if (ch === '"') quoted = false;
      else field += ch;
    } else if (ch === '"') quoted = true;
    else if (ch === ',') { row.push(field); field = ''; }
    else if (ch === '\n' || ch === '\r') {
      if (ch === '\r' && text[i + 1] === '\n') i++;
      row.push(field);
      field = '';
      if (row.some((c) => c !== '')) rows.push(row);
      row = [];
    } else field += ch;
  }
  if (field !== '' || row.length) {
    row.push(field);
    if (row.some((c) => c !== '')) rows.push(row);
  }
  return rows;
}

function listFiles(dir, out = []) {
  if (!fs.existsSync(dir)) return out;
  for (const e of fs.readdirSync(dir, { withFileTypes: true })) {
    const p = path.join(dir, e.name);
    if (e.isDirectory()) {
      if (e.name !== 'node_modules' && e.name !== 'target') listFiles(p, out);
    } else if (/\.(rs|js|mjs)$/.test(e.name)) out.push(p);
  }
  return out;
}

/** The identifier at the start of a proof cell, e.g. `d3_empty_pattern_rejected (inverts f1)`. */
export function proofTestName(proof) {
  const m = /^[A-Za-z_][A-Za-z0-9_]*/.exec(proof.trim());
  return m && m[0].includes('_') ? m[0] : null;
}

/** Returns a list of problem strings for one CSV text; empty means valid. */
export function checkChangeLog(text, repoRoot, label = 'csv') {
  const problems = [];
  const rows = parseCsv(text);
  if (rows.length === 0) return [`${label}: empty file`];
  if (rows[0].join(',') !== COLUMNS.join(',')) problems.push(`${label}: header must be ${COLUMNS.join(',')}`);
  const seen = new Set();
  let sources = null;
  for (const [n, r] of rows.slice(1).entries()) {
    const at = `${label} row ${n + 2}`;
    if (r.length !== COLUMNS.length) {
      problems.push(`${at}: ${r.length} columns, expected ${COLUMNS.length}`);
      continue;
    }
    const o = Object.fromEntries(COLUMNS.map((c, i) => [c, r[i]]));
    if (!o.id) problems.push(`${at}: empty id`);
    else if (seen.has(o.id)) problems.push(`${at}: duplicate id ${o.id}`);
    seen.add(o.id);
    if (!STATUSES.includes(o.status)) problems.push(`${at} (${o.id}): status "${o.status}" is not one of ${STATUSES.join('/')}`);
    if (o.status === 'dropped' && !o.why) problems.push(`${at} (${o.id}): dropped rows need a why`);
    if (o.status === 'done') {
      if (!o.shipped_in) problems.push(`${at} (${o.id}): done needs shipped_in`);
      const name = proofTestName(o.proof);
      if (!name) problems.push(`${at} (${o.id}): done needs a proof test name`);
      else {
        sources ??= PROOF_DIRS.flatMap((d) => listFiles(path.join(repoRoot, d))).map((f) => fs.readFileSync(f, 'utf8'));
        if (!sources.some((s) => s.includes(name))) problems.push(`${at} (${o.id}): proof test "${name}" not found in ${PROOF_DIRS.join('/')}`);
      }
    }
  }
  return problems;
}

if (process.argv[1] === fileURLToPath(import.meta.url)) {
  const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..');
  let files = process.argv.slice(2);
  if (files.length === 0) {
    const dir = path.join(root, 'docs', 'changes');
    files = fs.existsSync(dir) ? fs.readdirSync(dir).filter((f) => f.endsWith('.csv')).map((f) => path.join(dir, f)) : [];
  }
  const problems = files.flatMap((f) => checkChangeLog(fs.readFileSync(f, 'utf8'), root, path.relative(root, f)));
  for (const p of problems) console.error(p);
  console.log(`[check-change-log] ${files.length} file(s), ${problems.length} problem(s)`);
  process.exit(problems.length ? 1 : 0);
}
