// Tests for scripts/engine-build-check.mjs. The ok, stale-source and stale cases use this
// repository's real git history (79304e9 is older and before the HNSW change, so src/ changed
// since). The warn case builds a throwaway git repository, so it does not depend on whether
// the real history changed src/ lately. Run: node --test test/test-engine-build-check.mjs
import { test } from 'node:test';
import assert from 'node:assert/strict';
import { execFileSync } from 'node:child_process';
import fs from 'node:fs';
import os from 'node:os';
import path from 'node:path';
import { fileURLToPath } from 'node:url';
import { checkEngineBuild } from '../scripts/engine-build-check.mjs';

const repoRoot = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..');
const head = execFileSync('git', ['rev-parse', 'HEAD'], { cwd: repoRoot, encoding: 'utf8' }).trim();
const FUTURE = Math.floor(Date.now() / 1000) + 10 ** 9; // built "after" every tracked file

const info = (gitHash, builtAtUnix = FUTURE) => ({ version: '0.3.7', gitHash, gitDirty: 'false', builtAtUnix });
const run = (buildInfo) => checkEngineBuild({ repoRoot, buildInfo });

test('binary built from HEAD is ok', () => {
  assert.equal(run(info(head.slice(0, 7))).status, 'ok');
});

test('binary built from HEAD but sources edited after the build is stale-source', () => {
  assert.equal(run(info(head.slice(0, 7), 1)).status, 'stale-source');
});

test('older binary whose commits changed no engine input is warn', () => {
  const dir = fs.mkdtempSync(path.join(os.tmpdir(), 'engine-build-check-'));
  try {
    const g = (...args) => execFileSync('git', ['-c', 'user.name=t', '-c', 'user.email=t@example.com', ...args], { cwd: dir, encoding: 'utf8' }).trim();
    g('init', '-q');
    fs.mkdirSync(path.join(dir, 'src'));
    fs.writeFileSync(path.join(dir, 'src', 'a.rs'), 'fn a() {}' + String.fromCharCode(10));
    g('add', '.');
    g('commit', '-q', '-m', 'engine input');
    const built = g('rev-parse', 'HEAD').slice(0, 7);
    fs.writeFileSync(path.join(dir, 'NOTES.md'), 'no engine input here' + String.fromCharCode(10));
    g('add', '.');
    g('commit', '-q', '-m', 'docs only');
    assert.equal(checkEngineBuild({ repoRoot: dir, buildInfo: info(built) }).status, 'warn');
  } finally {
    fs.rmSync(dir, { recursive: true, force: true });
  }
});

test('older binary across an engine input change is stale', () => {
  assert.equal(run(info('79304e9')).status, 'stale');
});

test('binary built without git is unknown, not ok', () => {
  assert.equal(run(info('unknown')).status, 'unknown');
});

test('binary whose build commit is not in this repository is unknown', () => {
  assert.equal(run(info('0000000')).status, 'unknown');
});

test('binary without buildInfo is unknown', () => {
  assert.equal(run(undefined).status, 'unknown');
});
