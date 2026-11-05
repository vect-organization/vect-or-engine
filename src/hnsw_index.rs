// Copyright (c) 2026 1abcdefggs
// SPDX-License-Identifier: MIT
// https://github.com/1abcdefggs/vect-or-engine

#![forbid(unsafe_code)]

//! Hierarchical Navigable Small World (HNSW) index — Phase 3.
//!
//! Provides O(log N) approximate nearest-neighbor search.
//! Reference: Malkov & Yashunin (2016) "Efficient and robust approximate nearest
//! neighbor search using Hierarchical Navigable Small World graphs."
//!
//! Design decisions:
//! - Pure Rust, zero external ANN crates (no FFI dependencies).
//! - Negative dot-product as distance (HNSW minimises; caller maximises similarity).
//! - Safe 8-wide chunked dot product — the compiler vectorises it at the target's baseline (SSE2 by default).
//! - `select_nth_unstable` for O(N) top-K without full sort.

use std::cmp::Ordering;
use std::collections::{BinaryHeap, HashSet};

// ─── Tuneable constants ────────────────────────────────────────────────────────
/// Max bi-directional connections per node per layer.
const M: usize = 16;
/// Build-time search width (higher = better recall, slower build).
const EF_CONSTRUCTION: usize = 200;
/// Query-time search width.
const EF_SEARCH: usize = 64;
// mL = 1/ln(M) is computed at runtime in next_layer() since ln() is not const-stable.

/// Index parameters. `Default` reproduces the compiled-in constants above.
/// Experiments (benchmarks) use `HnswIndex::build_with` / `search_with` to vary them.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct HnswParams {
    /// Connections per node on upper layers.
    pub m: usize,
    /// Connections per node on layer 0 (the classic choice is 2 * m).
    pub m0: usize,
    /// Build-time search width.
    pub ef_construction: usize,
    /// Query-time search width.
    pub ef_search: usize,
    /// Use the diversity heuristic (Malkov & Yashunin, alg. 4) when choosing neighbours.
    pub heuristic: bool,
    /// With the heuristic, fill remaining slots with rejected candidates (keepPrunedConnections).
    pub keep_pruned: bool,
}

impl Default for HnswParams {
    fn default() -> Self {
        HnswParams {
            m: M,
            m0: 2 * M,
            ef_construction: EF_CONSTRUCTION,
            ef_search: EF_SEARCH,
            heuristic: true,
            keep_pruned: true,
        }
    }
}

// ─── Internal heap wrappers ────────────────────────────────────────────────────

/// Min-heap by distance — used for the candidate queue.
#[derive(PartialEq)]
struct MinCand(f32, usize);

impl Eq for MinCand {}
impl PartialOrd for MinCand {
    fn partial_cmp(&self, o: &Self) -> Option<Ordering> { Some(self.cmp(o)) }
}
impl Ord for MinCand {
    // Inverted so BinaryHeap becomes min-heap.
    fn cmp(&self, o: &Self) -> Ordering {
        o.0.total_cmp(&self.0)
    }
}

/// Max-heap by distance — used to maintain the ef-sized result window.
#[derive(PartialEq)]
struct MaxCand(f32, usize);

impl Eq for MaxCand {}
impl PartialOrd for MaxCand {
    fn partial_cmp(&self, o: &Self) -> Option<Ordering> { Some(self.cmp(o)) }
}
impl Ord for MaxCand {
    fn cmp(&self, o: &Self) -> Ordering {
        self.0.total_cmp(&o.0)
    }
}

// ─── HNSW Index ───────────────────────────────────────────────────────────────

/// Stored node: vector + per-layer neighbour lists.
struct Node {
    vector: Vec<f32>,
    /// `layers[lc]` = neighbour node indices at layer `lc`.
    layers: Vec<Vec<usize>>,
}

/// HNSW approximate nearest-neighbour index over f32 vectors.
#[derive(Default)]
pub struct HnswIndex {
    nodes: Vec<Node>,
    entry_point: Option<usize>,
    max_layer: usize,
    /// Simple counter-based seed for deterministic layer assignment.
    rng_state: u64,
    params: HnswParams,
}

impl HnswIndex {
    /// Create an empty index with the default parameters.
    pub fn new() -> Self {
        Self::with_params(HnswParams::default())
    }

