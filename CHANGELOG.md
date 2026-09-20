# Changelog

All notable changes to **VectOrEngine** (`@1abcdefggs/vect-or-engine`) will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

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
