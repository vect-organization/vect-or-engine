# VectOrEngine (`vect-or-engine`)

> **High-Performance Rust-Powered Vector Indexing (HNSW) & Semantic Validation Engine**

[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg?style=flat-square)](LICENSE)
[![Version](https://img.shields.io/endpoint?url=https%3A%2F%2Fraw.githubusercontent.com%2F1abcdefggs%2Fvect-or-engine%2Fmain%2F.github%2Fbadges%2Fversion.json&style=flat-square)](package.json)

**VectOrEngine** is a lightweight, ultra-high-performance native semantic computation engine crafted in Rust. Engineered specifically as the **core native backend** for desktop editors and local-first AI runtimes, it delivers **HNSW vector similarity search (SIMD)** with measured recall ([see Search quality](#search-quality-recall)), **real-time schema-driven static linting**, and **zero-copy memory-mapped knowledge management** via direct in-memory Node.js/Electron N-API bindings.

> 💡 **Dedicated Native Engine for VectOrEdit**:  
> **VectOrEngine** is not an editor itself — it is the **pure native engine** purposefully designed to empower **[VectOrEdit](https://github.com/1abcdefggs/vect-or-edit)** (our Electron desktop editor). Loaded into VectOrEdit's main process via N-API, it runs the real-time guideline linter (`loadProfile`, `validateSync`) and the knowledge-base vector search behind <kbd>Alt</kbd>+<kbd>S</kbd> (`loadKnowledgeBase`, `buildIndex`, `search`), keeping user documents 100% offline and private.

> **Engineered for Speed**: Pure Rust core featuring `f16` half-precision quantization, SIMD auto-vectorization, and Rayon multi-threaded scans.  
> **Schema-Driven Validation**: High-throughput document linting and conflict analysis powered by the Aho-Corasick automaton and regular expressions.  
> **Zero-Cost Interoperability**: Direct C-ABI native bindings via `napi-rs`, completely eliminating IPC serialization and network latency.

- **Repository**: [https://github.com/1abcdefggs/vect-or-engine](https://github.com/1abcdefggs/vect-or-engine)
- **Author / Copyright**: Copyright (c) 2026 [@1abcdefggs](https://github.com/1abcdefggs)

<div align="center">
  <img src="https://raw.githubusercontent.com/1abcdefggs/vect-or-edit/main/docs/assets/repository-ui-v0356.gif" alt="VectOrEdit v0.3.56 demo, powered by VectOrEngine" width="720" />
  <br />
  <sub>VectOrEdit v0.3.56 · more in the <a href="https://github.com/1abcdefggs/vect-or-edit">VectOrEdit repository</a></sub>
</div>

---

## Key Features

- **Hierarchical Navigable Small World (HNSW)**: $O(\log N)$ approximate nearest neighbor cosine similarity search with SIMD auto-vectorization (heuristic neighbour selection; $M=16$, $M_0=32$, $efConstruction=200$, $efSearch=64$).
- **SearchQuery Parameter Bundle**: Structured, type-safe query formulation with top-k limits, slot routing, and minimum similarity threshold filtering.
- **Fast Warm-Restarts with SQLite Cache**: `saveKbCache` and `loadKbCache` eliminate JSON re-parsing and re-quantization on subsequent app launches.
- **Schema-Driven Real-Time Linter**: Document validation and clinical/editorial conflict detection powered by `aho-corasick` and regular expressions (`loadProfile`, `validateSync`, `validate`).
- **Memory-Efficient Representation**:
  - `f16` half-precision float vector quantization (50% RAM reduction).
  - `memmap2` memory-mapped I/O, so a knowledge file is read without copying it into a buffer first.
- **Native Node.js / Electron Bindings (N-API)**: Direct in-memory bindings via `napi-rs` with non-blocking worker pool execution (`spawn_blocking`).
- **Model Context Protocol (MCP) Server (Experimental)**: HTTP JSON-RPC daemon code (`mcp/server.mjs`) is provided as an experimental reference implementation; formal integration testing has not yet been conducted.
- **Standalone CLI Binary**: Headless `vect_or_engine_cli` for terminal-based and non-Node.js pipeline integrations.

---

## Search quality (recall)

Recall@10 is the fraction of the true top 10 nearest vectors that the index returns. Higher is better.
The values below come from synthetic data (random or clustered vectors, 384 dimensions, 1,000 queries).
They are not measured on real sentence embeddings.

| Data | Previous default | Current default |
| :--- | ---: | ---: |
| uniform, 1,000 vectors | 0.8374 | **0.9751** |
| clustered, 1,000 vectors | 0.6227 | **0.9998** |
| uniform, 10,000 vectors | 0.2629 | **0.4786** |
| clustered, 10,000 vectors | 0.6558 | **0.9990** |

The current default uses heuristic neighbour selection, with M = 16 and M0 = 32 links on layer 0.
Uniform vectors point in random directions, so they are the hardest case for graph search. Recall stays
at about 0.48 at 10,000 vectors. Dimension 768 was also measured: uniform 0.385, clustered 0.999.

A check on the project's own sentence embeddings (multilingual-e5-small, 384 dimensions; 7 dictionaries, 3,840 vectors in one index; every stored vector as a query) gave recall@10 0.998 with the current default (2026-11-05). This data behaves like the clustered case, not the uniform one. That script is not in this repository yet.

Query speed is not stated here, because it varies between runs on the same machine.
The full procedure, reports and reproduction script are in [verification/recall](verification/recall/README.md).

## Project Structure

```
vect-or-engine/
├── Cargo.toml          # Rust crate configuration, `napi` feature & optimization profiles
├── Cargo.lock          # Deterministic Rust dependency lockfile
├── build.rs            # Embeds git hash, dirty flag and build time for buildInfo()
├── lib.rs              # Crate root: core engine modules & public re-exports (no N-API)
├── index.js            # Cross-platform native addon loader
├── index.d.ts          # TypeScript type declarations
├── package.json        # npm package configuration & NAPI metadata (@1abcdefggs/vect-or-engine)
├── LICENSE             # MIT License
├── README.md           # Documentation
├── mcp/
│   └── server.mjs      # Model Context Protocol (MCP) JSON-RPC HTTP server
├── scripts/
│   └── engine-build-check.mjs # Checks that a built .node matches HEAD (npm run check:engine)
├── examples/
│   └── bench.rs        # HNSW recall and latency benchmark
├── verification/
│   └── recall/         # Recall procedure, reports and reproduction scripts
├── test/
│   ├── test-napi.js    # Node.js N-API integration tests
│   ├── test-match-modes.js # Match modes literal / regex / both (needs a built .node)
│   ├── test-engine-build-check.mjs # Build-check verdicts on the git history
│   └── test-kb.json    # Minimal test dataset
└── src/
    ├── napi_bindings.rs # N-API bindings & Node.js native interface (only with --features napi)
    ├── main.rs         # Headless CLI binary (vect_or_engine_cli), core engine only
    ├── hnsw_index.rs   # HNSW graph indexing & search implementation
    ├── knowledge_store.rs # Vector quantization (f16), mmap, and SQLite store
    ├── validator.rs    # Real-time static linter & rule evaluator
    ├── translator.rs   # Term mapping & colloquial pattern matching
    ├── profile.rs      # Schema-driven profile deserializer
    └── engine_error.rs # Typed engine error definitions
```

The core engine (`src/*.rs` except `napi_bindings.rs`) has no N-API dependency. The `.node` addon and the CLI both build on it:

| Target | Command | Includes N-API |
| :--- | :--- | :--- |
| Node.js addon (`.node`) | `npm run build` (passes `--features napi`) | Yes |
| CLI (`vect_or_engine_cli`) and plain `cargo build` | `cargo build` | No |

The `napi_*` symbols exist only inside a running Node.js process, so keeping them out of the default build lets the CLI link on Linux (checked by the `Rust` CI workflow on ubuntu-latest).

---

## Getting Started

### Prerequisites

[![Rust](https://img.shields.io/endpoint?url=https%3A%2F%2Fraw.githubusercontent.com%2F1abcdefggs%2Fvect-or-engine%2Fmain%2F.github%2Fbadges%2Frust.json&style=flat-square&logo=rust&logoColor=black)](https://www.rust-lang.org/) [![Node.js](https://img.shields.io/badge/Node.js-%3E%3D24-339933?style=flat-square&logo=nodedotjs&logoColor=white)](https://nodejs.org/) [![Platform: .node Windows x64, core and CLI Windows and Linux](https://img.shields.io/badge/platform-.node%3A%20Windows%20x64%20%7C%20core%2FCLI%3A%20Windows%2C%20Linux-lightgrey?style=flat-square)](#getting-started)

- [Rust](https://www.rust-lang.org/) (2021 edition, `cargo` v1.75+)
- [Node.js](https://nodejs.org/) (v24+)

### Build Native Addon (N-API)

[![N-API](https://img.shields.io/badge/N--API-Native_Addon-green?style=flat-square&logo=cplusplus&logoColor=white)](https://napi.rs/)

To build the native `.node` binary for Node.js / Electron:

```bash
# Install dependencies
npm install

# Build release native binary via N-API CLI
npm run build
```

*(Note: The N-API bindings sit behind the `napi` Cargo feature. `npm run build` passes `--features napi` and generates the Node.js `.node` addon. A plain `cargo build` compiles the core engine and the CLI without N-API, so it links outside Node.js (checked on Windows and on Linux CI).)*

### Run Standalone CLI Binary

For non-Node.js pipelines and terminal testing:

```bash
# Run headless Rust CLI
cargo run --bin vect_or_engine_cli
```

The CLI reads one JSON-RPC request per line on stdin (`{"method":"ping","id":"1"}`) and writes one response per line on stdout. It runs on Windows and Linux (Linux checked under WSL Ubuntu on 2026-11-03). File paths must be relative; absolute paths and `..` are rejected.

- Requests run concurrently, so responses can arrive out of order; match them by `id`
- The process exits as soon as stdin closes, and responses still in progress are dropped; keep stdin open until every response has arrived

### Running Tests

```bash
# Run Rust unit & integration tests (core engine)
cargo test

# Also run the N-API binding unit tests (checked on Windows)
cargo test --features napi

# Run Node.js N-API binding integration tests
npm test

# Match modes (literal / regex / both) against the built addon
node test/test-match-modes.js

# Build-check verdicts (needs the full git history)
node --test test/test-engine-build-check.mjs

# Check that the built .node matches HEAD (exit 1 when stale or unknown)
npm run check:engine
```

`buildInfo()` returns `{ version, gitHash, gitDirty, builtAtUnix }`, fixed at build time. The version number alone cannot tell two builds apart; compare `gitHash` with HEAD (`npm run check:engine` does this).

### Start MCP Server (Port 4000) [Experimental / Unverified]

> ⚠️ **Implementation Note**: The MCP server script (`mcp/server.mjs`) is currently provided as experimental code only. Full end-to-end testing and production validation have not yet been performed.

```bash
npm run start-mcp
```

Every request passes a gate before it runs (on `main` since PR #25; not yet in a published release, planned for 0.4.0):

| Check | Default | Setting |
| :--- | :--- | :--- |
| `Host` and `Origin` headers must be `127.0.0.1` or `localhost` (blocks DNS rebinding and cross-site requests) | On; HTTP 403 otherwise | None |
| `Authorization: Bearer <token>` | Off; the server logs a warning at startup | Set `MCP_AUTH_TOKEN`; HTTP 401 without the right token |
| `load` paths must resolve (links included) under an allowed folder | On; the folder is this repository | Set `KB_ALLOWED_ROOT`; the server stops at startup if it does not exist |

Errors follow JSON-RPC 2.0: -32700 parse error, -32600 invalid request, -32601 unknown method, -32602 invalid params (with the reason), -32603 internal error (details only in the server log).

---

## Installation in Electron / Node.js Apps

[![npm](https://img.shields.io/badge/npm-%401abcdefggs%2Fvect--or--engine-CB3837?style=flat-square&logo=npm)](https://github.com/1abcdefggs/vect-or-engine/packages)

To integrate VectOrEngine into your desktop editor (e.g., `vect-or-edit`):

The package is published to GitHub Packages (latest published: 0.3.6). npm needs the scope's registry and a token with `read:packages`, for example in `.npmrc`:

```ini
@1abcdefggs:registry=https://npm.pkg.github.com
//npm.pkg.github.com/:_authToken=${NODE_AUTH_TOKEN}
```

```bash
# Via npm / GitHub Packages
npm install @1abcdefggs/vect-or-engine

# Or link directly as a local workspace package
npm link ../vect-or-engine
```

VectOrEdit itself depends on a sibling checkout (`"@1abcdefggs/vect-or-engine": "file:../vect-or-engine"`) and ships the built `.node` with its installer.

---

## JavaScript / TypeScript API Usage (Full Flow)

A full example flow. **VectOrEdit** calls `loadProfile`, `validateSync`, `loadKnowledgeBase`, `buildIndex` and `search`; the SQLite cache calls (`loadKbCache`, `saveKbCache`) and `kbInfo` are optional:

```typescript
import {
  buildInfo,
  loadProfile,
  validateSync,
  loadKnowledgeBase,
  saveKbCache,
  loadKbCache,
  buildIndex,
  search,
  kbInfo
} from '@1abcdefggs/vect-or-engine';

// ── 0. Which build is loaded ──
console.log(buildInfo()); // { version, gitHash, gitDirty, builtAtUnix }

// ── 1. Document Linting & Clinical Rule Validation ──
// Load rule schema (e.g. conflict keywords, numeric thresholds)
// matchMode: 'literal' (default), 'regex' or 'both'. Call again with the same profile to change it; the KB is kept
await loadProfile('./guidelines/medical_rules.json', 'literal');

// Validate editor text (Aho-Corasick for literal rules)
const lintResult = validateSync("Target document text from Monaco Editor...");
if (!lintResult.isValid) {
  console.warn("Linter Markers:", lintResult.markers);
}

// ── 2. Fast Warm-Restart or Initial Load ──
const cachePath = './cache/knowledge.sqlite';

try {
  // Fast path: load cached vectors from SQLite (faster than re-parsing JSON; speed-up not measured)
  const count = await loadKbCache(cachePath);
  console.log(`Loaded ${count} vectors from cache.`);
} catch {
  // Cold path: parse JSON, quantize to f16, and persist cache
  await loadKnowledgeBase('./knowledge_base.json');
  await saveKbCache(cachePath);
}

// ── 3. Build HNSW Index & Associative Search ──
await buildIndex();

// Query 384-dimensional dense vector
const queryVector = new Float32Array([0.12, 0.45, -0.33 /* ... 384 dims */]);
const results = await search(queryVector, 5); // Retrieve top-5 nearest neighbors

console.log("Vector Recommendations:", results);
```

---

## Security & Audit

[![Rust CI](https://github.com/1abcdefggs/vect-or-engine/actions/workflows/rust.yml/badge.svg)](https://github.com/1abcdefggs/vect-or-engine/actions/workflows/rust.yml)
[![CodeQL](https://github.com/1abcdefggs/vect-or-engine/actions/workflows/dynamic/github-code-scanning/codeql/badge.svg)](https://github.com/1abcdefggs/vect-or-engine/security/code-scanning)
[![Security: reviewed 2026-11-05](https://img.shields.io/badge/security-reviewed%202026--11--05-green?style=flat-square)](#security--audit)

Checked on `main` on 2026-11-05:
- **Dependencies**: no runtime npm dependencies; `npm audit` reports 0 vulnerabilities. GitHub Dependabot, code scanning (CodeQL) and secret scanning have 0 open alerts
- **SQL**: the SQLite cache (`knowledge_store.rs`) uses prepared statements with bound parameters (`rusqlite::params!`); no SQL is built from strings
- **`unsafe` code**: 5 blocks. 2 in `hnsw_index.rs` (`get_unchecked` in the 4-way unrolled dot product and its tail) and 3 in `knowledge_store.rs` (one mmap, two `get_unchecked` loops). Memory safety outside these blocks is checked at compile time
- **MCP server** (experimental, runs only when started with `npm run start-mcp`): listens on `127.0.0.1` only. Every request must have a `Host` / `Origin` of `127.0.0.1` or `localhost` (against DNS rebinding and cross-site requests), and `load` is limited to `KB_ALLOWED_ROOT`. The bearer token (`MCP_AUTH_TOKEN`) is opt-in, and the server warns at startup when it is not set. This gate is on `main` but not yet in a published release (planned for 0.4.0); 0.3.6 and earlier have no gate
- **Error details**: since 0.3.6 the MCP server does not return raw error details to the client (CodeQL alert #1); internal errors answer `Internal error` and are logged on the server

An earlier audit (2026-09-14) is kept as an internal record and is not published here. It did not cover the `unsafe` blocks or the missing MCP gate, so its rating is not quoted.

---

## License

This project is licensed under the [MIT License](LICENSE).  
Copyright (c) 2026 [@1abcdefggs](https://github.com/1abcdefggs).