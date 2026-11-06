// Copyright (c) 2026 1abcdefggs
// SPDX-License-Identifier: MIT
// https://github.com/1abcdefggs/vect-or-engine

#![forbid(unsafe_code)]

//! Knowledge base storage — Phase 3.
//!
//! Optimisations layered in this module:
//!  1. **fs::read**       — the JSON file is read once and parsed (no unsafe, no mmap).
//!  2. **Flat f32 matrix** — one contiguous `VectorMatrix` shared by the exact scan and the
//!     HNSW index (P3); f16 is only the SQLite cache format.
//!  3. **rayon**          — parallel linear scan for mid-size datasets.
//!  4. **HnswIndex**      — O(log N) approximate search for large datasets.
//!  5. **SQLite cache**   — avoids re-parsing JSON on every restart.

use half::f16;
use rayon::prelude::*;
use serde::{Deserialize, Serialize};
use std::fs;
use std::sync::Arc;

use crate::limits::{F16_MAX_ABS, MAX_KB_BYTES, MAX_TOP_K};
use crate::matrix::{dot, VectorMatrix};

use crate::engine_error::EngineError;
use crate::hnsw_index::{HnswIndex, HnswParams};

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
    /// Query-time HNSW width (A4). `None` means `max(64, 2 * top_k)`. Ignored by the exact scan.
    pub ef: Option<usize>,
}

impl<'a> SearchQuery<'a> {
    /// Create a new search query with default settings (`top_k = 10`).
    pub fn new(vector: &'a [f32]) -> Self {
        Self {
            vector,
            top_k: 10,
            min_similarity: None,
            slot_id: None,
            ef: None,
        }
    }

    /// Set the HNSW query-time width (a larger value raises recall and costs time).
    pub fn with_ef(mut self, ef: usize) -> Self {
        self.ef = Some(ef);
        self
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

/// Size below which the knowledge base is searched with the exact scan and no HNSW index is
/// built (P2, A2). Chosen from the measurement in `verification/c2/` (owner decision
/// 2026-11-06): on all cores the exact scan stays near 1 ms up to 10,000 items, its recall is
/// always 1.0, while HNSW recall on uniform data falls to 0.48 at 10,000 and building the
/// index takes up to 30 s.
pub const EXACT_SCAN_MAX_ITEMS: usize = 10_000;

/// The central in-memory knowledge base with pluggable search backends.
pub struct KnowledgeStore {
    /// Metadata for each item (id + arbitrary JSON fields). Private (D11): the vectors, the
    /// dimension and the index must stay in step, so callers read through accessors.
    items: Vec<SlimItem>,
    /// Exact f32 vectors, row `i` for `items[i]`. Shared with the HNSW index, never copied.
    matrix: Arc<VectorMatrix>,
    /// HNSW index (built on demand via `build_index`).
    hnsw: Option<HnswIndex>,
    /// Bumped by every load. An index built off-lock (P1) is installed only if this is
    /// still the value it was built from.
    generation: u64,
    /// How far the loaded vectors were from unit length before normalisation (A1).
    norm_report: NormReport,
    /// Item count from which an HNSW index is used (default `EXACT_SCAN_MAX_ITEMS`).
    exact_limit: usize,
}

/// Result of normalising a knowledge base at load (A1).
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct NormReport {
    /// Largest `| ||v|| - 1 |` over all rows, measured before normalisation.
    pub max_deviation: f64,
    /// Rows that were rescaled (deviation above `NORM_TOLERANCE`).
    pub rescaled: usize,
}

/// A row whose length differs from 1 by no more than this is left as it is, so a knowledge
/// base that is already unit length keeps its exact bits.
pub const NORM_TOLERANCE: f64 = 1e-5;

/// L2 norm, accumulated in f64 so the deviation report is not limited by f32 rounding.
fn l2_norm(v: &[f32]) -> f64 {
    v.iter().map(|&x| f64::from(x) * f64::from(x)).sum::<f64>().sqrt()
}

/// Rescale `row` to unit length in place (a zero row stays zero). Returns the deviation
/// `| ||row|| - 1 |` before the change and whether the row was rescaled.
fn normalise_row(row: &mut [f32]) -> (f64, bool) {
    let n = l2_norm(row);
    let dev = (n - 1.0).abs();
    if dev <= NORM_TOLERANCE || n == 0.0 {
        return (dev, false);
    }
    for x in row.iter_mut() {
        *x = (f64::from(*x) / n) as f32;
    }
    (dev, true)
}

/// Normalise every row of a flat buffer and report the deviation.
fn normalise_all(dim: usize, data: &mut [f32]) -> NormReport {
    let mut report = NormReport::default();
    if dim == 0 {
        return report;
    }
    for row in data.chunks_exact_mut(dim) {
        let (dev, changed) = normalise_row(row);
        report.max_deviation = report.max_deviation.max(dev);
        report.rescaled += usize::from(changed);
    }
    report
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
            matrix: Arc::new(VectorMatrix::default()),
            hnsw: None,
            generation: 0,
            norm_report: NormReport::default(),
            exact_limit: EXACT_SCAN_MAX_ITEMS,
        }
    }

