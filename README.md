# VectOrEngine (`vect-or-engine`)

> **High-Performance Rust-Powered Vector Indexing (HNSW) & Semantic Validation Engine**

[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg?style=flat-square)](LICENSE)
[![npm](https://img.shields.io/badge/npm-%401abcdefggs%2Fvect--or--engine-CB3837?style=flat-square&logo=npm)](https://github.com/1abcdefggs/vect-or-engine/packages)
[![Version: v0.3.3](https://img.shields.io/badge/version-0.3.3-indigo?style=flat-square)](package.json)
[![Rust](https://img.shields.io/badge/Rust-2021_Edition-DEA584?style=flat-square&logo=rust&logoColor=black)](https://www.rust-lang.org/)
[![Node.js](https://img.shields.io/badge/Node.js-%3E%3D18-339933?style=flat-square&logo=nodedotjs&logoColor=white)](https://nodejs.org/)
[![N-API](https://img.shields.io/badge/N--API-Native_Addon-green?style=flat-square&logo=cplusplus&logoColor=white)](https://napi.rs/)
[![Tests](https://img.shields.io/badge/tests-passing-brightgreen?style=flat-square&logo=githubactions&logoColor=white)](#running-tests)
[![Security: Audited (A+)](https://img.shields.io/badge/security-audited%20(A%2B)-success?style=flat-square)](#security--audit)
[![Platform](https://img.shields.io/badge/platform-Windows%20%7C%20macOS%20%7C%20Linux-lightgrey?style=flat-square)](#)

**VectOrEngine** is a lightweight, ultra-high-performance native semantic computation engine crafted in Rust. Engineered specifically as the **core native backend** for desktop editors and local-first AI runtimes, it delivers **sub-millisecond vector similarity search (HNSW + SIMD)**, **real-time schema-driven static linting**, and **zero-copy memory-mapped knowledge management** via direct in-memory Node.js/Electron N-API bindings.

> 💡 **Dedicated Native Engine for VectOrEdit**:  
> **VectOrEngine** is not an editor itself — it is the **pure native engine** purposefully designed to empower **[VectOrEdit](https://github.com/1abcdefggs/VectOrEdit)** (our Electron desktop editor). By binding directly into VectOrEdit's main process via N-API, it provides zero-latency vector associative retrieval, instantaneous warm restarts with SQLite caching, and schema linting while keeping user documents 100% offline and private.

> **Engineered for Speed**: Pure Rust core featuring `f16` half-precision quantization, SIMD auto-vectorization, and Rayon multi-threaded scans.  
> **Schema-Driven Validation**: High-throughput document linting and conflict analysis powered by the Aho-Corasick automaton and regular expressions.  
> **Zero-Cost Interoperability**: Direct C-ABI native bindings via `napi-rs`, completely eliminating IPC serialization and network latency.

- **Repository**: [https://github.com/1abcdefggs/vect-or-engine](https://github.com/1abcdefggs/vect-or-engine)
- **Author / Copyright**: Copyright (c) 2026 [@1abcdefggs](https://github.com/1abcdefggs)

<div align="center">
  <img src="docs/asset/vect-or-edit-ui-v0321.png" alt="VectOrEdit UI Powered by VectOrEngine" width="100%" />
</div>

---

## Key Features

- **Hierarchical Navigable Small World (HNSW)**: $O(\log N)$ approximate nearest neighbor cosine similarity search with SIMD auto-vectorization ($M=16, efConstruction=64, efSearch=32$).
- **SearchQuery Parameter Bundle**: Structured, type-safe query formulation with top-k limits, slot routing, and minimum similarity threshold filtering.
- **Fast Warm-Restarts with SQLite Cache**: `saveKbCache` and `loadKbCache` eliminate JSON re-parsing and re-quantization on subsequent app launches.
- **Schema-Driven Real-Time Linter**: Document validation and clinical/editorial conflict detection powered by `aho-corasick` and regular expressions (`loadProfile`, `validateSync`, `validate`).
- **Memory-Efficient Representation**:
  - `f16` half-precision float vector quantization (50% RAM reduction).
  - `memmap2` zero-copy memory-mapped I/O for instant multi-gigabyte knowledge loading.
- **Native Node.js / Electron Bindings (N-API)**: Direct in-memory bindings via `napi-rs` with non-blocking worker pool execution (`spawn_blocking`).
- **Model Context Protocol (MCP) Server (Experimental)**: HTTP JSON-RPC daemon code (`mcp/server.mjs`) is provided as an experimental reference implementation; formal integration testing has not yet been conducted.
- **Standalone CLI Binary**: Headless `vect_or_engine_cli` for terminal-based and non-Node.js pipeline integrations.

---

## Project Structure

```
vect-or-engine/
├── Cargo.toml          # Rust crate configuration (v0.3.0) & optimization profiles
├── Cargo.lock          # Deterministic Rust dependency lockfile
├── lib.rs              # N-API bindings & Node.js native interface (bridge layer)
├── index.js            # Cross-platform native addon loader
├── index.d.ts          # TypeScript type declarations
├── package.json        # npm package configuration & NAPI metadata (@1abcdefggs/vect-or-engine)
├── LICENSE             # MIT License
├── README.md           # Documentation
├── mcp/
│   └── server.mjs      # Model Context Protocol (MCP) JSON-RPC HTTP server
├── test/
│   ├── test-napi.js    # Node.js N-API integration tests
│   └── test-kb.json    # Minimal test dataset
└── src/
    ├── lib.rs          # Core Rust library crate root
    ├── main.rs         # Headless CLI binary (vect_or_engine_cli)
    ├── hnsw_index.rs   # HNSW graph indexing & search implementation
    ├── knowledge_store.rs # Vector quantization (f16), mmap, and SQLite store
    ├── validator.rs    # Real-time static linter & rule evaluator
    ├── translator.rs   # Term mapping & colloquial pattern matching
    ├── profile.rs      # Schema-driven profile deserializer
    └── engine_error.rs # Typed engine error definitions
```

---

## Getting Started

### Prerequisites

- [Rust](https://www.rust-lang.org/) (2021 edition, `cargo` v1.75+)
- [Node.js](https://nodejs.org/) (v18+)

### Build Native Addon (N-API)

To build the native `.node` binary for Node.js / Electron:

```bash
# Install dependencies
npm install

# Build release native binary via N-API CLI
npm run build
```

*(Note: Running `cargo build --release` compiles the standard Rust rlib/cdylib, while `npm run build` generates the required Node.js `.node` addon binding.)*

### Run Standalone CLI Binary

For non-Node.js pipelines and terminal testing:

```bash
# Run headless Rust CLI
cargo run --bin vect_or_engine_cli
```

### Running Tests

```bash
# Run Rust unit & integration tests
cargo test

# Run Node.js N-API binding integration tests
npm test
```

### Start MCP Server (Port 4000) [Experimental / Unverified]

> ⚠️ **Implementation Note**: The MCP server script (`mcp/server.mjs`) is currently provided as experimental code only. Full end-to-end testing and production validation have not yet been performed.

```bash
npm run start-mcp
```

---

## Installation in Electron / Node.js Apps

To integrate VectOrEngine into your desktop editor (e.g., `vect-or-edit`):

```bash
# Via npm / GitHub Packages
npm install @1abcdefggs/vect-or-engine

# Or link directly as a local workspace package
npm link ../vect-or-engine
```

---

## JavaScript / TypeScript API Usage (Full Flow)

Here is how **VectOrEdit** utilizes VectOrEngine in production:

```typescript
import {
  loadProfile,
  validateSync,
  loadKnowledgeBase,
  saveKbCache,
  loadKbCache,
  buildIndex,
  search,
  kbInfo
} from '@1abcdefggs/vect-or-engine';

// ── 1. Document Linting & Clinical Rule Validation ──
// Load rule schema (e.g. conflict keywords, numeric thresholds)
await loadProfile('./guidelines/medical_rules.json');

// Validate editor text in real time (< 1ms via Aho-Corasick)
const lintResult = validateSync("Target document text from Monaco Editor...");
if (!lintResult.isValid) {
  console.warn("Linter Markers:", lintResult.markers);
}

// ── 2. Fast Warm-Restart or Initial Load ──
const cachePath = './cache/knowledge.sqlite';

try {
  // Fast path: load cached vectors from SQLite (~10x faster)
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

VectOrEngine has undergone a comprehensive static and dynamic security audit, earning an **A+ (Audited & Production-Ready)** rating:
- **Zero Supply-Chain Vulnerabilities**: Zero runtime npm package dependencies. `npm audit` returned 0 vulnerabilities.
- **SQL Injection Immune**: SQLite caching (`knowledge_store.rs`) strictly employs parameterized prepared statements (`rusqlite::params!`).
- **Loopback Isolation**: The MCP daemon (`server.mjs`) binds exclusively to `127.0.0.1`, completely blocking external network access.
- **Rust Memory Safety**: Compile-time lifetime and bounds enforcement eliminates buffer overflows and memory corruption risks.

📄 *Full Audit Report available at: [docs/PROJECT/05_VECT_OR_ENGINE/SECURITY_REVIEW_0914/SECURITY_AUDIT_REPORT_20260914.md](file:///c:/vect/docs/PROJECT/05_VECT_OR_ENGINE/SECURITY_REVIEW_0914/SECURITY_AUDIT_REPORT_20260914.md)*

---

## License

This project is licensed under the [MIT License](LICENSE).  
Copyright (c) 2026 [@1abcdefggs](https://github.com/1abcdefggs).