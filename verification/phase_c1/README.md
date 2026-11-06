# Phase C1 measurements (flat f32 matrix, 16-lane dot)

Machine: i5-10310U, 8 logical CPUs, shared with other programs (values move by tens of percent).

- `bench_before_0.3.9.md` and `bench_after_0.3.10.md`: `cargo run --release --example bench -- --sizes 1000,3000,10000 --queries 1000`. Before, the benchmark rounded vectors through f16 like the old store; after, it indexes exact f32
- `examples/store_scan.rs`: times `KnowledgeStore::linear_search` (10,000 x 384). Old and new binaries were run alternately:

| Threads | 0.3.9 (f16) p50 | 0.3.10 (f32) p50 |
| :--- | ---: | ---: |
| 1 | 12.5, 25.7, 26.9 ms | 3.2, 3.7, 2.7 ms |
| 8 | 6.2, 5.7, 3.4, 3.2 ms | 0.93, 1.06, 1.06, 0.67 ms |

Run `RAYON_NUM_THREADS=1 cargo run --release --example store_scan` for the one-thread figure. A plain loop over the same matrix took 1.3-2.0 ms on one thread, so the scan overhead around the dot product is small.
