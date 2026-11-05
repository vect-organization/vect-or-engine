import { test } from 'node:test';
import assert from 'node:assert/strict';
import fs from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';
import { checkChangeLog, parseCsv, proofTestName, COLUMNS } from '../scripts/check-change-log.mjs';

const repo = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..');
const head = COLUMNS.join(',');
const row = (o) => COLUMNS.map((c) => o[c] ?? '').join(',');
const base = { id: 'D1', area: 'defect', why: 'w', status: 'planned' };

test('the committed change logs are valid', () => {
  const dir = path.join(repo, 'docs', 'changes');
  for (const f of fs.readdirSync(dir).filter((x) => x.endsWith('.csv'))) {
    assert.deepEqual(checkChangeLog(fs.readFileSync(path.join(dir, f), 'utf8'), repo, f), []);
  }
});

test('parseCsv handles quotes, doubled quotes and embedded commas', () => {
  assert.deepEqual(parseCsv('a,"b,c","d ""e"""\r\nx,y,z\n'), [['a', 'b,c', 'd "e"'], ['x', 'y', 'z']]);
});

test('proofTestName takes the leading identifier and ignores prose', () => {
  assert.equal(proofTestName('d3_empty_pattern_rejected (inverts f1)'), 'd3_empty_pattern_rejected');
  assert.equal(proofTestName('Review'), null);
});

test('rejects a wrong header, a duplicate id and an unknown status', () => {
  assert.ok(checkChangeLog('id,area\nD1,defect\n', repo).some((p) => p.includes('header')));
  const dup = [head, row(base), row(base)].join('\n');
  assert.ok(checkChangeLog(dup, repo).some((p) => p.includes('duplicate id D1')));
  const bad = [head, row({ ...base, status: 'finished' })].join('\n');
  assert.ok(checkChangeLog(bad, repo).some((p) => p.includes('status "finished"')));
});

test('a dropped row needs a why; a done row needs shipped_in and an existing proof test', () => {
  assert.ok(checkChangeLog([head, row({ ...base, why: '', status: 'dropped' })].join('\n'), repo).some((p) => p.includes('why')));
  const noShip = checkChangeLog([head, row({ ...base, status: 'done', proof: ['absent', 'proof', 'name'].join('_') })].join('\n'), repo);
  assert.ok(noShip.some((p) => p.includes('shipped_in')));
  assert.ok(noShip.some((p) => p.includes('not found')));
  // a real identifier from this very file counts as found
  const ok = checkChangeLog([head, row({ ...base, status: 'done', shipped_in: '0.3.8', proof: 'noShip_exists' })].join('\n'), repo);
  assert.deepEqual(ok, []);
});
