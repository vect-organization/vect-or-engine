# Changelog

All notable changes to **VectOrEngine** (`@1abcdefggs/vect-or-engine`) will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

---

## [Unreleased]

The MCP request gate described below was added and then removed together with the server in 0.3.14 (S9).

### Security
- MCP server ran any request without a gate (Copilot code-scanning AI finding on PR #17, "Excessive agency risk"). Added before dispatch: `Host` / `Origin` must be `127.0.0.1` or `localhost` (HTTP 403), an opt-in bearer token via `MCP_AUTH_TOKEN` (HTTP 401), and `load` paths must resolve, links included, under `KB_ALLOWED_ROOT` (default: the repository; checked as written before any file access, then after resolving links). **Behaviour change**: `load` outside the repository now needs `KB_ALLOWED_ROOT`
- The server logs a warning at startup when `MCP_AUTH_TOKEN` is not set; the token stays opt-in (owner decision 2026-11-04)

### Added
- `npm run check:engine` (`scripts/engine-build-check.mjs`): compares `buildInfo().gitHash` of the built `.node` with HEAD and reports ok, warn, stale-source, stale or unknown; exit 1 for stale and unknown. The publish workflow runs it after the build (PR #25)
- Tests: `test/test-engine-build-check.mjs` (7 verdict cases on the git history) and `test/test-match-modes.js`; the `Rust` CI workflow checks out the full history and runs the build-check tests on Node.js 24 (PR #26)

### Changed
- `engines.node` is `>= 24` (was `>= 18`, end of life); README Node.js requirement updated
- Removed the `Badge metadata` workflow and `.github/badges/`: its push to `main` is refused by the branch rule ("Changes must be made through a pull request", first run after PR #33). The README version badge reads `package.json` directly and the Rust edition badge is static

### Documentation
- README: security section lists the facts checked on 2026-11-05 instead of the 2026-09-14 rating; project structure, test commands, `buildInfo` and `matchMode` usage, GitHub Packages `.npmrc`, and a real-data recall check added

---

## [0.3.14] - 2026-11-06

Phase D of the 0.4.0 work (cache and input). Change-log rows: `docs/changes/v0.4.0.csv` (P7, P8, R4, S3, S6, S9; S1 closes with the fixed Edit release).

### Added
- **P8**: `loadKnowledgeBaseFromString(json)` loads a knowledge base from JSON text held in memory, with the same format, limits and errors as `loadKnowledgeBase`; a failure keeps the previous one. No file is read or written
- **R4**: the SQLite cache has a header (format version, dimension, item count, a hash of the rows and, when given, the model id). `saveKbCache(path, modelId?)` and `loadKbCache(path, modelId?)` take an optional model id; a cache with another model (or, when a model id is requested, with none stored), a missing or mismatching header or edited rows is rejected and the loaded knowledge base is kept

### Changed
- **P7 (breaking cache format)**: the cache is written to a temporary file in one transaction and renamed over the target, so a crash leaves the old cache or none. **Caches written by 0.3.13 and earlier have no header and are rejected; rebuild them from the JSON**
- **S3**: SQLite is the optional cargo feature `cache`, off by default (the npm build `npm run build` turns it on). Without it the cache calls return an error and the default build has no SQLite code. rusqlite 0.31 to 0.41, which bundles SQLite 3.54.1 (was 3.45.0)
- **S6**: every export that takes a path goes through the one path check, and a test keeps it that way
- CI runs the tests a second time with `--features cache`

### Removed
- **S9**: the experimental MCP server (`mcp/server.mjs`, `npm run start-mcp`) and its README sections. It was a private JSON-RPC HTTP API, not MCP (it rejected `initialize` and `tools/list`), unused by VectOrEdit and untested. The design notes stay in `docs/`. Change-log row S9 ships in 0.4.0

---

## [0.3.13] - 2026-11-06

Phase F of the 0.4.0 work (gates). Change-log rows: `docs/changes/v0.4.0.csv` (R3, S5, S8, S9; S9 waits for an owner decision).

### Changed
- **R3**: `build.rs` uses `SOURCE_DATE_EPOCH` for the embedded build time when it is a valid integer, so two builds of the same source give the same binary. `node scripts/check-reproducible-build.mjs` builds twice and compares SHA-256 (on MSVC it adds `/Brepro` for the check only; the normal build is unchanged). Result on Windows: identical
- **S5**: every synchronous N-API export uses `catch_unwind`, so a Rust panic reaches JavaScript as an `Error` instead of unwinding across the C boundary and ending the host process. `test/test-panic-to-js-error.mjs` builds with the `test-hooks` feature (an export that panics on purpose), loads it and checks that the error arrives and the engine still answers. CI runs it
- **S8**: `deny.toml` and a `deny` job (cargo-deny: advisories, licences, bans, sources) on every pull request; three cargo-fuzz targets in `fuzz/` (`load`, `rules`, `search`); a fixed-seed property test that the exact scan equals a naive reference
- **S9**: the leftovers of the experimental MCP server are recorded in `docs/changes/mcp-server-leftovers.md` (CORS value, token off by default, stale binary names, unguarded `OPTIONS`). Nothing was changed; the owner decides whether to keep or remove the server
- New CI job needs `EmbarkStudios/cargo-deny-action@v2`

---

## [0.3.12] - 2026-11-06

Phase E of the 0.4.0 work (linter). Change-log rows: `docs/changes/v0.4.0.csv` (P6, D5, A6). Measurements: `verification/validator/p6_bench.md`.

### Changed
- **P6**: the keyword rules are matched with one Aho-Corasick pass for the literal words and one `RegexSet` pass for the regex words, instead of one scan per trigger word. Which rule fires, and the marker lines, are unchanged. On a 10,000-character document with 1,000 keyword rules, p50 went from 1.36 ms to 0.02 ms (literal, no match), 1.71 ms to 0.03 ms (literal, match), 1.70 ms to 0.07 ms (both, no match) and 0.53 ms to 0.07 ms (regex, no match)
- **D5 (behaviour change)**: conflict items are matched longest-first (`LeftmostLongest`). With both `F20` and `F20.0` as conflict items, a document that says `F20.0` now reports `F20.0`; before, only `F20` was reported because the shorter match ended first
- **A6**: opt-in text normalisation. `Validator::with_options(profile, mode, true)` NFKC-normalises and lower-cases the document and the literal words before matching, so full-width codes (`Ｆ２０`) and case variants hit. Marker lines and offsets still refer to the original text (an offset map is kept). Regex words are used as written. The default (`with_mode`, and everything reachable from Node.js) is unchanged: this option is Rust API only for now
- New dependency: `unicode-normalization` 0.1 (and its `tinyvec`)

---

## [0.3.11] - 2026-11-06

Phase C2 to C4 of the 0.4.0 work (search core). Change-log rows: `docs/changes/v0.4.0.csv` (P2, P5, A1, A2, A4). Measurements: `verification/c2/`, `verification/c3/`.

### Changed
- **P2, A2**: a knowledge base under 10,000 items is searched with the exact scan (recall 1.0) and `buildIndex` builds nothing for it (it returns the count; `hnswReady` stays false). From 10,000 items on, `buildIndex` builds the HNSW index and `search` uses it. The threshold comes from a measurement of the store's parallel scan: about 1 ms up to 10,000 items on 8 cores, while HNSW recall on uniform data falls to 0.48 at 10,000 and building the index takes up to 30 s (`verification/c2/threshold_parallel_2026-11-06.md`). Every dictionary in use today has fewer than 2,000 items, so none of them builds an index any more
- **A1**: vectors are normalised to unit length at load and the query at search time, so scores are cosine similarities. `kbInfo()` returns `maxNormDeviation` (the largest distance of a vector length from 1 before normalising) and `rescaledCount`. A knowledge base loaded from the SQLite cache reports every row as rescaled, because f16 rounding moves the length by about 1e-3
- **A4**: `search(query, topK, ef?)` takes an optional HNSW width; the default is `max(64, 2 * topK)`. The exact scan ignores it
- **P5**: index build and search use an epoch array for visited nodes instead of a `HashSet` per search, no neighbour-list clones, and the whole candidate set of a layer as the entry set of the next. Index build time is 8 to 40 percent shorter at equal recall (`verification/c3/`)
- Rust API: `SearchQuery::ef` and `with_ef`, `KnowledgeStore::norm_report`, `KnowledgeStore::wants_index`, `NormReport`, `hnsw_index::default_ef`

---

## [0.3.10] - 2026-11-06

Phase C1 of the 0.4.0 work (search core). Change-log rows: `docs/changes/v0.4.0.csv` (P3, P4, R1, R5, A3). Measurements: `verification/phase_c1/`.

### Changed
- The knowledge base keeps its vectors as exact f32 in one contiguous matrix (`VectorMatrix`) that the exact scan and the HNSW index share (P3). f16 remains only as the SQLite cache format, so a knowledge base loaded from the cache scores slightly differently from the JSON it was saved from. Scores now equal the f32 dot product (A3); expect changes in the last digits against 0.3.9
- One safe 16-lane dot product (`vect_or_engine::dot`, still exported as `hnsw_index::dot_product_simd`) is used by the exact scan and the index, with a fixed, documented summation order (P4, R5). The `unsafe`-free f16 dot product is gone
- Equal scores are ordered by index ascending in the exact scan and in the index (R1)
- Building the index shares the matrix instead of copying it: `index_job()` no longer copies the vectors
- Rust API: `HnswIndex::build`, `build_with` and `insert` return `Result` (a vector of another length is an error); new `HnswIndex::build_from_matrix`, `VectorMatrix`; the vector is no longer stored per node
- Measured (this PC, shared and noisy; 10,000 x 384 vectors): the store's exact scan p50 went from 12.5-26.9 ms to 2.7-3.7 ms on one thread, and from 3.2-6.2 ms to 0.67-1.06 ms on 8 threads; index build time 16-30% shorter; recall@10 unchanged (the benchmark no longer rounds through f16, which lifts the clustered figure to 1.0)

---

## [0.3.9] - 2026-11-06

Phase B of the 0.4.0 work (load safety). Change-log rows: `docs/changes/v0.4.0.csv` (D1, D2, D9, D10, D11, D12, P1).

### Changed
- `buildIndex` no longer holds the engine lock while it builds (P1): it copies the vectors under a read lock, builds with no lock, and installs the index under a short write lock. `validateSync` and `search` keep answering during a build (3,000 vectors: slowest `validateSync` 2.3 ms during an 8.3 s build). If the knowledge base is loaded again while an index is being built, the stale index is not installed and `buildIndex` fails with an error that says so
- A failed load keeps the previous knowledge base (D1), and a cache load (`loadKbCache`) is parsed into locals first (D2): the dimension is reset from the cache, a vector of odd byte length, a dimension change between rows, a non-finite component or bad metadata fails the whole load instead of skipping the row, and an empty cache is an error
- **Behaviour change (D12)**: loading an empty knowledge-base file is an error (`the knowledge base file contains no items`); it used to return 0 and leave the previous knowledge base in place
- CLI: `search` rejects a query whose length differs from the knowledge base and any non-numeric entry (D9); requests are handled in the order sent, and every reply is written before the process exits at EOF (D10)
- Rust API (D11): `KnowledgeStore::items` and `KnowledgeStore::dim` are private; use `items()` and `dim()`. New `index_job()`, `install_index()`, `IndexJob` and `BuiltIndex` for building off-lock

---

## [0.3.8] - 2026-11-06

Phase A of the 0.4.0 work (guard rails). Change-log rows: `docs/changes/v0.4.0.csv`.

### Changed
- `Validator::new` now returns `Result` (an invalid `value_regex_pattern` is an error, not a silently skipped check). Rust API only
- A profile is checked at load: empty patterns, patterns over 4096 bytes, more than 10,000 rules or 100,000 patterns, and regexes over the 1 MiB size limit are rejected, and the error names the rule
- Guideline `severity` is case-insensitive; an unknown value is an error
- A knowledge base with a zero-length vector, a non-finite component or a component beyond f16 range (65504) is rejected before the previous KB is touched; a rejected load keeps the previous KB
- A query with NaN or an infinity is rejected (`search` in N-API and the CLI); `top_k` is capped at 1000; score ordering uses `total_cmp`
- The SQLite cache is opened read-only for loading, so a wrong path no longer leaves a new empty file

### Security
- All `unsafe` code is gone: the dot products are safe 8-lane loops and the knowledge base is read with `fs::read` instead of `mmap`. The six core modules carry `#![forbid(unsafe_code)]`
- Removed the unused dependencies `anyhow` and `memmap2`, and the unused `serde` feature of `half`
- Removed `once_cell` (replaced by `std::sync::LazyLock`); `tokio` is now optional and a new default feature `cli` enables the CLI's extra tokio features, so `--no-default-features` builds the core without tokio. tokio is 1.54.1 or later
- A profile file over 16 MiB is rejected before it is read (CLI and N-API); a knowledge-base file over 1 GiB is rejected
- README and comments no longer describe `memmap2`, the 4-way unrolled loop or `unsafe` blocks that were removed; `tests/repo_hygiene.rs` guards this and the dependency list

---

## [0.3.7] - 2026-11-04

Not tagged or released. `main` carries this version number together with the Unreleased items above; the next release is planned as 0.4.0.

### Changed
- **HNSW default** (PR #21): neighbour selection uses the diversity heuristic and fills the remaining slots with pruned candidates; layer 0 keeps up to M0 = 2 x M = 32 links. M = 16, efConstruction = 200 and efSearch = 64 are unchanged. Search results change for every caller of the default. Recall@10 on synthetic 384-dimension data, 1,000 queries: uniform 1,000 vectors 0.8374 to 0.9751, clustered 1,000 0.6227 to 0.9998, uniform 10,000 0.2629 to 0.4786, clustered 10,000 0.6558 to 0.9990 (`verification/recall`)
- `HnswParams` exposes the knobs for experiments (`build_with`, `search_with`); `examples/bench.rs` takes `--dim`, `--m`, `--m0`, `--ef-construction`, `--ef-search`, `--no-heuristic`, `--no-keep-pruned`

### Added
- `buildInfo()` returns `{ version, gitHash, gitDirty, builtAtUnix }` so a `.node` file can be traced to the build that produced it. `build.rs` embeds the short git hash (`unknown` without git), whether the working tree had uncommitted changes (`"true"` / `"false"` / `"unknown"`), and the build time in UNIX seconds. The build reruns when sources, `Cargo.toml` or git HEAD change
- `verification/recall/`: procedure, reports, `reproduce.ps1` and `make_recall_report.ps1` for the recall numbers (PR #21, #24)
- `.github/workflows/badge-meta.yml`: writes the version and Rust badge JSON files when `Cargo.toml` changes on `main` (PR #23; not exercised yet)

### Documentation
- README: search parameters and speed claims corrected, recall section added (PR #22); badges placed in the sections they describe, CodeQL badge (PR #23); recall chart and method flow (PR #24)

---

## [0.3.6] - 2026-11-03

### Fixed
- `cargo build` failed on Linux: the CLI binary linked the N-API bindings, whose `napi_*` symbols only exist inside Node.js. The bindings moved to `src/napi_bindings.rs` behind a new `napi` Cargo feature; `npm run build` / `build:debug` pass `--features napi` (Cargo.toml, lib.rs, package.json)
- `package-lock.json` still said 0.3.0; now matches the package version

### Security
- MCP server (`mcp/server.mjs`) returned `String(err?.message ?? err)` for every failure, which could expose internal error details (CodeQL `js/stack-trace-exposure`, alert #1). Errors are now classified per JSON-RPC 2.0: parse error -32700, invalid request -32600 (non-object, `null`, batch array), unknown method -32601, bad params and engine `InvalidArg` errors -32602 with their message; anything else returns -32603 `Internal error` and the details go to the server log only

### Added
- `Rust` CI workflow: `cargo build` and `cargo test` on ubuntu-latest (`.github/workflows/rust.yml`)

### Documentation
- README: project structure shows the core / `napi` split and which build targets include N-API
- README: VectOrEdit link fixed (was a 404), demo replaced with the VectOrEdit v0.3.56 GIF, and the integration text lists the calls VectOrEdit actually makes
- README: badges match the facts (live Rust CI badge; `.node` is built for Windows x64 only; security badge points to the notes); the security section adds the 5 `unsafe` blocks and the unauthenticated MCP server that the 2026-09-14 audit did not cover
- README: CLI section documents the stdin JSON-RPC format, Linux support, out-of-order responses and the exit-on-EOF behaviour
- README: MCP section warns that the server has no request gate yet (no authentication, no limit on `load` paths) and lists its JSON-RPC error codes; the gate is developed on the `feat/mcp-request-gate` branch, not in this release

## [0.3.5] - 2026-10-01

### Fixed
- `load_profile` replaced the whole `Engine`, silently discarding the loaded KB and HNSW index; now swaps profile / validator / translator only (lib.rs, main.rs CLI)
- `search` accepted queries of a different dimension and `dot_product_simd` truncated to `min(len)` (meaningless scores); now returns `DimensionMismatch`

### Added
- `Profile::load_any_json_str` + `from_guideline_json_str`: vect-or-generate `guideline_*.json` adapted into Profile (previously deserialized to an all-`None` Profile without error); `Profile::is_empty`; unit tests
- `load_profile(path, match_mode?)`: keyword-rule matching mode `literal` (default, unchanged behaviour), `regex`, or `both` (literal OR regex; a trigger word that is not a valid regex matches as literal text under `both`). Invalid regexes are rejected at load time and keep the previously loaded rules (ADR-0006)
- Unit tests for the translator (colloquial matching and per-rule item mapping), the match modes, and the 0.3.5 regressions (KB kept on `load_profile`, dimension check)

### Changed
- `Profile::is_empty` destructures `DocumentTemplate`, so a new field there fails to compile until the check is updated

---

## [0.3.4] - 2026-09-20

### Added
- **Automated GitHub Release Binary Upload**: Updated `.github/workflows/publish-package.yml` with `contents: write` permissions and `gh release upload` step to automatically attach compiled native binary addons (`*.node`) to GitHub Releases upon publication.

### Changed
- **Node.js 24 Environment Maintenance**: Explicitly set and maintained `node-version: 24` in release workflows for modern runner compatibility.

---

## [0.3.3] - 2026-09-20

### Changed
- **Version-Independent Native Binary Name**: Updated `napi.name` to `"vect-or-engine"` so that the generated binary is consistently named `vect-or-engine.win32-x64-msvc.node` across releases, preventing stale versioned binaries from accumulating in `npm pack`.
  - Windows x64 Binary SHA-256: `6fb8ce8a86a908250d9239afbe99d8ffc22a3f3287478b3e0f091cdae9de6c37`

---

## [0.3.2] - 2026-09-14

### Fixed
- **Compiler Warning Elimination**: Added `#[allow(dead_code)]` to the internal `Engine` struct in `lib.rs` to eliminate unused struct field warnings for clean release builds.

---

## [0.3.1] - 2026-09-14

### Added
- **Security Audit & Badge Integration**: Formally incorporated the 2026-09-14 Security Audit Report (`docs/PROJECT/05_VECT_OR_ENGINE/SECURITY_REVIEW_0914/SECURITY_AUDIT_REPORT_20260914.md`), earning an **A+ (Audited & Production-Ready)** rating. Added security status badge to `README.md`.
- **Comprehensive API Documentation**: Expanded `README.md` with end-to-end TypeScript usage examples showcasing real-time schema linting (`loadProfile`, `validateSync`), high-speed SQLite warm-restarts (`loadKbCache`, `saveKbCache`), and HNSW SIMD cosine search (`search`).

### Changed
- **Positioning Clarification**: Updated project documentation to emphasize that VectOrEngine is the dedicated native backend engine powering **VectOrEdit** (Electron desktop editor), providing zero-latency vector retrieval and 100% offline schema validation.
- **MCP Daemon Status**: Clarified that the Model Context Protocol daemon (`mcp/server.mjs`) is currently an experimental reference implementation with code provided, but without formal production verification.
- **Repository Maintenance**: Consolidated and archived historical draft reviews into `docs/_ARCHIVE_REVIEWS/` and updated `.gitignore` to keep active workspace clean.
- **Compiler Warning Suppression**: Added `#[allow(dead_code)]` to the global `Engine` struct in `lib.rs` for clean release builds without unused field warnings.

---

## [0.3.0] - 2026-08-25

### Added
- **HNSW Vector Search Engine**: Hierarchical Navigable Small World approximate nearest neighbor search with SIMD auto-vectorization in Rust.
- **f16 Quantization**: Half-precision float vector quantization for 50% memory footprint reduction.
- **SQLite Knowledge Cache**: Fast-load binary/vector cache store via `rusqlite` with parameterized prepared statements.
- **Aho-Corasick & Regex Linter**: High-throughput rule and constraint validation engine with sub-millisecond execution.
- **N-API Native Addon**: Zero-copy native bindings for Node.js and Electron runtimes via `napi-rs`.
- **Standalone CLI**: Headless binary `vect_or_engine_cli` for non-Node environments and direct terminal usage.
