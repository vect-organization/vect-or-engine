// Copyright (c) 2026 1abcdefggs
// SPDX-License-Identifier: MIT
// https://github.com/1abcdefggs/vect-or-engine

#![forbid(unsafe_code)]

//! Knowledge base storage — Phase 3.
//!
//! Optimisations layered in this module:
//!  1. **fs::read**       — the JSON file is read once and parsed (no unsafe, no mmap).
//!  2. **f16 quantisation** — vectors stored as 16-bit floats (50 % memory).
//!  3. **rayon**          — parallel linear scan for mid-size datasets.
//!  4. **HnswIndex**      — O(log N) approximate search for large datasets.
//!  5. **SQLite cache**   — avoids re-parsing JSON on every restart.

use half::f16;
use rayon::prelude::*;
use serde::{Deserialize, Serialize};
use std::fs;

use crate::limits::{F16_MAX_ABS, MAX_KB_BYTES, MAX_TOP_K};

use crate::engine_error::EngineError;
use crate::hnsw_index::HnswIndex;

// ─── Public data types ────────────────────────────────────────────────────────

/// Full deserialization target — used only during JSON loading.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KnowledgeItem {
    pub id: Option<String>,
    /// Full-precision f32 vector (only present in JSON source).
    pub vector: Vec<f32>,
    #[serde(flatten)]
    pub metadata: serde_json::Map<String, serde_json::Value>,
}

/// Slim runtime representation — metadata only, vector stored separately.
#[derive(Debug, Clone)]
pub struct SlimItem {
    pub id: Option<String>,
    pub metadata: serde_json::Map<String, serde_json::Value>,
}

/// Query parameter bundle for semantic vector searches.
///
/// Encapsulates the query vector, result limits, and optional filters
/// to prevent data clumps across search APIs.
#[derive(Debug, Clone)]
pub struct SearchQuery<'a> {
    /// The dense query embedding vector.
    pub vector: &'a [f32],
    /// Maximum number of matching items to return.
    pub top_k: usize,
    /// Minimum cosine similarity score threshold (0.0 .. 1.0).
    pub min_similarity: Option<f32>,
    /// Optional slot identifier for multi-vector domain routing.
    pub slot_id: Option<String>,
}

impl<'a> SearchQuery<'a> {
    /// Create a new search query with default settings (`top_k = 10`).
    pub fn new(vector: &'a [f32]) -> Self {
        Self {
            vector,
            top_k: 10,
            min_similarity: None,
            slot_id: None,
        }
    }

    /// Set the maximum number of results to return.
    pub fn with_top_k(mut self, top_k: usize) -> Self {
        self.top_k = top_k;
        self
    }

    /// Set a minimum similarity score threshold.
    pub fn with_min_similarity(mut self, min_similarity: f32) -> Self {
        self.min_similarity = Some(min_similarity);
        self
    }

    /// Filter results to a specific slot ID.
    pub fn with_slot_id(mut self, slot_id: impl Into<String>) -> Self {
        self.slot_id = Some(slot_id.into());
        self
    }
}

// ─── KnowledgeStore ───────────────────────────────────────────────────────────

/// The central in-memory knowledge base with pluggable search backends.
pub struct KnowledgeStore {
    /// Metadata for each item (id + arbitrary JSON fields).
    pub items: Vec<SlimItem>,
    /// f16-quantised vectors — parallel to `items`.
    vectors_f16: Vec<Vec<f16>>,
    /// HNSW index (built on demand via `build_index`).
    hnsw: Option<HnswIndex>,
    /// Expected vector dimensionality (set on first load).
    pub dim: usize,
}

impl Default for KnowledgeStore {
    fn default() -> Self {
        KnowledgeStore::new()
    }
}

impl KnowledgeStore {
    pub fn new() -> Self {
        KnowledgeStore {
            items: Vec::new(),
            vectors_f16: Vec::new(),
            hnsw: None,
            dim: 0,
        }
    }