    /// Change the item count from which an HNSW index is built and used (default
    /// `EXACT_SCAN_MAX_ITEMS`). Takes effect at the next `build_index` and `search`.
    pub fn set_exact_scan_limit(&mut self, items: usize) {
        self.exact_limit = items;
    }

    /// Vector dimensionality of the loaded knowledge base (0 when nothing is loaded).
    pub fn dim(&self) -> usize {
        self.matrix.dim()
    }

    /// Metadata of the loaded items, in load order.
    pub fn items(&self) -> &[SlimItem] {
        &self.items
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

    /// Deviation from unit length found at the last load, before normalisation (A1).
    pub fn norm_report(&self) -> NormReport {
        self.norm_report
    }

    // ── Loading ───────────────────────────────────────────────────────────────

    /// Load knowledge base from a JSON file.
    ///
    /// The file is read and parsed once; vectors are kept as exact f32.
    /// Files larger than `MAX_KB_BYTES`, zero-length vectors and non-finite components are
    /// rejected, and so is a component beyond f16 range, because the SQLite cache stores f16.
    /// Returns the number of items loaded.
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

        // D12: an empty file is an error, not a silent no-op that leaves the old KB in place.
        if count == 0 {
            return Err(EngineError::InvalidInput("the knowledge base file contains no items".into()));
        }

        let new_dim = raw[0].vector.len();
        if new_dim == 0 {
            return Err(EngineError::InvalidInput("the first item has an empty vector (dimension 0)".into()));
        }

        // D1: build into locals; `self` is touched only after every item has passed.
        let mut items = Vec::with_capacity(count);
        let mut data: Vec<f32> = Vec::with_capacity(count * new_dim);
        for (i, item) in raw.into_iter().enumerate() {
            if item.vector.len() != new_dim {
                return Err(EngineError::DimensionMismatch { expected: new_dim, got: item.vector.len() });
            }
            if let Some(x) = item.vector.iter().find(|x| !x.is_finite() || x.abs() > F16_MAX_ABS) {
                return Err(EngineError::InvalidInput(format!(
                    "item #{i} has a vector component {x} that is not finite or exceeds {F16_MAX_ABS}"
                )));
            }
            data.extend_from_slice(&item.vector);
            items.push(SlimItem { id: item.id, metadata: item.metadata });
        }

        let report = normalise_all(new_dim, &mut data);
        self.install(items, VectorMatrix::from_flat(new_dim, data)?, report);
        Ok(count)
    }

    /// Replace the whole knowledge base in one step: items, vectors, dimension, a dropped
    /// index and a new generation. The only place the parallel vectors change.
    fn install(&mut self, items: Vec<SlimItem>, matrix: VectorMatrix, norm_report: NormReport) {
        debug_assert_eq!(items.len(), matrix.len());
        self.norm_report = norm_report;
        self.items = items;
        self.matrix = matrix.into_shared();
        self.hnsw = None;
        self.generation += 1;
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
        // P2: a knowledge base under the threshold is searched exactly, so there is nothing to build.
        if !self.wants_index() {
            self.hnsw = None;
            return;
        }
        // `&mut self` is held throughout, so the generation cannot change.
        self.hnsw = Some(self.index_job().build().index);
    }

    /// Whether this knowledge base is large enough for an HNSW index (`EXACT_SCAN_MAX_ITEMS`).
    /// Under it, `build_index` does nothing and `search` scans exactly.
    pub fn wants_index(&self) -> bool {
        self.items.len() >= self.exact_limit
    }

