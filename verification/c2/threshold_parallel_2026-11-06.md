# C2 threshold, measured on the store's parallel scan (2026-11-06)

Tool: `examples/threshold.rs` (`cargo run --release --example threshold -- --sizes 1000,2000,5000,10000,20000 --queries 200`). 384 dimensions, top-10, 200 queries per row, 8 logical CPUs, the PC shared with other programs (single rows move by tens of percent; the clustered 20,000 exact figure looks like such an outlier). Exact scan = `KnowledgeStore::linear_search` (rayon, with query normalisation); HNSW = `HnswIndex::search_with` with the default ef, single thread; recall is measured against the exact result.

logical CPUs: 8
| shape | N | exact p50 us (all cores) | exact p50 us (1 thread) | HNSW p50 us (1 thread) | HNSW recall@10 | HNSW build s |
| :--- | ---: | ---: | ---: | ---: | ---: | ---: |
| uniform | 1000 | 196 | 245 | 277 | 0.9785 | 1.4 |
| uniform | 2000 | 374 | 543 | 514 | 0.8885 | 5.3 |
| uniform | 5000 | 505 | 953 | 565 | 0.6620 | 17.4 |
| uniform | 10000 | 760 | 1699 | 606 | 0.4830 | 30.8 |
| uniform | 20000 | 1828 | 4213 | 852 | 0.3335 | 116.5 |
| clustered | 1000 | 171 | 227 | 170 | 1.0000 | 0.8 |
| clustered | 2000 | 549 | 610 | 340 | 1.0000 | 2.8 |
| clustered | 5000 | 824 | 1647 | 327 | 1.0000 | 10.7 |
| clustered | 10000 | 817 | 1663 | 198 | 1.0000 | 15.8 |
| clustered | 20000 | 4398 | 7283 | 492 | 0.9990 | 43.7 |

## Decision

- The earlier threshold (2,000 items, `threshold_decision_2026-11-06.md`) compared HNSW with a single-thread exact scan. The store scans on all cores, which moves the crossover far up: exact stays near 1 ms to 10,000 items (1.7 ms on one thread), with recall 1.0
- HNSW wins by 0.1 to 0.6 ms at those sizes, but its recall on uniform data is 0.89 at 2,000, 0.66 at 5,000 and 0.48 at 10,000, and building takes 5 s at 2,000, 17 s at 5,000 and 31 s at 10,000
- Owner decision 2026-11-06: `EXACT_SCAN_MAX_ITEMS = 10_000`, and `buildIndex` builds nothing under it
- Not measured: 50,000 items; real-data recall with `recall_real.js` (the script is not in the repository; every real dictionary has fewer than 2,000 items, so they are scanned exactly and their recall is 1.0 by construction)
