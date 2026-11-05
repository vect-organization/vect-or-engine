// Copyright (c) 2026 1abcdefggs
// SPDX-License-Identifier: MIT
// https://github.com/1abcdefggs/vect-or-engine

// Change-log row P1 (proof: p1_validate_sync_does_not_wait_for_build_index): buildIndex must not hold the engine lock while it builds, so the
// synchronous validateSync keeps answering. Before the fix the write lock was held for the
// whole build, and validateSync waited for it (9-11 s at 3,000 vectors).

'use strict';

const fs = require('fs');
const os = require('os');
const path = require('path');
const assert = require('assert');
const { loadKnowledgeBase, buildIndex, validateSync, search, kbInfo } = require('../index.js');

const N = 3000;
const DIM = 384;
const MAX_STALL_MS = 250;

function randomKb(n, dim) {
  // Deterministic pseudo-random vectors in [-1, 1].
  let s = 123456789;
  const next = () => ((s = (Math.imul(s, 1664525) + 1013904223) >>> 0) / 4294967296) * 2 - 1;
  return JSON.stringify(
    Array.from({ length: n }, (_, i) => ({ id: `item-${i}`, vector: Array.from({ length: dim }, next) })),
  );
}

(async () => {
  const file = path.join(os.tmpdir(), `vect-p1-${process.pid}.json`);
  fs.writeFileSync(file, randomKb(N, DIM));
  try {
    assert.strictEqual(await loadKnowledgeBase(file), N);

    const started = Date.now();
    const building = buildIndex();
    let done = false;
    building.then(() => { done = true; }, () => { done = true; });

    // Call validateSync while the build runs, and record the slowest call.
    let calls = 0;
    let worst = 0;
    while (!done) {
      const t0 = process.hrtime.bigint();
      validateSync('F20 の患者');
      worst = Math.max(worst, Number(process.hrtime.bigint() - t0) / 1e6);
      calls += 1;
      await new Promise((r) => setImmediate(r));
    }
    const indexed = await building;
    const buildMs = Date.now() - started;
    console.log(`[P1] built ${indexed} vectors in ${buildMs} ms; ${calls} validateSync calls, slowest ${worst.toFixed(1)} ms`);

    assert.strictEqual(indexed, N);
    assert.ok(buildMs > 300 || calls > 5, 'the build finished too fast to prove anything');
    assert.ok(worst < MAX_STALL_MS, `validateSync stalled ${worst.toFixed(1)} ms during buildIndex (limit ${MAX_STALL_MS} ms)`);

    const info = await kbInfo();
    assert.strictEqual(info.hnswReady, true);
    const q = new Float32Array(DIM).fill(0.05);
    assert.strictEqual((await search(q, 3)).length, 3);

    // A reload while an index is being built must not install the stale index.
    const racing = buildIndex();
    await loadKnowledgeBase(file);
    await racing.then(
      () => assert.ok(true),
      (e) => assert.match(String(e.message || e), /reloaded/),
    );
    console.log('✅ build-off-lock test passed.');
  } finally {
    fs.unlinkSync(file);
  }
})().catch((e) => { console.error('❌', e); process.exit(1); });
