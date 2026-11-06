//! Exact scan (the store's rayon `linear_search`) against HNSW, by knowledge-base size.
//! Measurement tool for the C2 threshold (see `verification/c2/`).
//!
//!   cargo run --release --example threshold -- [--sizes 1000,2000,5000,10000,20000] [--queries 200]
//!
//! For each size and data shape it prints: exact scan p50 on all cores and on one thread,
//! HNSW p50 (single thread, default ef), HNSW recall@10 against the exact result, build time.

use std::sync::Arc;
use std::time::Instant;

use vect_or_engine::hnsw_index::{default_ef, HnswIndex, HnswParams};
use vect_or_engine::{KnowledgeStore, VectorMatrix};

const DIM: usize = 384;
const TOP_K: usize = 10;

struct Rng(u64);
impl Rng {
    fn next_u64(&mut self) -> u64 {
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

fn random_unit(rng: &mut Rng) -> Vec<f32> {
    let mut v: Vec<f32> = (0..DIM).map(|_| rng.gauss()).collect();
    normalize(&mut v);
    v
}

fn generate(rng: &mut Rng, clustered: bool, centres: &[Vec<f32>], n: usize) -> Vec<Vec<f32>> {
    (0..n)
        .map(|_| {
            if !clustered {
                return random_unit(rng);
            }
            let c = &centres[(rng.next_u64() % centres.len() as u64) as usize];
            let noise = random_unit(rng);
            let mut v: Vec<f32> = c.iter().zip(&noise).map(|(a, b)| a + 0.7 * b).collect();
            normalize(&mut v);
            v
        })
        .collect()
}

fn arg_list(name: &str, default: &[usize]) -> Vec<usize> {
    let args: Vec<String> = std::env::args().collect();
    args.iter()
        .position(|a| a == name)
        .and_then(|i| args.get(i + 1))
        .map(|v| v.split(',').filter_map(|s| s.parse().ok()).collect())
        .unwrap_or_else(|| default.to_vec())
}

fn p50(mut v: Vec<f64>) -> f64 {
    v.sort_by(|a, b| a.total_cmp(b));
    v[v.len() / 2]
}

fn main() {
    let sizes = arg_list("--sizes", &[1000, 2000, 5000, 10000, 20000]);
    let queries = arg_list("--queries", &[200])[0];
    let one_thread = rayon::ThreadPoolBuilder::new().num_threads(1).build().unwrap();
    println!("logical CPUs: {}", std::thread::available_parallelism().map_or(0, |n| n.get()));
    println!("| shape | N | exact p50 us (all cores) | exact p50 us (1 thread) | HNSW p50 us (1 thread) | HNSW recall@10 | HNSW build s |");
    println!("| :--- | ---: | ---: | ---: | ---: | ---: | ---: |");

    for clustered in [false, true] {
        let mut rng = Rng(0x5EED_2026_1106 ^ clustered as u64);
        let centres: Vec<Vec<f32>> = (0..50).map(|_| random_unit(&mut rng)).collect();
        for &n in &sizes {
            let data = generate(&mut rng, clustered, &centres, n);
            let qs = generate(&mut rng, clustered, &centres, queries);

            // The store, through its real loader and search path.
            let path = std::env::temp_dir().join(format!("vect-thr-{}-{n}.json", std::process::id()));
            let rows: Vec<serde_json::Value> = data
                .iter()
                .enumerate()
                .map(|(i, v)| serde_json::json!({ "id": format!("i{i}"), "vector": v }))
                .collect();
            std::fs::write(&path, serde_json::to_vec(&rows).unwrap()).unwrap();
            let mut store = KnowledgeStore::new();
            store.load_from_path(&path.to_string_lossy()).unwrap();
            std::fs::remove_file(&path).ok();

            // Same vectors for the index (the store's matrix is private).
            let matrix = Arc::new(VectorMatrix::from_rows(&data).unwrap());
            let t = Instant::now();
            let index = HnswIndex::build_from_matrix(HnswParams::default(), Arc::clone(&matrix));
            let build_s = t.elapsed().as_secs_f64();

            for q in qs.iter().take(10) {
                std::hint::black_box(store.linear_search(q, TOP_K));
            }
            let mut exact_all = Vec::new();
            let mut exact_one = Vec::new();
            let mut hnsw_t = Vec::new();
            let mut hits = 0usize;
            for q in &qs {
                let t = Instant::now();
                let truth = store.linear_search(q, TOP_K);
                exact_all.push(t.elapsed().as_secs_f64() * 1e6);

                let t = Instant::now();
                one_thread.install(|| std::hint::black_box(store.linear_search(q, TOP_K)));
                exact_one.push(t.elapsed().as_secs_f64() * 1e6);

                // The store normalises the query; do the same for the index.
                let mut unit = q.clone();
                normalize(&mut unit);
                let t = Instant::now();
                let found = index.search_with(&unit, TOP_K, default_ef(TOP_K));
                hnsw_t.push(t.elapsed().as_secs_f64() * 1e6);

                let truth_ids: Vec<usize> = truth.iter().map(|r| r.idx).collect();
                hits += found.iter().filter(|(_, i)| truth_ids.contains(i)).count();
            }
            let recall = hits as f64 / (queries * TOP_K) as f64;
            println!(
                "| {} | {} | {:.0} | {:.0} | {:.0} | {:.4} | {:.1} |",
                if clustered { "clustered" } else { "uniform" },
                n,
                p50(exact_all),
                p50(exact_one),
                p50(hnsw_t),
                recall,
                build_s
            );
        }
    }
}