    pub fn len(&self) -> usize {
        self.items.len()
    }

    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }

    /// Whether an HNSW index has been built.
    pub fn has_index(&self) -> bool {
        self.hnsw.is_some()
    }

    // ── Loading ───────────────────────────────────────────────────────────────

    /// Load knowledge base from a JSON file.
    ///
    /// The file is read and parsed once; vectors are quantised to f16 on the fly.
    /// Files larger than `MAX_KB_BYTES`, zero-length vectors and non-finite or
    /// f16-overflowing components are rejected. Returns the number of items loaded.
    pub fn load_from_path(&mut self, path: &str) -> Result<usize, EngineError> {
        let size = fs::metadata(path)?.len();
        if size > MAX_KB_BYTES {
            return Err(EngineError::InvalidInput(format!(
                "knowledge base file is {size} bytes, larger than the {MAX_KB_BYTES} byte limit"
            )));
        }
        let bytes = fs::read(path)?;

        let raw: Vec<KnowledgeItem> = serde_json::from_slice(&bytes)?;
        let count = raw.len();

        if count == 0 {
            return Ok(0);
        }

        let new_dim = raw[0].vector.len();
        if new_dim == 0 {
            return Err(EngineError::InvalidInput("the first item has an empty vector (dimension 0)".into()));
        }
        for (i, item) in raw.iter().enumerate() {
            if item.vector.len() != new_dim {
                return Err(EngineError::DimensionMismatch { expected: new_dim, got: item.vector.len() });
            }
            if let Some(x) = item.vector.iter().find(|x| !x.is_finite() || x.abs() > F16_MAX_ABS) {
                return Err(EngineError::InvalidInput(format!(
                    "item #{i} has a vector component {x} that is not finite or exceeds {F16_MAX_ABS}"
                )));
            }
        }

        // Reset state (only reached when every item passed the checks above).
        self.dim = new_dim;
        self.hnsw = None;
        self.items = Vec::with_capacity(count);
        self.vectors_f16 = Vec::with_capacity(count);

        for item in raw {
            // Dimension guard.
            if item.vector.len() != self.dim {
                return Err(EngineError::DimensionMismatch {
                    expected: self.dim,
                    got: item.vector.len(),
                });
            }
            // Quantise f32 → f16.
            let f16_vec: Vec<f16> = item.vector.iter().map(|&x| f16::from_f32(x)).collect();

            self.vectors_f16.push(f16_vec);
            self.items.push(SlimItem {
                id: item.id,
                metadata: item.metadata,
            });
        }

        Ok(count)
    }

    /// Rejects a query that contains NaN or an infinity (change-log D4).
    pub fn check_finite_query(query: &[f32]) -> Result<(), EngineError> {
        match query.iter().position(|x| !x.is_finite()) {
            Some(i) => Err(EngineError::InvalidInput(format!("query component #{i} is not finite"))),
            None => Ok(()),
        }
    }

    // ── Index building ────────────────────────────────────────────────────────

    /// Build (or rebuild) the HNSW approximate-nearest-neighbour index.
    ///
    /// **CPU-intensive** — call this inside `tokio::task::spawn_blocking`.
    pub fn build_index(&mut self) {
        // Dequantise for HNSW build (needed once; not kept afterwards).
        let vecs_f32: Vec<Vec<f32>> = self
            .vectors_f16
            .iter()
            .map(|v| v.iter().map(|&x| f32::from(x)).collect())
            .collect();

        self.hnsw = Some(HnswIndex::build(&vecs_f32));
    }

    // ── Search ────────────────────────────────────────────────────────────────

    /// Search using a structured `SearchQuery` parameter bundle.
    ///
    /// Handles vector similarity, result capping, and score thresholding.
    pub fn search_query(&self, query: &SearchQuery) -> Vec<SearchResult> {
        // A non-finite query would give NaN scores; callers should reject it with
        // `check_finite_query`, and the store returns nothing rather than a bogus ranking.
        if Self::check_finite_query(query.vector).is_err() {
            return Vec::new();
        }
        let top_k = query.top_k.min(MAX_TOP_K);
        let results = if let Some(hnsw) = &self.hnsw {
            let raw = hnsw.search(query.vector, top_k);
            raw.into_iter()
                .map(|(score, idx)| self.make_result(score, idx))
                .collect::<Vec<_>>()
        } else {
            self.linear_search(query.vector, top_k)
        };

        if let Some(min_score) = query.min_similarity {
            results.into_iter().filter(|r| r.score >= min_score).collect()
        } else {
            results
        }
    }

    /// HNSW O(log N) search — use after calling `build_index`.
    ///
    /// Retained for ergonomic backward compatibility; delegates to `search_query`.
    pub fn search(&self, query: &[f32], top_k: usize) -> Vec<SearchResult> {
        self.search_query(&SearchQuery::new(query).with_top_k(top_k))
    }

    /// Parallel rayon linear scan — O(N) with SIMD dot product.
    ///
    /// Suitable for ≤ ~10 K items or when HNSW is not yet built.
    pub fn linear_search(&self, query: &[f32], top_k: usize) -> Vec<SearchResult> {
        let mut scores: Vec<(f32, usize)> = self
            .vectors_f16
            .par_iter()
            .enumerate()
            .map(|(idx, v)| {
                let score = dot_product_f16(query, v);
                (score, idx)
            })
            .collect();

        // O(N) partial selection — avoids full sort.
        let n = scores.len();
        if n > top_k {
            scores.select_nth_unstable_by(top_k, |a, b| b.0.total_cmp(&a.0));
            scores.truncate(top_k);
        }
        scores.sort_unstable_by(|a, b| b.0.total_cmp(&a.0));

        scores
            .into_iter()
            .map(|(score, idx)| self.make_result(score, idx))
            .collect()
    }

    // ── SQLite persistence ────────────────────────────────────────────────────

    /// Persist the current knowledge base metadata to a SQLite database.
    ///
    /// Subsequent startups can call `load_from_sqlite` for faster startup
    /// (avoids re-parsing the JSON file and re-quantising vectors).
    pub fn save_to_sqlite(&self, db_path: &str) -> Result<(), EngineError> {
        let conn = rusqlite::Connection::open(db_path)
            .map_err(|e| EngineError::Database(e.to_string()))?;

        conn.execute_batch("
            CREATE TABLE IF NOT EXISTS kb_items (
                idx       INTEGER PRIMARY KEY,
                id        TEXT,
                metadata  TEXT NOT NULL,
                vector_f16 BLOB NOT NULL
            );
            DELETE FROM kb_items;
        ")
        .map_err(|e| EngineError::Database(e.to_string()))?;

        let mut stmt = conn.prepare(
            "INSERT INTO kb_items (idx, id, metadata, vector_f16) VALUES (?1,?2,?3,?4)"
        ).map_err(|e| EngineError::Database(e.to_string()))?;

        for (i, (item, f16v)) in self.items.iter().zip(self.vectors_f16.iter()).enumerate() {
            let metadata_json = serde_json::to_string(&item.metadata)?;
            // Store f16 vector as raw bytes (little-endian).
            let blob: Vec<u8> = f16v.iter().flat_map(|x| x.to_le_bytes()).collect();
            stmt.execute(rusqlite::params![i as i64, item.id, metadata_json, blob])
                .map_err(|e| EngineError::Database(e.to_string()))?;
        }

        Ok(())
    }

    /// Load knowledge base from a SQLite cache (fast restart path).
    pub fn load_from_sqlite(&mut self, db_path: &str) -> Result<usize, EngineError> {
        // Read-only and never create: opening a missing path must not leave a new file behind.
        let conn = rusqlite::Connection::open_with_flags(
            db_path,
            rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY | rusqlite::OpenFlags::SQLITE_OPEN_URI,
        )
        .map_err(|e| EngineError::Database(e.to_string()))?;

        let mut stmt = conn.prepare(
            "SELECT id, metadata, vector_f16 FROM kb_items ORDER BY idx"
        ).map_err(|e| EngineError::Database(e.to_string()))?;

        let rows: Vec<(Option<String>, String, Vec<u8>)> = stmt
            .query_map([], |row| {
                Ok((row.get(0)?, row.get(1)?, row.get(2)?))
            })
            .map_err(|e| EngineError::Database(e.to_string()))?
            .filter_map(|r| r.ok())
            .collect();

        self.items.clear();
        self.vectors_f16.clear();
        self.hnsw = None;

        for (id, meta_json, blob) in rows {
            let metadata: serde_json::Map<String, serde_json::Value> =
                serde_json::from_str(&meta_json)?;
            let f16v: Vec<f16> = blob
                .chunks_exact(2)
                .map(|b| f16::from_le_bytes([b[0], b[1]]))
                .collect();

            if self.dim == 0 {
                self.dim = f16v.len();
            }
            self.items.push(SlimItem { id, metadata });
            self.vectors_f16.push(f16v);
        }

        Ok(self.items.len())
    }

    // ── Internal helpers ──────────────────────────────────────────────────────

    fn make_result(&self, score: f32, idx: usize) -> SearchResult {
        let item = &self.items[idx];
        SearchResult {
            idx,
            score,
            id: item.id.clone(),
            metadata: item.metadata.clone(),
        }
    }
}