    /// Create an empty index with the given parameters.
    pub fn with_params(params: HnswParams) -> Self {
        HnswIndex {
            nodes: Vec::new(),
            entry_point: None,
            max_layer: 0,
            rng_state: 0x9e3779b97f4a7c15,
            params,
        }
    }

    /// Build index from a slice of vectors (sequential; call from `spawn_blocking`).
    pub fn build(vectors: &[Vec<f32>]) -> Self {
        Self::build_with(HnswParams::default(), vectors)
    }

    /// Build index with explicit parameters (used by the benchmark).
    pub fn build_with(params: HnswParams, vectors: &[Vec<f32>]) -> Self {
        let mut idx = HnswIndex::with_params(params);
        for v in vectors {
            idx.insert(v);
        }
        idx
    }

    /// Number of indexed vectors.
    pub fn len(&self) -> usize {
        self.nodes.len()
    }

    pub fn is_empty(&self) -> bool {
        self.nodes.is_empty()
    }

    // ── Layer assignment ──────────────────────────────────────────────────────

    fn next_layer(&mut self) -> usize {
        // Xorshift64 PRNG — fast, dependency-free.
        let mut x = self.rng_state;
        x ^= x << 13;
        x ^= x >> 7;
        x ^= x << 17;
        self.rng_state = x;

        // Geometric distribution: floor(-ln(uniform) / ln(M))
        // mL = 1/ln(M) — computed here since ln() is not const-stable.
        let uniform = (x >> 11) as f64 / (1u64 << 53) as f64; // ∈ (0,1)
        let ml = 1.0_f64 / (self.params.m as f64).ln();
        let f = (-uniform.ln() * ml).floor() as usize;
        f.min(32) // cap at 32 layers
    }

    // ── Distance ─────────────────────────────────────────────────────────────

    /// Negative dot product — HNSW minimises distance; we want max similarity.
    #[inline(always)]
    fn dist(a: &[f32], b: &[f32]) -> f32 {
        -dot_product_simd(a, b)
    }

    // ── Core: search within one layer ────────────────────────────────────────

    fn search_layer(
        &self,
        query: &[f32],
        entry: usize,
        ef: usize,
        layer: usize,
    ) -> Vec<(f32, usize)> {
        let mut visited: HashSet<usize> = HashSet::new();
        let ep_dist = Self::dist(query, &self.nodes[entry].vector);

        let mut candidates: BinaryHeap<MinCand> = BinaryHeap::new();
        let mut window: BinaryHeap<MaxCand> = BinaryHeap::new();

        candidates.push(MinCand(ep_dist, entry));
        window.push(MaxCand(ep_dist, entry));
        visited.insert(entry);

        while let Some(MinCand(c_dist, c_idx)) = candidates.pop() {
            // Pruning: if nearest candidate is farther than worst in window → stop.
            if let Some(MaxCand(worst, _)) = window.peek() {
                if c_dist > *worst {
                    break;
                }
            }

            let neighbours = self
                .nodes
                .get(c_idx)
                .and_then(|n| n.layers.get(layer))
                .map(|v| v.as_slice())
                .unwrap_or(&[]);

            for &nb in neighbours {
                if visited.contains(&nb) {
                    continue;
                }
                visited.insert(nb);

                let nb_dist = Self::dist(query, &self.nodes[nb].vector);
                let worst = window.peek().map(|x| x.0).unwrap_or(f32::MAX);

                if nb_dist < worst || window.len() < ef {
                    candidates.push(MinCand(nb_dist, nb));
                    window.push(MaxCand(nb_dist, nb));
                    if window.len() > ef {
                        window.pop(); // evict the farthest
                    }
                }
            }
        }

        // Collect and sort ascending by distance.
        let mut result: Vec<(f32, usize)> =
            window.into_iter().map(|x| (x.0, x.1)).collect();
        result.sort_unstable_by(|a, b| a.0.total_cmp(&b.0));
        result
    }

    // ── Insert ────────────────────────────────────────────────────────────────

    /// Maximum neighbours kept on layer `lc` (layer 0 uses `m0`).
    fn cap_at(&self, lc: usize) -> usize {
        if lc == 0 { self.params.m0 } else { self.params.m }
    }

