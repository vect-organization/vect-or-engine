//! Reproducible benchmark for the engine (recall, latency, build time, validator scaling).
//!
//! Run (release is required for meaningful numbers):
//!   cargo run --release --example bench -- [--sizes 1000,10000,50000] [--queries 1000] [--out bench.md]
//!
//! Method
//! - Vectors are 384-dimensional and L2-normalised, generated from a fixed seed (no external data).
//! - Two data shapes: `uniform` (random directions, the hardest case for graph search) and
//!   `clustered` (50 centres plus noise, closer to real sentence embeddings).
//! - Vectors are quantised to f16 and back before indexing, like `KnowledgeStore::build_index`.
//! - Ground truth is an exact f32 scan over the original vectors, so recall includes the
//!   quantisation loss.
//! - Latency is single-threaded, one query at a time, wall clock per query.
//! - The validator section measures keyword-rule scaling; it does not use the N-API layer.

use std::fmt::Write as _;
use std::time::Instant;

use half::f16;
use vect_or_engine::hnsw_index::{dot_product_simd, HnswIndex, HnswParams};
use vect_or_engine::{MatchMode, Profile, Validator};

/// Default vector dimension; override with `--dim`.
const DEFAULT_DIM: usize = 384;
const TOP_K: usize = 10;
const SEED: u64 = 0x5EED_2026_1102;

struct Rng(u64);

impl Rng {
    fn next_u64(&mut self) -> u64 {
        // xorshift64*
        self.0 ^= self.0 >> 12;
        self.0 ^= self.0 << 25;
        self.0 ^= self.0 >> 27;
        self.0.wrapping_mul(0x2545_F491_4F6C_DD1D)
    }

    fn unit(&mut self) -> f32 {
        ((self.next_u64() >> 40) as f32 + 0.5) / (1u64 << 24) as f32
    }

    fn gauss(&mut self) -> f32 {
        let (u1, u2) = (self.unit(), self.unit());
        (-2.0 * u1.ln()).sqrt() * (std::f32::consts::TAU * u2).cos()
    }
}

fn normalize(v: &mut [f32]) {
    let n = v.iter().map(|x| x * x).sum::<f32>().sqrt().max(1e-12);
    v.iter_mut().for_each(|x| *x /= n);
}

fn random_unit(rng: &mut Rng, dim: usize) -> Vec<f32> {
    let mut v: Vec<f32> = (0..dim).map(|_| rng.gauss()).collect();
    normalize(&mut v);
    v
}

#[derive(Clone, Copy)]
enum Shape {
    Uniform,
    Clustered,
}

impl Shape {
    fn name(self) -> &'static str {
        match self {
            Shape::Uniform => "uniform",
            Shape::Clustered => "clustered",
        }
    }
}

fn generate(rng: &mut Rng, shape: Shape, centres: &[Vec<f32>], n: usize, dim: usize) -> Vec<Vec<f32>> {
    (0..n)
        .map(|_| match shape {
            Shape::Uniform => random_unit(rng, dim),
            Shape::Clustered => {
                let c = &centres[(rng.next_u64() % centres.len() as u64) as usize];
                let noise = random_unit(rng, dim);
                let mut v: Vec<f32> = c.iter().zip(&noise).map(|(a, b)| a + 0.7 * b).collect();
                normalize(&mut v);
                v
            }
        })
        .collect()
}

fn exact_top_k(data: &[Vec<f32>], query: &[f32], k: usize) -> Vec<usize> {
    let mut scores: Vec<(f32, usize)> = data
        .iter()
        .enumerate()
        .map(|(i, v)| (dot_product_simd(query, v), i))
        .collect();
    let k = k.min(scores.len());
    scores.select_nth_unstable_by(k - 1, |a, b| b.0.total_cmp(&a.0));
    scores.truncate(k);
    scores.sort_unstable_by(|a, b| b.0.total_cmp(&a.0));
    scores.into_iter().map(|(_, i)| i).collect()
}

fn percentile(sorted: &[f64], p: f64) -> f64 {
    let i = ((sorted.len() as f64 - 1.0) * p).round() as usize;
    sorted[i]
}

fn describe(p: &HnswParams) -> String {
    format!(
        "m={} m0={} efC={} efS={} heur={} keep={}",
        p.m, p.m0, p.ef_construction, p.ef_search, p.heuristic, p.keep_pruned
    )
}

struct Row {
    shape: &'static str,
    n: usize,
    params: String,
    build_s: f64,
    recall: f64,
    p50_us: f64,
    p95_us: f64,
    p99_us: f64,
    max_us: f64,
    qps: f64,
    scan_p50_us: f64,
    f32_mib: f64,
    f16_mib: f64,
}

