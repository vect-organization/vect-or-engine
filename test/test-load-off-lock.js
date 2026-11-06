// Copyright (c) 2026 1abcdefggs
// SPDX-License-Identifier: MIT
// https://github.com/1abcdefggs/vect-or-engine

// Parsing a large knowledge base must not hold the engine lock: loadKnowledgeBaseFromString and
// loadKnowledgeBase parse first and take the write lock only to install the result, so the
// synchronous validateSync keeps answering while a large JSON text loads.

'use strict';

const assert = require('assert');
const { loadKnowledgeBaseFromString, validateSync, kbInfo } = require('../index.js');

const N = Number(process.env.VECT_LOAD_N || 100000);
const DIM = 32;
const MAX_STALL_MS = 100;

function randomKbText(n, dim) {
  let s = 987654321;
  const next = () => (((s = (Math.imul(s, 1664525) + 1013904223) >>> 0) / 4294967296) * 2 - 1).toFixed(6);
  const rows = new Array(n);
  for (let i = 0; i < n; i++) {
    const v = new Array(dim);
    for (let j = 0; j < dim; j++) v[j] = next();
    rows[i] = `{"id":"item-${i}","vector":[${v.join(',')}]}`;
  }
  return `[${rows.join(',')}]`;
}

(async () => {
  const text = randomKbText(N, DIM);
  const started = Date.now();
  const loading = loadKnowledgeBaseFromString(text);
  let done = false;
  loading.then(() => { done = true; }, () => { done = true; });

  let calls = 0;
  let worst = 0;
  while (!done) {
    const t0 = process.hrtime.bigint();
    validateSync('F20 の患者');
    worst = Math.max(worst, Number(process.hrtime.bigint() - t0) / 1e6);
    calls += 1;
    await new Promise((r) => setImmediate(r));
  }
  const count = await loading;
  const loadMs = Date.now() - started;
  console.log(`[load-off-lock] ${(text.length / 1e6).toFixed(0)} MB, ${count} vectors in ${loadMs} ms; ${calls} validateSync calls, slowest ${worst.toFixed(1)} ms`);

  assert.strictEqual(count, N);
  assert.ok(loadMs > 300 || calls > 5, 'the load finished too fast to prove anything');
  assert.ok(worst < MAX_STALL_MS, `validateSync stalled ${worst.toFixed(1)} ms during the load (limit ${MAX_STALL_MS} ms)`);
  assert.strictEqual((await kbInfo()).count, N);
  console.log('ok test-load-off-lock');
})().catch((e) => { console.error(e); process.exit(1); });
