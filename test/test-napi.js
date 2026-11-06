// Copyright (c) 2026 1abcdefggs
// SPDX-License-Identifier: MIT
// https://github.com/1abcdefggs/vect-or-engine

const assert = require('assert');
const path = require('path');
const fs = require('fs');
const {
    ping,
    validateSync,
    loadKnowledgeBase,
    buildIndex,
    search,
    kbInfo,
} = require('../index');

function testPing() {
    const response = ping('world');
    console.log(`[Test] ping('world') => "${response}"`);
    assert.strictEqual(response, 'pong from rust: world!');
    console.log('✅ ping test passed.');
}

async function testValidation() {
    console.log('\n--- Testing Validation ---');

    // With the default (empty) profile, any text should be valid.
    const clean_text = "this is a clean message";
    let result = validateSync(clean_text);
    console.log(`[Test] validateSync('${clean_text}') =>`, result);
    assert.strictEqual(result.isValid, true, "Clean text with empty profile should be valid.");
    assert.strictEqual(result.markers.length, 0, "Empty profile should produce no markers.");
    console.log('✅ validation tests passed.');
}

async function testKnowledgeStore() {
    console.log('\n--- Testing KnowledgeStore ---');
    const kbPath = path.resolve(__dirname, 'test-kb.json');

    const loadCount = await loadKnowledgeBase(kbPath);
    assert.strictEqual(loadCount, 3, 'Should load 3 items');

    const indexCount = await buildIndex();
    assert.strictEqual(indexCount, 3, 'Should index 3 items');

    const info = await kbInfo();
    assert.strictEqual(info.count, 3);
    assert.strictEqual(info.dim, 3);
    assert.strictEqual(info.hnswReady, true);
    // A1: vec-1 has length 0.927, so one vector is rescaled and the deviation is reported.
    assert.strictEqual(info.rescaledCount, 1);
    assert(Math.abs(info.maxNormDeviation - 0.0725) < 0.001, `deviation ${info.maxNormDeviation}`);

    const query = new Float32Array([0.1, 0.9, 0.2]);
    const results = await search(query, 1);
    assert.strictEqual(results.length, 1, 'Should return 1 result');
    assert.strictEqual(results[0].id, 'vec-1', 'The nearest vector should be vec-1');
    assert(results[0].score > 0.85, `Score should be close to 1 (cosine), got ${results[0].score}`);

    console.log('✅ KnowledgeStore tests passed.');
}

async function main() {
    testPing();
    await testValidation();
    await testKnowledgeStore();
}

main().catch(err => {
    console.error('Test failed:', err);
    process.exit(1);
});
