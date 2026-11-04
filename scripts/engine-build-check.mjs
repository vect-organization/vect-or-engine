// Checks whether a built vect-or-engine .node matches the repository it sits in.
// Design: docs/PROJECT/05_VECT_OR_ENGINE/05_05_ENGINE_BUILD_INFO_VERIFICATION.md
//
// checkEngineBuild() is pure with respect to the binary: it takes the buildInfo()
// object, so tests can pass fake values. The CLI loads the .node and calls it.
//
// Statuses:
//   ok            binary built from HEAD, no engine input is newer than the build
//   warn          binary built from an older commit, but no engine input changed since
//   stale-source  binary built from HEAD, but a tracked engine input was modified after the build
//   stale         binary built from an older commit that changed engine inputs (rebuild needed)
//   unknown       the binary cannot be tied to a commit (no git, no buildInfo, commit missing)
//
// Exit codes (CLI): 1 for stale and unknown, 0 otherwise.

import { execFileSync } from 'node:child_process';
import fs from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';
import { createRequire } from 'node:module';

// Paths whose change makes a compiled binary differ from the source.
export const ENGINE_INPUTS = ['src', 'lib.rs', 'Cargo.toml', 'build.rs'];

function git(repoRoot, args) {
  try {
    return execFileSync('git', args, {
      cwd: repoRoot,
      encoding: 'utf8',
      stdio: ['ignore', 'pipe', 'ignore'],
    }).trim();
  } catch {
    return null;
  }
}

export function checkEngineBuild({ repoRoot, buildInfo, binaryPath = null }) {
  const result = { status: 'unknown', reason: '', binaryPath, buildInfo, head: null, changedInputs: [], sourceNewer: [] };

  if (!buildInfo || typeof buildInfo !== 'object') {
    result.reason = 'binary does not export buildInfo()';
    return result;
  }

  const head = git(repoRoot, ['rev-parse', 'HEAD']);
  result.head = head ? head.slice(0, 7) : null;
  if (!head) {
    result.reason = 'git is not available in the repository';
    return result;
  }

  const gitHash = String(buildInfo.gitHash ?? '');
  if (!gitHash || gitHash === 'unknown') {
    result.reason = 'binary was built without git (gitHash is unknown)';
    return result;
  }

  // The build commit must exist here, otherwise there is nothing to compare with.
  if (git(repoRoot, ['cat-file', '-e', `${gitHash}^{commit}`]) === null) {
    result.reason = `build commit ${gitHash} is not in this repository`;
    return result;
  }

  // Tracked engine inputs modified after the binary was built (covers uncommitted edits).
  const tracked = (git(repoRoot, ['ls-files', '--', ...ENGINE_INPUTS]) ?? '').split('\n').filter(Boolean);
  const builtAtMs = Number(buildInfo.builtAtUnix) * 1000;
  for (const file of tracked) {
    try {
      const st = fs.statSync(path.join(repoRoot, file));
      if (Number.isFinite(builtAtMs) && st.mtimeMs > builtAtMs) result.sourceNewer.push(file);
    } catch {
      // A deleted tracked file is shown by git status; not a timing question.
    }
  }

  const sameCommit = head.startsWith(gitHash);
  if (!sameCommit) {
    const diff = git(repoRoot, ['diff', '--name-only', gitHash, 'HEAD', '--', ...ENGINE_INPUTS]) ?? '';
    result.changedInputs = diff.split('\n').filter(Boolean);
    if (result.changedInputs.length > 0) {
      result.status = 'stale';
      result.reason = `built from ${gitHash}; engine inputs changed since then (rebuild needed)`;
    } else {
      result.status = 'warn';
      result.reason = `built from ${gitHash}; commits since then changed no engine input`;
    }
    return result;
  }

  if (result.sourceNewer.length > 0) {
    result.status = 'stale-source';
    result.reason = 'built from HEAD, but engine inputs were modified after the build (rebuild needed to match)';
    return result;
  }

  result.status = 'ok';
  result.reason = `built from HEAD (${gitHash})`;
  return result;
}

const EXIT_FAIL = new Set(['stale', 'unknown']);

function findDefaultBinary(repoRoot) {
  const entries = fs.readdirSync(repoRoot).filter((f) => /^vect-or-engine.*\.node$/.test(f));
  return entries.length ? path.join(repoRoot, entries[0]) : null;
}

function main(argv) {
  const repoRoot = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..');
  const asJson = argv.includes('--json');
  const i = argv.indexOf('--binary');
  let binaryPath = i >= 0 ? path.resolve(argv[i + 1]) : findDefaultBinary(repoRoot);

  if (!binaryPath || !fs.existsSync(binaryPath)) {
    console.error('engine-build-check: no .node binary found (run npm run build, or pass --binary <path>)');
    process.exitCode = 1;
    return;
  }

  let buildInfo = null;
  try {
    const req = createRequire(import.meta.url);
    buildInfo = req(binaryPath).buildInfo();
  } catch (err) {
    console.error(`engine-build-check: could not load ${binaryPath}: ${err.message}`);
    process.exitCode = 1;
    return;
  }

  const r = checkEngineBuild({ repoRoot, buildInfo, binaryPath });
  if (asJson) {
    console.log(JSON.stringify(r, null, 2));
  } else {
    console.log(`[engine-build-check] ${r.status.toUpperCase()}: ${r.reason}`);
    console.log(`  binary: ${binaryPath}`);
    console.log(`  binary gitHash=${buildInfo.gitHash} gitDirty=${buildInfo.gitDirty} version=${buildInfo.version}; HEAD=${r.head ?? 'n/a'}`);
    if (r.changedInputs.length) console.log(`  changed engine inputs: ${r.changedInputs.join(', ')}`);
    if (r.sourceNewer.length) console.log(`  inputs newer than the build: ${r.sourceNewer.join(', ')}`);
  }
  process.exitCode = EXIT_FAIL.has(r.status) ? 1 : 0;
}

if (process.argv[1] && path.resolve(process.argv[1]) === fileURLToPath(import.meta.url)) {
  main(process.argv.slice(2));
}
