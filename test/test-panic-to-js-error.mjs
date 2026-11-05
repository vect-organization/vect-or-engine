// Change-log S5: a Rust panic in a synchronous export reaches JavaScript as an Error and the
// host process keeps running. Builds the library with the `test-hooks` feature (which adds an
// export that panics on purpose), loads it as a .node file, and calls it.
// Usage: node --test test/test-panic-to-js-error.mjs   (slow: it runs cargo build)
import test from 'node:test';
import assert from 'node:assert/strict';
import { spawnSync } from 'node:child_process';
import { copyFileSync, mkdtempSync } from 'node:fs';
import { createRequire } from 'node:module';
import { tmpdir } from 'node:os';
import { join } from 'node:path';

const LIB = { win32: 'vect_or_engine.dll', darwin: 'libvect_or_engine.dylib' }[process.platform] ?? 'libvect_or_engine.so';

test('a forced panic becomes a JS error and the process survives', () => {
  const r = spawnSync('cargo', ['build', '--lib', '--features', 'test-hooks'], { stdio: 'inherit', shell: process.platform === 'win32' });
  assert.equal(r.status, 0, 'cargo build failed');
  const node = join(mkdtempSync(join(tmpdir(), 'vect-s5-')), 'engine.node');
  copyFileSync(join('target', 'debug', LIB), node);
  const engine = createRequire(import.meta.url)(node);

  assert.throws(() => engine.forcePanicForTest(), (e) => e instanceof Error && /forced panic for test/.test(e.message));
  // The process is alive and the engine still answers after the panic.
  assert.equal(engine.ping('x'), 'pong from rust: x!');
});