// ─── Result type ─────────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct SearchResult {
    pub idx: usize,
    pub score: f32,
    pub id: Option<String>,
    pub metadata: serde_json::Map<String, serde_json::Value>,
}

impl SearchResult {
    /// Convert to a `serde_json::Value` for JSON-RPC responses.
    pub fn to_json_value(&self) -> serde_json::Value {
        let mut map = self.metadata.clone();
        if let Some(id) = &self.id {
            map.insert("id".to_string(), serde_json::json!(id));
        }
        map.insert("score".to_string(), serde_json::json!(self.score));
        serde_json::Value::Object(map)
    }
}

// ─── SIMD-friendly f16 dot product ───────────────────────────────────────────

/// Dot product between an f32 query and an f16-stored vector. Safe code only: fixed
/// 8-wide chunks give the compiler bounds-check-free loops it can vectorise.
#[inline(always)]
fn dot_product_f16(query: &[f32], stored: &[f16]) -> f32 {
    let n = query.len().min(stored.len());
    let (query, stored) = (&query[..n], &stored[..n]);
    let mut acc = [0.0f32; 8];
    let mut q_chunks = query.chunks_exact(8);
    let mut s_chunks = stored.chunks_exact(8);
    for (q, s) in (&mut q_chunks).zip(&mut s_chunks) {
        for k in 0..8 {
            acc[k] += q[k] * f32::from(s[k]);
        }
    }
    let mut sum = acc.iter().sum::<f32>();
    for (q, s) in q_chunks.remainder().iter().zip(s_chunks.remainder()) {
        sum += q * f32::from(*s);
    }
    sum
}

