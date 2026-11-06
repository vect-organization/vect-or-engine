# C2 exact-scan threshold (2026-11-06)

Source: `bench_c2_exact_vs_hnsw_2026-11-06.md` (release build, 8 logical CPUs, 500 queries per row, single-thread exact scan, HNSW m=16 efC=200 efS=64).

| shape | N | HNSW recall@10 | HNSW p50 us | exact p50 us |
| :--- | ---: | ---: | ---: | ---: |
| uniform | 1000 | 0.9734 | 257.8 | 122.3 |
| uniform | 5000 | 0.6608 | 615.8 | 865.8 |
| clustered | 1000 | 1.0000 | 178.8 | 106.4 |
| clustered | 5000 | 1.0000 | 215.5 | 797.3 |

- Exact scan costs about 0.11 to 0.17 ms per 1000 vectors on one thread; HNSW costs 0.18 to 0.26 ms at 1000 and stays flat
- Speed crossover on one thread is near 2000 vectors (uniform 2000 is about 0.24 ms for both); the store's scan uses rayon, so it is faster than the figure above
- Recall favours exact at every size (HNSW uniform 1000 is already 0.97)
- Decision: `EXACT_SCAN_MAX_ITEMS = 2000`; `search` uses the exact scan when the KB has fewer items, even if an index was built
- Not measured: `recall_real.js` (A2 real-data recall) was not run in this session
