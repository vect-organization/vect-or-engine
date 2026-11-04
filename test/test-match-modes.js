// Checks load_profile(path, matchMode?) from Node against the built native addon (ADR-0006).
// Run after `npm run build`:  node test/test-match-modes.js
// Note: the regex rule is built with String.fromCharCode(92) because a "\d" typed inside a JS string
// literal silently loses its backslash (the cause of a false "both misses G42" report on 2026-10-18).
const fs = require('fs');
const os = require('os');
const path = require('path');
const engine = require(path.join(__dirname, '..', 'index.js'));

const BACKSLASH = String.fromCharCode(92);
const regexRule = 'G' + BACKSLASH + 'd+';
const dir = fs.mkdtempSync(path.join(os.tmpdir(), 'engine-match-modes-'));
const profilePath = path.join(dir, 'profile.json');
fs.writeFileSync(
  profilePath,
  JSON.stringify({
    assessment_rules: {
      keyword_rules: [
        { trigger_words: ['C++'], warning_message: 'literal word with regex symbols' },
        { trigger_words: [regexRule], warning_message: 'regex form' },
      ],
    },
  })
);

const hit = (text) => engine.validateSync(text).isValid === false;
const failures = [];
const expect = (label, actual, expected) => {
  const ok = actual === expected;
  console.log(`${ok ? 'ok  ' : 'FAIL'} ${label}: ${actual}`);
  if (!ok) failures.push(label);
};

(async () => {
  // Default (no argument) must equal 'literal'.
  await engine.loadProfile(profilePath);
  expect('default  "use C++ now"', hit('use C++ now'), true);
  expect('default  "see G42" (regex rule)', hit('see G42'), false);

  await engine.loadProfile(profilePath, 'literal');
  expect('literal  "use C++ now"', hit('use C++ now'), true);
  expect('literal  "see G42"', hit('see G42'), false);
  expect('literal  rule text itself', hit('see ' + regexRule), true);

  // 'C++' is a valid regular expression in Rust (C+ followed by a repeat), so regex mode accepts it.
  await engine.loadProfile(profilePath, 'regex');
  expect('regex    "see G42"', hit('see G42'), true);
  expect('regex    "use C++ now"', hit('use C++ now'), true);

  // A word that is not a valid regular expression is rejected at load under regex mode,
  // and the rules loaded before stay in force.
  const badPath = path.join(dir, 'bad.json');
  fs.writeFileSync(badPath, JSON.stringify({ assessment_rules: { keyword_rules: [{ trigger_words: ['('], warning_message: 'bad regex' }] } }));
  await engine.loadProfile(profilePath, 'both');
  let rejected = false;
  try {
    await engine.loadProfile(badPath, 'regex');
  } catch (e) {
    rejected = true;
  }
  expect('regex    invalid pattern "(" rejected at load', rejected, true);
  expect('regex    previous rules kept: "see G42"', hit('see G42'), true);
  await engine.loadProfile(badPath, 'both');
  expect('both     "(" accepted as literal text', hit('f(x'), true);
  await engine.loadProfile(profilePath, 'both');

  expect('both     "use C++ now"', hit('use C++ now'), true);
  expect('both     "see G42"', hit('see G42'), true);

  let bogus = false;
  try {
    await engine.loadProfile(profilePath, 'bogus');
  } catch (e) {
    bogus = true;
  }
  expect('unknown mode rejected', bogus, true);
})()
  .catch((e) => {
    console.error('ERROR', e);
    failures.push('exception');
  })
  .finally(() => {
    fs.rmSync(dir, { recursive: true, force: true });
    if (failures.length) {
      console.error(`\n${failures.length} check(s) failed: ${failures.join(', ')}`);
      process.exit(1);
    }
    console.log('\nall match-mode checks passed');
  });
