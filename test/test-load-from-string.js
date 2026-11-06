// Copyright (c) 2026 1abcdefggs
// SPDX-License-Identifier: MIT
// https://github.com/1abcdefggs/vect-or-engine

// Change-log rows S1 / P8 (proof: s1_load_from_string_writes_no_file): a knowledge base held in
// memory loads through loadKnowledgeBaseFromString without any file being created, and answers
// the same as the file path. Rows P7 / R4 (proofs: p7_cache_save_is_atomic,
// r4_cache_header_mismatch_rejected) are checked through the Node API as well.

'use strict';

const fs = require('fs');
const os = require('os');
const path = require('path');
const assert = require('assert');
const { loadKnowledgeBase, loadKnowledgeBaseFromString, search, kbInfo, saveKbCache, loadKbCache } = require('../index.js');

(async () => {
  const json = JSON.stringify([
    { id: 'a', vector: [1, 0, 0, 0] },
    { id: 'b', vector: [0, 1, 0, 0] },
    { id: 'c', vector: [0, 0, 1, 0] },
  ]);
  const q = new Float32Array([0, 1, 0, 0]);

  // s1_load_from_string_writes_no_file: the temp directory is the same before and after.
  const dir = fs.mkdtempSync(path.join(os.tmpdir(), 'vect-s1-'));
  const before = fs.readdirSync(os.tmpdir()).length;
  assert.strictEqual(await loadKnowledgeBaseFromString(json), 3);
  assert.strictEqual(fs.readdirSync(os.tmpdir()).length, before, 'loading from a string must not create a file');
  const fromString = await search(q, 3);

  const file = path.join(dir, 'kb.json');
  fs.writeFileSync(file, json);
  assert.strictEqual(await loadKnowledgeBase(file), 3);
  const fromFile = await search(q, 3);
  assert.deepStrictEqual(fromString, fromFile);
  assert.strictEqual(fromString[0].id, 'b');

  // A bad string fails and keeps the loaded knowledge base.
  await assert.rejects(loadKnowledgeBaseFromString('not json'));
  await assert.rejects(loadKnowledgeBaseFromString('[]'));
  assert.strictEqual((await kbInfo()).count, 3);

  // p7_cache_save_is_atomic / r4_cache_header_mismatch_rejected through the Node API.
  const db = path.join(dir, 'kb.db');
  await saveKbCache(db, 'model-a');
  assert.deepStrictEqual(fs.readdirSync(dir).sort(), ['kb.db', 'kb.json'], 'no temporary file is left');
  assert.strictEqual(await loadKbCache(db, 'model-a'), 3);
  await assert.rejects(loadKbCache(db, 'model-b'), /model/);
  assert.strictEqual((await kbInfo()).count, 3);

  fs.rmSync(dir, { recursive: true, force: true });
  console.log('ok test-load-from-string');
})().catch((e) => {
  console.error(e);
  process.exit(1);
});
