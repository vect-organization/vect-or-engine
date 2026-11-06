# C3 (P5) build time and recall, before and after (2026-11-06)

Before: `verification/c2/bench_c2_exact_vs_hnsw_2026-11-06.md` (commit dcf2f7a). After: `bench_c3_after_2026-11-06.md` (epoch visited array, no list clones, whole candidate set as the next layer's entry). Same release settings, seed, 500 queries, 8 logical CPUs. The runs were not on an idle machine (the exact-scan column moves by up to 2x between runs), so read build seconds as a trend, not a precise ratio.

| shape | N | build s before | build s after | recall before | recall after |
| :--- | ---: | ---: | ---: | ---: | ---: |
| uniform | 1000 | 1.60 | 1.47 | 0.9734 | 0.9734 |
| uniform | 5000 | 11.82 | 10.15 | 0.6608 | 0.6614 |
| uniform | 10000 | 38.85 | 22.76 | 0.4896 | 0.4910 |
| uniform | 20000 | 77.36 | 49.94 | 0.3204 | 0.3112 |
| uniform | 50000 | 205.44 | 136.96 | 0.1836 | 0.1796 |
| clustered | 1000 | 0.78 | 0.68 | 1.0000 | 1.0000 |
| clustered | 5000 | 6.37 | 5.15 | 1.0000 | 1.0000 |
| clustered | 10000 | 15.89 | 11.27 | 1.0000 | 1.0000 |
| clustered | 20000 | 31.31 | 24.26 | 0.9990 | 0.9990 |
| clustered | 50000 | 86.34 | 78.67 | 0.9774 | 0.9774 |

- Build time falls by 8 to 40 percent; recall is unchanged within 0.01 (uniform 20000 and 50000 are 0.009 and 0.004 lower, clustered is identical)
- Uniform random vectors are the worst case for graph search; at efSearch 64 their recall is low before and after, and A4 (ef default `max(64, 2 * top_k)`, optional `ef`) is the lever for that

## Parallel build: deferred

- Realistic clustered data builds 10000 vectors in 11 s and 50000 in 79 s after C3; a 20000-item KB builds in 24 s, off the engine lock (P1)
- A parallel build changes the graph depending on thread timing unless it is made deterministic, which would break R2 (two builds equal) and the 0.3.x guarantee that a given KB gives a given graph
- Decision: deferred to a later phase; revisit when a KB above 50000 items is a real use case