fn run_vector(
    shape: Shape,
    n: usize,
    queries: usize,
    rng: &mut Rng,
    centres: &[Vec<f32>],
    dim: usize,
    params: &HnswParams,
) -> Row {
    let data = generate(rng, shape, centres, n, dim);
    let qs = generate(rng, shape, centres, queries, dim);

    // Same path as KnowledgeStore::build_index: f32 -> f16 -> f32, then build.
    let dequant: Vec<Vec<f32>> = data
        .iter()
        .map(|v| v.iter().map(|&x| f32::from(f16::from_f32(x))).collect())
        .collect();
    let t = Instant::now();
    let index = HnswIndex::build_with(*params, &dequant);
    let build_s = t.elapsed().as_secs_f64();

    let mut hits = 0usize;
    let mut lat_us: Vec<f64> = Vec::with_capacity(queries);
    for q in &qs {
        let t = Instant::now();
        let found = index.search_with(q, TOP_K, params.ef_search);
        lat_us.push(t.elapsed().as_secs_f64() * 1e6);
        let truth = exact_top_k(&data, q, TOP_K);
        hits += found.iter().filter(|(_, i)| truth.contains(i)).count();
    }
    let recall = hits as f64 / (queries * TOP_K) as f64;

    let sample = queries.min(100);
    let mut scan_us: Vec<f64> = Vec::with_capacity(sample);
    for q in qs.iter().take(sample) {
        let t = Instant::now();
        let _ = exact_top_k(&dequant, q, TOP_K);
        scan_us.push(t.elapsed().as_secs_f64() * 1e6);
    }

    lat_us.sort_by(|a, b| a.total_cmp(b));
    scan_us.sort_by(|a, b| a.total_cmp(b));
    let mean = lat_us.iter().sum::<f64>() / lat_us.len() as f64;
    let payload = (n * dim) as f64 / (1024.0 * 1024.0);

    Row {
        shape: shape.name(),
        n,
        params: describe(params),
        build_s,
        recall,
        p50_us: percentile(&lat_us, 0.50),
        p95_us: percentile(&lat_us, 0.95),
        p99_us: percentile(&lat_us, 0.99),
        max_us: *lat_us.last().unwrap(),
        qps: 1e6 / mean,
        scan_p50_us: percentile(&scan_us, 0.50),
        f32_mib: payload * 4.0,
        f16_mib: payload * 2.0,
    }
}

