// Change-log R3: with SOURCE_DATE_EPOCH set, two builds of the same source give the same binary.
// Builds the library with the `napi` feature (the only build that embeds the build time) twice,
// forcing build.rs to rerun in between, and compares the SHA-256 of the result.
// Usage: node verification/check-reproducible-build.mjs   (run from the repository root)
// On MSVC the linker stamps the build time into the PE header unless /Brepro is given, so the
// script adds it for this check only. It does not change the normal build.
import { spawnSync } from 'node:child_process';
import { createHash } from 'node:crypto';
import { readFileSync, utimesSync } from 'node:fs';

const FILE = { win32: 'vect_or_engine.dll', darwin: 'libvect_or_engine.dylib' }[process.platform] ?? 'libvect_or_engine.so';
const OUT = `target/debug/${FILE}`;
const env = { ...process.env, SOURCE_DATE_EPOCH: '1700000000' };
if (process.platform === 'win32') env.RUSTFLAGS = `${env.RUSTFLAGS ?? ''} -C link-arg=/Brepro`.trim();

function build() {
  const r = spawnSync('cargo', ['build', '--lib', '--features', 'napi'], { env, stdio: 'inherit', shell: process.platform === 'win32' });
  if (r.status !== 0) { console.error('build failed'); process.exit(2); }
  return createHash('sha256').update(readFileSync(OUT)).digest('hex');
}

const first = build();
const now = new Date();
utimesSync('build.rs', now, now); // make cargo rerun the build script
await new Promise((r) => setTimeout(r, 2000)); // a different "now" for a build without the variable
const second = build();
console.log(`first : ${first}\nsecond: ${second}`);
if (first !== second) { console.error('FAIL: builds differ'); process.exit(1); }
console.log('OK: identical');