    /// P1: hand out what an index build needs (a shared matrix, no copy), so the build can run
    /// without holding the engine lock. Pair with `install_index`.
    pub fn index_job(&self) -> IndexJob {
        // P3: the matrix is shared, not copied; the store never mutates it in place.
        IndexJob { generation: self.generation, matrix: Arc::clone(&self.matrix) }
    }

    /// Install an index built from `index_job`. Fails when the knowledge base was loaded
    /// again in the meantime (the index would point at the wrong items).
    pub fn install_index(&mut self, built: BuiltIndex) -> Result<usize, EngineError> {
        if built.generation != self.generation {
            return Err(EngineError::InvalidInput(
                "the knowledge base was reloaded while the index was being built; build it again".into(),
            ));
        }
        let n = self.items.len();
        self.hnsw = Some(built.index);
        Ok(n)
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
        // P2/A2: below the threshold the exact scan answers, even with an index built.
        // A1: scores are cosine similarities, so the query is brought to unit length too.
        let mut unit = query.vector.to_vec();
        normalise_row(&mut unit);
        let results = match &self.hnsw {
            Some(hnsw) if self.items.len() >= self.exact_limit => {
                let ef = query.ef.unwrap_or_else(|| crate::hnsw_index::default_ef(top_k));
                hnsw.search_with(&unit, top_k, ef)
                    .into_iter()
                    .map(|(score, idx)| self.make_result(score, idx))
                    .collect::<Vec<_>>()
            }
            _ => self.scan(&unit, top_k),
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

    /// Parallel rayon linear scan — O(N) over the shared f32 matrix, with the same `dot` the
    /// index uses. Results are ordered by score descending, equal scores by idx ascending (R1).
    ///
    /// Suitable for ≤ ~10 K items or when HNSW is not yet built.
    pub fn linear_search(&self, query: &[f32], top_k: usize) -> Vec<SearchResult> {
        let mut unit = query.to_vec();
        normalise_row(&mut unit);
        self.scan(&unit, top_k)
    }

    /// The exact scan itself; `query` must already be unit length.
    fn scan(&self, query: &[f32], top_k: usize) -> Vec<SearchResult> {
        let matrix = &self.matrix;
        let mut scores: Vec<(f32, usize)> = (0..matrix.len())
            .into_par_iter()
            .map(|idx| (dot(query, matrix.row(idx)), idx))
            .collect();

        // A total order (the idx is unique), so the selection and the sort are deterministic.
        let by_rank = |a: &(f32, usize), b: &(f32, usize)| b.0.total_cmp(&a.0).then(a.1.cmp(&b.1));
        // O(N) partial selection — avoids full sort.
        if scores.len() > top_k {
            scores.select_nth_unstable_by(top_k, by_rank);
            scores.truncate(top_k);
        }
        scores.sort_unstable_by(by_rank);

        scores
            .into_iter()
            .map(|(score, idx)| self.make_result(score, idx))
            .collect()
    }

    // ── SQLite persistence ────────────────────────────────────────────────────

    /// Persist the current knowledge base to a SQLite database. Vectors are written as f16, so
    /// a knowledge base loaded back from the cache scores slightly differently from the JSON
    /// it came from.
    ///
    /// Subsequent startups can call `load_from_sqlite` for faster startup
    /// (avoids re-parsing the JSON file).
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

        for (i, item) in self.items.iter().enumerate() {
            let metadata_json = serde_json::to_string(&item.metadata)?;
            // The cache format is f16, little-endian (the runtime copy stays exact f32).
            let blob: Vec<u8> = self
                .matrix
                .row(i)
                .iter()
                .flat_map(|&x| f16::from_f32(x).to_le_bytes())
                .collect();
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

        // D2: parse every row into locals, fail on the first bad one, and touch `self` only at
        // the end, so a bad cache neither skips rows silently nor keeps the previous dimension.
        let mut rows = stmt
            .query_map([], |row| {
                Ok((row.get::<_, Option<String>>(0)?, row.get::<_, String>(1)?, row.get::<_, Vec<u8>>(2)?))
            })
            .map_err(|e| EngineError::Database(e.to_string()))?;

        let mut items = Vec::new();
        let mut data: Vec<f32> = Vec::new();
        let mut dim = 0usize;
        let mut n = 0usize;
        for row in &mut rows {
            let (id, meta_json, blob) = row.map_err(|e| EngineError::Database(format!("cache row #{n}: {e}")))?;
            if blob.is_empty() || blob.len() % 2 != 0 {
                return Err(EngineError::InvalidInput(format!(
                    "cache row #{n} has a vector of {} bytes (not a whole number of f16 values)",
                    blob.len()
                )));
            }
            let metadata: serde_json::Map<String, serde_json::Value> = serde_json::from_str(&meta_json)
                .map_err(|e| EngineError::InvalidInput(format!("cache row #{n} has invalid metadata: {e}")))?;
            let row: Vec<f32> = blob
                .chunks_exact(2)
                .map(|b| f16::from_le_bytes([b[0], b[1]]).to_f32())
                .collect();
            if n == 0 {
                dim = row.len();
            } else if row.len() != dim {
                return Err(EngineError::DimensionMismatch { expected: dim, got: row.len() });
            }
            if row.iter().any(|x| !x.is_finite()) {
                return Err(EngineError::InvalidInput(format!("cache row #{n} has a non-finite component")));
            }
            items.push(SlimItem { id, metadata });
            data.extend_from_slice(&row);
            n += 1;
        }
        drop(rows);
        if n == 0 {
            return Err(EngineError::InvalidInput("the cache contains no items".into()));
        }

        let report = normalise_all(dim, &mut data);
        self.install(items, VectorMatrix::from_flat(dim, data)?, report);
        Ok(n)
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

// ─── Off-lock index build (P1) ───────────────────────────────────────────────

/// The vectors to index (a shared, immutable matrix), so the job can move to another thread.
pub struct IndexJob {
    generation: u64,
    matrix: Arc<VectorMatrix>,
}

impl IndexJob {
    /// CPU-intensive; run it on a blocking thread, with no engine lock held.
    pub fn build(self) -> BuiltIndex {
        BuiltIndex {
            generation: self.generation,
            index: HnswIndex::build_from_matrix(HnswParams::default(), self.matrix),
        }
    }
}

/// A finished index tagged with the generation it was built from.
pub struct BuiltIndex {
    generation: u64,
    index: HnswIndex,
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

// ─── Tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn make_store(n: usize, dim: usize) -> KnowledgeStore {
        let mut store = KnowledgeStore::new();
        let mut data = Vec::with_capacity(n * dim);
        let mut items = Vec::with_capacity(n);
        for i in 0..n {
            data.extend((0..dim).map(|j| if j == i % dim { 1.0f32 } else { 0.0 }));
            items.push(SlimItem {
                id: Some(format!("item-{i}")),
                metadata: serde_json::Map::new(),
            });
        }
        store.install(items, VectorMatrix::from_flat(dim, data).unwrap(), NormReport::default());
        store
    }

    fn write_kb(json: &str) -> String {
        // A counter, not the JSON length: tests run in parallel, and two tests that write the
        // same content (so the same length) used to share one file and delete it under each other.
        static NEXT: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);
        let n = NEXT.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        let path = std::env::temp_dir().join(format!("vect-ks-{}-{n}.json", std::process::id()));
        std::fs::write(&path, json).unwrap();
        path.to_string_lossy().into_owned()
    }

    /// Change-log S8: a fixed-seed randomized property test (no extra crate). For many random
    /// stores and queries, `linear_search` equals a naive reference: the best `top_k` rows by
    /// score descending, ties by index ascending, never more than `top_k` of them.
    #[test]
    fn s8_linear_search_matches_naive_reference() {
        let mut state: u64 = 0x9E37_79B9_7F4A_7C15; // fixed seed, xorshift64
        let mut next = move || {
            state ^= state << 13;
            state ^= state >> 7;
            state ^= state << 17;
            state
        };
        for _ in 0..200 {
            let n = (next() % 40) as usize + 1;
            let dim = (next() % 20) as usize + 1;
            // Small integers, so equal scores (ties) actually occur.
            let mut data = Vec::with_capacity(n * dim);
            for _ in 0..n * dim {
                data.push((next() % 5) as f32 - 2.0);
            }
            let query: Vec<f32> = (0..dim).map(|_| (next() % 5) as f32 - 2.0).collect();
            let top_k = (next() % 50) as usize;

            let items = (0..n).map(|i| SlimItem { id: Some(format!("i{i}")), metadata: serde_json::Map::new() }).collect();
            let mut store = KnowledgeStore::new();
            store.install(items, VectorMatrix::from_flat(dim, data.clone()).unwrap(), NormReport::default());

            // Since A1 the store scores cosine similarity: the query is brought to unit length
            // first (the rows are already used as given, because `install` skips the load step).
            let mut unit_query = query.clone();
            normalise_row(&mut unit_query);
            let mut expected: Vec<(f32, usize)> =
                (0..n).map(|i| (dot(&unit_query, &data[i * dim..(i + 1) * dim]), i)).collect();
            expected.sort_by(|a, b| b.0.total_cmp(&a.0).then(a.1.cmp(&b.1)));
            expected.truncate(top_k);

            let got: Vec<(f32, usize)> = store.linear_search(&query, top_k).iter().map(|r| (r.score, r.idx)).collect();
            assert!(got.len() <= top_k);
            assert_eq!(got, expected, "n={n} dim={dim} top_k={top_k}");
        }
    }

    #[test]
    fn d8_zero_dim_rejected() {
        let mut store = KnowledgeStore::new();
        let p = write_kb(r#"[{"id":"a","vector":[]}]"#);
        assert!(store.load_from_path(&p).is_err());
        assert_eq!(store.dim(), 0);
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
    fn d1_failed_load_keeps_previous_kb() {
        let mut store = KnowledgeStore::new();
        let ok = write_kb(r#"[{"id":"a","vector":[1.0,0.0]},{"id":"b","vector":[0.0,1.0]}]"#);
        assert_eq!(store.load_from_path(&ok).unwrap(), 2);
        let bad = write_kb(r#"[{"id":"c","vector":[1.0,0.0]},{"id":"d","vector":[1.0]}]"#);
        assert!(store.load_from_path(&bad).is_err());
        assert_eq!(store.len(), 2, "validation happens before the old KB is dropped");
        assert_eq!(store.dim(), 2);
        std::fs::remove_file(ok).ok();
        std::fs::remove_file(bad).ok();
    }

    #[test]
    fn d12_empty_file_is_an_error_and_keeps_the_previous_kb() {
        let mut store = KnowledgeStore::new();
        let ok = write_kb(r#"[{"id":"a","vector":[1.0,0.0]}]"#);
        store.load_from_path(&ok).unwrap();
        let empty = write_kb("[]");
        let e = store.load_from_path(&empty).unwrap_err().to_string();
        assert!(e.contains("no items"), "{e}");
        assert_eq!(store.len(), 1);
        std::fs::remove_file(ok).ok();
        std::fs::remove_file(empty).ok();
    }

    fn cache_path(tag: &str) -> String {
        std::env::temp_dir()
            .join(format!("vect-cache-{tag}-{}.db", std::process::id()))
            .to_string_lossy()
            .into_owned()
    }

    #[test]
    fn d2_cache_load_resets_dim() {
        // A 4-dim KB is loaded first; a 2-dim cache must replace the dimension, not inherit it.
        let mut store = make_store(3, 4);
        let src = write_kb(r#"[{"id":"a","vector":[1.0,0.0]},{"id":"b","vector":[0.0,1.0]}]"#);
        let mut two = KnowledgeStore::new();
        two.load_from_path(&src).unwrap();
        let db = cache_path("dim");
        two.save_to_sqlite(&db).unwrap();
        assert_eq!(store.load_from_sqlite(&db).unwrap(), 2);
        assert_eq!(store.dim(), 2);
        std::fs::remove_file(src).ok();
        std::fs::remove_file(db).ok();
    }

    #[test]
    fn d2_cache_with_a_bad_row_fails_and_keeps_the_previous_kb() {
        let mut store = make_store(3, 4);
        let db = cache_path("bad");
        {
            let conn = rusqlite::Connection::open(&db).unwrap();
            conn.execute_batch(
                "CREATE TABLE kb_items (idx INTEGER PRIMARY KEY, id TEXT, metadata TEXT NOT NULL, vector_f16 BLOB NOT NULL);
                 INSERT INTO kb_items VALUES (0, 'a', '{}', x'00380038');
                 INSERT INTO kb_items VALUES (1, 'b', '{}', x'003800');
                 INSERT INTO kb_items VALUES (2, 'c', '{}', x'0038');",
            )
            .unwrap();
        }
        let e = store.load_from_sqlite(&db).unwrap_err().to_string();
        assert!(e.contains("row #1"), "{e}");
        assert_eq!(store.len(), 3, "the previous KB is kept");
        assert_eq!(store.dim(), 4);
        // A dimension change between rows is an error too.
        {
            let conn = rusqlite::Connection::open(&db).unwrap();
            conn.execute_batch("UPDATE kb_items SET vector_f16 = x'00380038' WHERE idx = 1;").unwrap();
        }
        assert!(store.load_from_sqlite(&db).is_err());
        assert_eq!(store.len(), 3);
        std::fs::remove_file(db).ok();
    }

    #[test]
    fn p1_index_built_off_lock_is_rejected_when_the_kb_was_reloaded() {
        let mut store = make_store(20, 4);
        let job = store.index_job();
        let built = job.build(); // runs with no access to the store
        assert_eq!(store.install_index(built).unwrap(), 20);
        assert!(store.has_index());

        let job = store.index_job();
        let built = job.build();
        let other = write_kb(r#"[{"id":"x","vector":[1.0,0.0,0.0,0.0]}]"#);
        store.load_from_path(&other).unwrap(); // generation moves on, old index dropped
        assert!(!store.has_index());
        assert!(store.install_index(built).is_err(), "a stale index must not be installed");
        assert!(!store.has_index());
        std::fs::remove_file(other).ok();
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
    fn r1_ties_ordered_by_idx() {
        // Five identical vectors tie on every score: both search paths must return idx order.
        let mut store = KnowledgeStore::new();
        let kb = write_kb(
            r#"[{"id":"a","vector":[1.0,0.0]},{"id":"b","vector":[1.0,0.0]},{"id":"c","vector":[1.0,0.0]},{"id":"d","vector":[1.0,0.0]},{"id":"e","vector":[1.0,0.0]},{"id":"f","vector":[0.0,1.0]}]"#,
        );
        store.load_from_path(&kb).unwrap();
        let idxs = |r: Vec<SearchResult>| r.into_iter().map(|x| x.idx).collect::<Vec<_>>();
        assert_eq!(idxs(store.linear_search(&[1.0, 0.0], 4)), vec![0, 1, 2, 3]);
        store.build_index();
        assert_eq!(idxs(store.search(&[1.0, 0.0], 4)), vec![0, 1, 2, 3]);
        std::fs::remove_file(kb).ok();
    }

    #[test]
    fn p2_small_kb_uses_exact() {
        // Under the threshold nothing is built and every answer is the exact scan.
        let n = EXACT_SCAN_MAX_ITEMS - 1;
        let dim = 8;
        let mut store = KnowledgeStore::new();
        let mut data = Vec::new();
        let mut items = Vec::new();
        let mut x = 0x2545_F491_4F6C_DD1Du64;
        for i in 0..n {
            let mut row: Vec<f32> = (0..dim)
                .map(|_| {
                    x ^= x << 13;
                    x ^= x >> 7;
                    x ^= x << 17;
                    (x >> 40) as f32 / (1u64 << 24) as f32 - 0.5
                })
                .collect();
            let norm = row.iter().map(|v| v * v).sum::<f32>().sqrt();
            row.iter_mut().for_each(|v| *v /= norm);
            data.extend(row);
            items.push(SlimItem { id: Some(format!("i{i}")), metadata: serde_json::Map::new() });
        }
        store.install(items, VectorMatrix::from_flat(dim, data).unwrap(), NormReport::default());
        assert!(!store.wants_index());
        store.build_index();
        assert!(!store.has_index(), "a small knowledge base builds no index");
        for q in 0..n.min(40) {
            let query = store.matrix.row(q).to_vec();
            let via_search: Vec<usize> = store.search(&query, 10).into_iter().map(|r| r.idx).collect();
            let exact: Vec<usize> = store.linear_search(&query, 10).into_iter().map(|r| r.idx).collect();
            assert_eq!(via_search, exact);
        }
    }

    #[test]
    fn p2_large_kb_builds_and_uses_the_index() {
        // At the threshold the index is built and answers; each stored vector finds itself first.
        let limit = 400; // a small limit keeps the debug-build test fast
        let n = limit;
        let dim = 8;
        let mut data = Vec::with_capacity(n * dim);
        let mut items = Vec::with_capacity(n);
        let mut x = 0x9E37_79B9_7F4A_7C15u64;
        for i in 0..n {
            let mut row: Vec<f32> = (0..dim)
                .map(|_| {
                    x ^= x << 13;
                    x ^= x >> 7;
                    x ^= x << 17;
                    (x >> 40) as f32 / (1u64 << 24) as f32 - 0.5
                })
                .collect();
            let norm = row.iter().map(|v| v * v).sum::<f32>().sqrt();
            row.iter_mut().for_each(|v| *v /= norm);
            data.extend(row);
            items.push(SlimItem { id: Some(format!("i{i}")), metadata: serde_json::Map::new() });
        }
        let mut store = KnowledgeStore::new();
        store.set_exact_scan_limit(limit);
        store.install(items, VectorMatrix::from_flat(dim, data).unwrap(), NormReport::default());
        assert!(store.wants_index());
        store.build_index();
        assert!(store.has_index());
        let hits = (0..n)
            .filter(|&q| {
                let query = store.matrix.row(q).to_vec();
                store.search(&query, 1).first().is_some_and(|r| r.idx == q)
            })
            .count();
        assert!(hits >= n * 97 / 100, "the index found itself for {hits}/{n} stored vectors");
        // The same store with the default limit scans exactly and builds nothing.
        store.set_exact_scan_limit(EXACT_SCAN_MAX_ITEMS);
        assert!(!store.wants_index());
        store.build_index();
        assert!(!store.has_index());
    }

    #[test]
    fn a1_unnormalised_kb_is_normalised() {
        // "long" and "far" point away from the query; "same" points the same way but is 5 long.
        // Raw dot products would rank by length; cosine ranks by direction.
        let mut store = KnowledgeStore::new();
        let kb = write_kb(
            r#"[{"id":"far","vector":[0.0,3.0]},{"id":"near","vector":[0.8,0.6]},{"id":"long","vector":[0.0,10.0]},{"id":"same","vector":[5.0,0.0]}]"#,
        );
        assert_eq!(store.load_from_path(&kb).unwrap(), 4);
        std::fs::remove_file(kb).ok();

        let rep = store.norm_report();
        assert_eq!(rep.rescaled, 3, "near is already unit length");
        assert!((rep.max_deviation - 9.0).abs() < 1e-9, "{rep:?}");
        for i in 0..store.len() {
            let n: f64 = store.matrix.row(i).iter().map(|&x| f64::from(x).powi(2)).sum::<f64>().sqrt();
            assert!((n - 1.0).abs() < 1e-6, "row {i} has norm {n}");
        }
        // The query is scaled too: a query of length 7 gives cosine scores.
        let top = store.linear_search(&[7.0, 0.0], 4);
        assert_eq!(top[0].id.as_deref(), Some("same"));
        assert!((top[0].score - 1.0).abs() < 1e-6);
        assert_eq!(top[1].id.as_deref(), Some("near"));
        assert!((top[1].score - 0.8).abs() < 1e-6);
        assert!(top[2].score.abs() < 1e-6 && top[3].score.abs() < 1e-6);
    }

    #[test]
    fn a4_ef_is_optional_and_defaults_to_twice_top_k() {
        assert_eq!(crate::hnsw_index::default_ef(10), 64);
        assert_eq!(crate::hnsw_index::default_ef(50), 100);
        let q = [1.0f32, 0.0, 0.0, 0.0];
        assert_eq!(SearchQuery::new(&q).ef, None);
        assert_eq!(SearchQuery::new(&q).with_ef(300).ef, Some(300));
    }

    #[test]
    fn p3_index_and_store_share_one_matrix() {
        let mut store = make_store(30, 8);
        let job = store.index_job();
        assert!(Arc::ptr_eq(&job.matrix, &store.matrix), "the index job must not copy the vectors");
        let built = job.build();
        store.install_index(built).unwrap();
        assert!(store.has_index());
    }

    #[test]
    fn a3_scores_are_exact_f32_not_f16_rounded() {
        // 0.6 is not representable in f16 (it rounds to 0.6000977); f32 keeps it. The vector is
        // (0.6, 0.8), already unit length, so A1 normalisation leaves it alone.
        let mut store = KnowledgeStore::new();
        let kb = write_kb(r#"[{"id":"a","vector":[0.6,0.8]}]"#);
        store.load_from_path(&kb).unwrap();
        let score = store.linear_search(&[1.0, 0.0], 1)[0].score;
        assert_eq!(score, 0.6f32);
        std::fs::remove_file(kb).ok();
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