    /// Choose at most `max` neighbours from `cands`, which are `(distance to base, node)`
    /// pairs sorted by ascending distance.
    ///
    /// Without the heuristic this is the nearest `max` candidates. With it, a candidate is
    /// kept only if it is closer to the base node than to every neighbour already kept;
    /// rejected candidates fill the remaining slots, so the node keeps `max` links.
    fn select_neighbours(&self, cands: &[(f32, usize)], max: usize) -> Vec<usize> {
        if !self.params.heuristic {
            return cands.iter().take(max).map(|&(_, idx)| idx).collect();
        }
        let mut kept: Vec<usize> = Vec::with_capacity(max);
        let mut rejected: Vec<usize> = Vec::new();
        for &(d_base, c) in cands {
            if kept.len() >= max {
                break;
            }
            let c_vec = &self.nodes[c].vector;
            let diverse = kept
                .iter()
                .all(|&s| d_base < Self::dist(c_vec, &self.nodes[s].vector));
            if diverse {
                kept.push(c);
            } else {
                rejected.push(c);
            }
        }
        if self.params.keep_pruned {
            for c in rejected {
                if kept.len() >= max {
                    break;
                }
                kept.push(c);
            }
        }
        kept
    }

    /// Insert one vector into the index; returns the assigned node ID.
    pub fn insert(&mut self, vector: &[f32]) -> usize {
        let node_id = self.nodes.len();
        let node_layer = self.next_layer();

        self.nodes.push(Node {
            vector: vector.to_vec(),
            layers: vec![Vec::new(); node_layer + 1],
        });

        let Some(mut ep) = self.entry_point else {
            // First node — becomes entry point.
            self.entry_point = Some(node_id);
            self.max_layer = node_layer;
            return node_id;
        };

        let top = self.max_layer;

        // Phase 1: greedy descent from top_layer to node_layer+1.
        for lc in (node_layer + 1..=top).rev() {
            let nearest = self.search_layer(vector, ep, 1, lc);
            if let Some((_, nb)) = nearest.into_iter().next() {
                ep = nb;
            }
        }

        // Phase 2: insert with ef_construction at each layer ≤ node_layer.
        for lc in (0..=node_layer.min(top)).rev() {
            let cap = self.cap_at(lc);
            let candidates = self.search_layer(vector, ep, self.params.ef_construction, lc);

            // Select neighbours (nearest first, or the diversity heuristic).
            let neighbours: Vec<usize> = self.select_neighbours(&candidates, cap);

            // Set outgoing edges for new node.
            self.nodes[node_id].layers[lc] = neighbours.clone();

            // Add incoming edges + prune to the layer cap if needed.
            for &nb in &neighbours {
                if lc >= self.nodes[nb].layers.len() {
                    self.nodes[nb].layers.resize(lc + 1, Vec::new());
                }
                let mut conn = self.nodes[nb].layers[lc].clone();
                conn.push(node_id);

                if conn.len() > cap {
                    let nb_vec = &self.nodes[nb].vector;
                    let mut distances: Vec<(f32, usize)> = conn.iter()
                        .map(|&idx| (Self::dist(nb_vec, &self.nodes[idx].vector), idx))
                        .collect();
                    distances.sort_unstable_by(|a, b| a.0.total_cmp(&b.0));
                    conn = self.select_neighbours(&distances, cap);
                }
                self.nodes[nb].layers[lc] = conn;
            }

            if let Some((_, nearest)) = candidates.first() {
                ep = *nearest;
            }
        }

        // Update global entry point if new node is on a higher layer.
        if node_layer > self.max_layer {
            self.max_layer = node_layer;
            self.entry_point = Some(node_id);
        }

        node_id
    }

    // ── Query ─────────────────────────────────────────────────────────────────

    /// Search for `top_k` approximate nearest neighbours.
    ///
    /// Returns `(score, node_id)` pairs sorted by **descending** dot-product score.
    pub fn search(&self, query: &[f32], top_k: usize) -> Vec<(f32, usize)> {
        self.search_with(query, top_k, self.params.ef_search)
    }

