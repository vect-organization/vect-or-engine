// Copyright (c) 2026 1abcdefggs
// SPDX-License-Identifier: MIT
// https://github.com/1abcdefggs/vect-or-engine

//! Times `KnowledgeStore::linear_search`, the exact scan the editor uses, on a generated
//! knowledge base (the benchmark in `bench.rs` measures the HNSW index, not this path).
//!
//!   cargo run --release --example store_scan -- [--n 10000] [--dim 384] [--queries 200]
//!
//! Vectors are L2-normalised pseudo-random numbers from a fixed seed. The scan is the rayon
//! parallel one, so the figures depend on the number of logical CPUs (printed).

use std::time::Instant;

use vect_or_engine::KnowledgeStore;

struct Rng(u64);

impl Rng {
    fn next(&mut self) -> f32 {
        // xorshift64*, mapped to [-1, 1)
        self.0 ^= self.0 >> 12;
        self.0 ^= self.0 << 25;
        self.0 ^= self.0 >> 27;
        let x = self.0.wrapping_mul(0x2545_F491_4F6C_DD1D);
        ((x >> 40) as f32 / (1u64 << 24) as f32) * 2.0 - 1.0
    }

    fn unit_vector(&mut self, dim: usize) -> Vec<f32> {
        let mut v: Vec<f32> = (0..dim).map(|_| self.next()).collect();
        let norm = v.iter().map(|x| x * x).sum::<f32>().sqrt().max(1e-12);
        v.iter_mut().for_each(|x| *x /= norm);
        v
    }
}

fn arg(name: &str, default: usize) -> usize {
    let args: Vec<String> = std::env::args().collect();
    args.iter()
        .position(|a| a == name)
        .and_then(|i| args.get(i + 1))
        .and_then(|v| v.parse().ok())
        .unwrap_or(default)
}

fn main() {
    let (n, dim, queries) = (arg("--n", 10_000), arg("--dim", 384), arg("--queries", 200));
    let mut rng = Rng(0x5EED_2026_1105);

    let data: Vec<Vec<f32>> = (0..n).map(|_| rng.unit_vector(dim)).collect();
    let rows: Vec<serde_json::Value> = data
        .iter()
        .enumerate()
        .map(|(i, v)| serde_json::json!({ "id": format!("item-{i}"), "vector": v }))
        .collect();
    let path = std::env::temp_dir().join(format!("vect-store-scan-{}.json", std::process::id()));
    std::fs::write(&path, serde_json::to_vec(&rows).unwrap()).unwrap();

    let mut store = KnowledgeStore::new();
    let t = Instant::now();
    store.load_from_path(&path.to_string_lossy()).unwrap();
    let load_ms = t.elapsed().as_secs_f64() * 1e3;
    std::fs::remove_file(&path).ok();

    let qs: Vec<Vec<f32>> = (0..queries).map(|_| rng.unit_vector(dim)).collect();
    // Warm-up, so the thread pool and caches are ready.
    for q in qs.iter().take(10) {
        std::hint::black_box(store.linear_search(q, 10));
    }
    let mut times: Vec<f64> = qs
        .iter()
        .map(|q| {
            let t = Instant::now();
            std::hint::black_box(store.linear_search(q, 10));
            t.elapsed().as_secs_f64() * 1e6
        })
        .collect();
    times.sort_by(|a, b| a.total_cmp(b));
    let pct = |p: f64| times[((times.len() - 1) as f64 * p) as usize];

    // Reference: the same scan as a plain loop over the shared matrix (C1 and later), to show
    // how much of `linear_search` is the dot product itself.
    let matrix = vect_or_engine::VectorMatrix::from_rows(&data).unwrap();
    let mut ref_times: Vec<f64> = qs
        .iter()
        .take(50)
        .map(|q| {
            let t = Instant::now();
            let best = (0..matrix.len())
                .map(|i| vect_or_engine::dot(q, matrix.row(i)))
                .fold(f32::MIN, f32::max);
            std::hint::black_box(best);
            t.elapsed().as_secs_f64() * 1e6
        })
        .collect();
    ref_times.sort_by(|a, b| a.total_cmp(b));
    println!("plain dot loop over the matrix (1 thread): p50 {:.0} us", ref_times[ref_times.len() / 2]);

    println!("logical CPUs: {}", std::thread::available_parallelism().map_or(0, |n| n.get()));
    println!("n={n} dim={dim} queries={queries}");
    println!("load: {load_ms:.1} ms");
    println!("linear_search top-10: p50 {:.0} us, p95 {:.0} us, max {:.0} us", pct(0.5), pct(0.95), times[times.len() - 1]);
    println!("vector payload: {:.1} MiB", (n * dim * 4) as f64 / (1024.0 * 1024.0));
}