fn guideline_json(rules: usize) -> String {
    let mut s = String::from(r#"{"domain":"bench","version":"0","rules":["#);
    for i in 0..rules {
        if i > 0 {
            s.push(',');
        }
        let _ = write!(
            s,
            r#"{{"id":"R{i}","pattern":"BANNEDTERM{i:05}","message":"m","severity":"error"}}"#
        );
    }
    s.push_str("]}");
    s
}

fn document(chars: usize, rng: &mut Rng) -> String {
    const WORDS: [&str; 8] = ["alpha", "beta", "gamma", "delta", "sample", "record", "note", "value"];
    let mut s = String::with_capacity(chars + 16);
    while s.len() < chars {
        s.push_str(WORDS[(rng.next_u64() % WORDS.len() as u64) as usize]);
        s.push(if rng.next_u64() % 12 == 0 { '\n' } else { ' ' });
    }
    s.truncate(chars);
    s
}

/// Copy of `base` with a banned term (cycling through the rules) inserted every 500 characters,
/// so that the validator has real matches to report.
fn with_hits(base: &str, rules: usize) -> String {
    let mut s = String::with_capacity(base.len() + base.len() / 500 * 20);
    for (i, chunk) in base.as_bytes().chunks(500).enumerate() {
        s.push_str(std::str::from_utf8(chunk).unwrap_or(""));
        let _ = write!(s, " BANNEDTERM{:05} ", i % rules);
    }
    s
}

struct ValRow {
    mode: &'static str,
    text: &'static str,
    rules: usize,
    p50_us: f64,
    p95_us: f64,
}

fn run_validator(rng: &mut Rng) -> Vec<ValRow> {
    let clean = document(10_000, rng);
    let mut rows = Vec::new();
    for &rules in &[10usize, 100, 1000] {
        let profile = Profile::from_guideline_json_str(&guideline_json(rules)).expect("profile");
        let hits = with_hits(&clean, rules);
        for (text_name, text) in [("no-match", clean.as_str()), ("matches", hits.as_str())] {
            for (name, mode) in [("literal", MatchMode::Literal), ("regex", MatchMode::Regex), ("both", MatchMode::Both)] {
                let v = Validator::with_mode(&profile, mode).expect("validator");
                for _ in 0..20 {
                    let _ = v.validate(text);
                }
                let mut us: Vec<f64> = (0..300)
                    .map(|_| {
                        let t = Instant::now();
                        let r = v.validate(text);
                        std::hint::black_box(&r);
                        t.elapsed().as_secs_f64() * 1e6
                    })
                    .collect();
                us.sort_by(|a, b| a.total_cmp(b));
                rows.push(ValRow {
                    mode: name,
                    text: text_name,
                    rules,
                    p50_us: percentile(&us, 0.5),
                    p95_us: percentile(&us, 0.95),
                });
            }
        }
    }
    rows
}

fn arg_value(args: &[String], key: &str) -> Option<String> {
    args.iter().position(|a| a == key).and_then(|i| args.get(i + 1)).cloned()
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let sizes: Vec<usize> = arg_value(&args, "--sizes")
        .unwrap_or_else(|| "1000,10000,50000".to_string())
        .split(',')
        .filter_map(|s| s.trim().parse().ok())
        .collect();
    let queries: usize = arg_value(&args, "--queries").and_then(|s| s.parse().ok()).unwrap_or(1000);
    let out = arg_value(&args, "--out");

    // HNSW parameters (defaults reproduce the compiled-in constants).
    let mut params = HnswParams::default();
    if let Some(m) = arg_value(&args, "--m").and_then(|s| s.parse().ok()) {
        params.m = m;
        params.m0 = m;
    }
    if let Some(m0) = arg_value(&args, "--m0").and_then(|s| s.parse().ok()) {
        params.m0 = m0;
    }
    if let Some(v) = arg_value(&args, "--ef-construction").and_then(|s| s.parse().ok()) {
        params.ef_construction = v;
    }
    if let Some(v) = arg_value(&args, "--ef-search").and_then(|s| s.parse().ok()) {
        params.ef_search = v;
    }
    // The heuristic is on by default (see HnswParams::default); --no-heuristic turns it off.
    params.heuristic = !args.iter().any(|a| a == "--no-heuristic");
    params.keep_pruned = !args.iter().any(|a| a == "--no-keep-pruned");

    let dim: usize = arg_value(&args, "--dim").and_then(|s| s.parse().ok()).unwrap_or(DEFAULT_DIM);

    let mut rng = Rng(SEED);
    let centres: Vec<Vec<f32>> = (0..50).map(|_| random_unit(&mut rng, dim)).collect();

    let mut report = String::new();
    let threads = std::thread::available_parallelism().map(|n| n.get()).unwrap_or(0);
    let _ = writeln!(report, "# Engine benchmark\n");
    let _ = writeln!(
        report,
        "- crate version: {}\n- logical CPUs: {}\n- dimension: {}, top-k: {}, queries per row: {}, seed: {:#x}\n- HNSW parameters for this run: {}\n",
        env!("CARGO_PKG_VERSION"), threads, dim, TOP_K, queries, SEED, describe(&params)
    );

    let _ = writeln!(report, "## Vector search (HNSW, single thread)\n");
    let _ = writeln!(
        report,
        "| shape | N | params | build s | recall@{TOP_K} | p50 us | p95 us | p99 us | max us | QPS | exact scan p50 us | f32 MiB | f16 MiB |"
    );
    let _ = writeln!(report, "| :--- | ---: | :--- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |");
    for shape in [Shape::Uniform, Shape::Clustered] {
        for &n in &sizes {
            eprintln!("running {} N={n} ...", shape.name());
            let r = run_vector(shape, n, queries, &mut rng, &centres, dim, &params);
            let _ = writeln!(
                report,
                "| {} | {} | {} | {:.2} | {:.4} | {:.1} | {:.1} | {:.1} | {:.1} | {:.0} | {:.1} | {:.1} | {:.1} |",
                r.shape, r.n, r.params, r.build_s, r.recall, r.p50_us, r.p95_us, r.p99_us, r.max_us, r.qps, r.scan_p50_us, r.f32_mib, r.f16_mib
            );
        }
    }

    let _ = writeln!(report, "\n## Validator, keyword rules on a 10,000-character document\n");
    let _ = writeln!(report, "| mode | text | rules | p50 us | p95 us |");
    let _ = writeln!(report, "| :--- | :--- | ---: | ---: | ---: |");
    eprintln!("running validator ...");
    for r in run_validator(&mut rng) {
        let _ = writeln!(report, "| {} | {} | {} | {:.1} | {:.1} |", r.mode, r.text, r.rules, r.p50_us, r.p95_us);
    }

    let _ = writeln!(
        report,
        "\nNotes\n- Memory columns are the vector payload only (N x {dim} x 4 or x 2 bytes); graph and process memory are not measured\n- `exact scan` is a single-thread f32 scan, not the rayon `KnowledgeStore::linear_search`\n- Keyword rules are matched rule by rule (no shared automaton); only conflict items use Aho-Corasick, and that path is not measured here"
    );

    print!("{report}");
    if let Some(path) = out {
        std::fs::write(&path, &report).expect("write report");
        eprintln!("report written to {path}");
    }
}
