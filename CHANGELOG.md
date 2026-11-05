# Changelog

All notable changes to **VectOrEngine** (`@1abcdefggs/vect-or-engine`) will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

---

## [Unreleased]

Planned as 0.4.0: the MCP request gate changes how the server is used.

### Security
- MCP server ran any request without a gate (Copilot code-scanning AI finding on PR #17, "Excessive agency risk"). Added before dispatch: `Host` / `Origin` must be `127.0.0.1` or `localhost` (HTTP 403), an opt-in bearer token via `MCP_AUTH_TOKEN` (HTTP 401), and `load` paths must resolve, links included, under `KB_ALLOWED_ROOT` (default: the repository; checked as written before any file access, then after resolving links). **Behaviour change**: `load` outside the repository now needs `KB_ALLOWED_ROOT`
- The server logs a warning at startup when `MCP_AUTH_TOKEN` is not set; the token stays opt-in (owner decision 2026-11-04)

### Added
- `npm run check:engine` (`scripts/engine-build-check.mjs`): compares `buildInfo().gitHash` of the built `.node` with HEAD and reports ok, warn, stale-source, stale or unknown; exit 1 for stale and unknown. The publish workflow runs it after the build (PR #25)
- Tests: `test/test-engine-build-check.mjs` (7 verdict cases on the git history) and `test/test-match-modes.js`; the `Rust` CI workflow checks out the full history and runs the build-check tests on Node.js 24 (PR #26)

### Changed
- `engines.node` is `>= 24` (was `>= 18`, end of life); README Node.js requirement updated

### Documentation
- README: security section lists the facts checked on 2026-11-05 instead of the 2026-09-14 rating; project structure, test commands, `buildInfo` and `matchMode` usage, GitHub Packages `.npmrc`, and a real-data recall check added

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