// ─── Tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn make_store(n: usize, dim: usize) -> KnowledgeStore {
        let mut store = KnowledgeStore::new();
        store.dim = dim;
        for i in 0..n {
            let v: Vec<f16> = (0..dim)
                .map(|j| f16::from_f32(if j == i % dim { 1.0 } else { 0.0 }))
                .collect();
            store.vectors_f16.push(v);
            store.items.push(SlimItem {
                id: Some(format!("item-{i}")),
                metadata: serde_json::Map::new(),
            });
        }
        store
    }

    fn write_kb(json: &str) -> String {
        let path = std::env::temp_dir().join(format!("vect-ks-{}-{}.json", std::process::id(), json.len()));
        std::fs::write(&path, json).unwrap();
        path.to_string_lossy().into_owned()
    }

    #[test]
    fn d8_zero_dim_rejected() {
        let mut store = KnowledgeStore::new();
        let p = write_kb(r#"[{"id":"a","vector":[]}]"#);
        assert!(store.load_from_path(&p).is_err());
        assert_eq!(store.dim, 0);
        std::fs::remove_file(p).ok();
    }

    #[test]
    fn d4_non_finite_rejected() {
        let mut store = KnowledgeStore::new();
        // 70000 overflows f16 (max 65504) and would become infinity.
        let p = write_kb(r#"[{"id":"a","vector":[70000.0,0.0]}]"#);
        let e = store.load_from_path(&p).unwrap_err().to_string();
        assert!(e.contains("65504"), "{e}");
        std::fs::remove_file(p).ok();
        assert!(KnowledgeStore::check_finite_query(&[1.0, f32::NAN]).is_err());
        assert!(KnowledgeStore::check_finite_query(&[1.0, f32::INFINITY]).is_err());
        assert!(KnowledgeStore::check_finite_query(&[1.0, -2.0]).is_ok());
        let store = make_store(5, 4);
        assert!(store.search(&[f32::NAN, 0.0, 0.0, 0.0], 3).is_empty());
    }

    #[test]
    fn a_rejected_load_keeps_the_previous_kb() {
        let mut store = KnowledgeStore::new();
        let ok = write_kb(r#"[{"id":"a","vector":[1.0,0.0]},{"id":"b","vector":[0.0,1.0]}]"#);
        assert_eq!(store.load_from_path(&ok).unwrap(), 2);
        let bad = write_kb(r#"[{"id":"c","vector":[1.0,0.0]},{"id":"d","vector":[1.0]}]"#);
        assert!(store.load_from_path(&bad).is_err());
        assert_eq!(store.len(), 2, "validation happens before the old KB is dropped");
        std::fs::remove_file(ok).ok();
        std::fs::remove_file(bad).ok();
    }

    #[test]
    fn s4_top_k_is_capped_and_sqlite_open_does_not_create_files() {
        let store = make_store(5, 4);
        assert_eq!(store.search(&[1.0, 0.0, 0.0, 0.0], usize::MAX).len(), 5);
        let missing = std::env::temp_dir().join(format!("vect-missing-{}.db", std::process::id()));
        let mut s = KnowledgeStore::new();
        assert!(s.load_from_sqlite(&missing.to_string_lossy()).is_err());
        assert!(!missing.exists());
    }

    #[test]
    fn safe_dot_matches_a_plain_sum() {
        let q: Vec<f32> = (0..21).map(|i| i as f32 * 0.5).collect();
        let s: Vec<f16> = (0..21).map(|i| f16::from_f32(1.0 + i as f32)).collect();
        let want: f32 = q.iter().zip(&s).map(|(a, b)| a * f32::from(*b)).sum();
        assert!((dot_product_f16(&q, &s) - want).abs() < 1e-3);
    }

    #[test]
    fn linear_search_returns_correct_top1() {
        let store = make_store(50, 64);
        let mut query = vec![0.0f32; 64];
        query[0] = 1.0;
        let results = store.linear_search(&query, 1);
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].idx, 0);
    }

    #[test]
    fn hnsw_search_returns_approximate_top1() {
        let mut store = make_store(200, 64);
        store.build_index();
        let mut query = vec![0.0f32; 64];
        query[5] = 1.0;
        let results = store.search(&query, 3);
        assert!(!results.is_empty());
        // Node 5 should appear in top results (it is an exact unit vector match).
        assert!(results.iter().any(|r| r.idx == 5));
    }

    #[test]
    fn search_query_builder_and_threshold() {
        let store = make_store(50, 64);
        let mut query_vec = vec![0.0f32; 64];
        query_vec[0] = 1.0;

        let query = SearchQuery::new(&query_vec)
            .with_top_k(5)
            .with_min_similarity(0.5);

        let results = store.search_query(&query);
        assert!(!results.is_empty());
        assert!(results.iter().all(|r| r.score >= 0.5));
        assert_eq!(results[0].idx, 0);
    }
}