    /// Search with an explicit query-time width `ef_search` (used by the benchmark).
    pub fn search_with(&self, query: &[f32], top_k: usize, ef_search: usize) -> Vec<(f32, usize)> {
        let Some(mut ep) = self.entry_point else {
            return vec![];
        };

        // Greedy descent from max_layer to layer 1.
        for lc in (1..=self.max_layer).rev() {
            let candidates = self.search_layer(query, ep, 1, lc);
            if let Some((_, nb)) = candidates.into_iter().next() {
                ep = nb;
            }
        }

        // Full ef-wide search at layer 0.
        let ef = ef_search.max(top_k);
        let candidates = self.search_layer(query, ep, ef, 0);

        candidates
            .into_iter()
            .take(top_k)
            .map(|(dist, idx)| (-dist, idx)) // negate back → dot-product score
            .collect()
    }
}

// ─── SIMD-friendly dot product ────────────────────────────────────────────────

/// Dot product in safe Rust: 8 independent accumulators over fixed-size chunks (no bounds
/// checks in the hot loop), then a scalar tail. With `opt-level=3` the compiler vectorises
/// it at the build target's baseline (SSE2 unless `target-cpu` is raised).
#[inline(always)]
pub fn dot_product_simd(a: &[f32], b: &[f32]) -> f32 {
    let n = a.len().min(b.len());
    let (a, b) = (&a[..n], &b[..n]);
    let mut acc = [0.0f32; 8];
    let mut ca = a.chunks_exact(8);
    let mut cb = b.chunks_exact(8);
    for (x, y) in (&mut ca).zip(&mut cb) {
        for k in 0..8 {
            acc[k] += x[k] * y[k];
        }
    }
    let mut sum = acc.iter().sum::<f32>();
    for (x, y) in ca.remainder().iter().zip(cb.remainder()) {
        sum += x * y;
    }
    sum
}

// ─── Tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dot_product_correctness() {
        let a = vec![1.0f32, 2.0, 3.0, 4.0, 5.0];
        let b = vec![1.0f32, 0.0, 1.0, 0.0, 1.0];
        assert!((dot_product_simd(&a, &b) - 9.0).abs() < 1e-5);
    }

    #[test]
    fn default_params_are_the_reference_configuration() {
        let p = HnswParams::default();
        assert_eq!(p.m, 16);
        assert_eq!(p.m0, 32);
        assert_eq!(p.ef_construction, 200);
        assert_eq!(p.ef_search, 64);
        assert!(p.heuristic);
        assert!(p.keep_pruned);
    }

    #[test]
    fn heuristic_keeps_a_different_direction_and_nearest_does_not() {
        // Base vector A = (1, 0). B is 8 degrees away, C is 12 degrees (close to B),
        // D is 30 degrees on the other side of A.
        let deg = |d: f32| (d.to_radians().cos(), d.to_radians().sin());
        let (b, c, d) = (deg(8.0), deg(12.0), deg(-30.0));
        let nodes = [vec![b.0, b.1], vec![c.0, c.1], vec![d.0, d.1]];
        // Candidates as (distance to A, node) sorted ascending; distance = -dot(A, v).
        let cands = vec![(-b.0, 0usize), (-c.0, 1usize), (-d.0, 2usize)];

        let mut heur = HnswIndex::with_params(HnswParams { heuristic: true, ..HnswParams::default() });
        let mut plain = HnswIndex::with_params(HnswParams { heuristic: false, ..HnswParams::default() });
        for v in &nodes {
            heur.insert(v);
            plain.insert(v);
        }

        // Nearest-M keeps B and C (both on the same side).
        assert_eq!(plain.select_neighbours(&cands, 2), vec![0, 1]);
        // The heuristic rejects C (closer to B than to A) and keeps D.
        assert_eq!(heur.select_neighbours(&cands, 2), vec![0, 2]);
    }

    #[test]
    fn hnsw_basic_search() {
        let mut idx = HnswIndex::new();
        for i in 0..64usize {
            let v: Vec<f32> = (0..64).map(|j| if j == i { 1.0 } else { 0.0 }).collect();
            idx.insert(&v);
        }
        assert_eq!(idx.len(), 64);

        let query: Vec<f32> = (0..64).map(|j| if j == 0 { 1.0 } else { 0.0 }).collect();
        let results = idx.search(&query, 1);
        assert!(!results.is_empty());
        assert_eq!(results[0].1, 0); // node 0 should be nearest to itself
    }
}
