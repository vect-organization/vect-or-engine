# Engine benchmark

- crate version: 0.3.7
- logical CPUs: 8
- dimension: 384, top-k: 10, queries per row: 1000, seed: 0x5eed20261102
- HNSW parameters for this run: m=16 m0=32 efC=200 efS=64 heur=true

## Vector search (HNSW, single thread)

| shape | N | params | build s | recall@10 | p50 us | p95 us | p99 us | max us | QPS | exact scan p50 us | f32 MiB | f16 MiB |
| :--- | ---: | :--- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| uniform | 1000 | m=16 m0=32 efC=200 efS=64 heur=true | 1.88 | 0.9751 | 258.1 | 401.3 | 494.4 | 789.2 | 3572 | 121.9 | 1.5 | 0.7 |
| uniform | 10000 | m=16 m0=32 efC=200 efS=64 heur=true | 37.08 | 0.4786 | 1038.4 | 1543.1 | 1825.9 | 2220.3 | 939 | 2518.8 | 14.6 | 7.3 |
| clustered | 1000 | m=16 m0=32 efC=200 efS=64 heur=true | 1.58 | 0.9998 | 288.2 | 462.6 | 584.4 | 1081.1 | 3234 | 170.0 | 1.5 | 0.7 |
| clustered | 10000 | m=16 m0=32 efC=200 efS=64 heur=true | 19.58 | 0.9990 | 295.1 | 440.4 | 642.8 | 9665.1 | 3071 | 1914.3 | 14.6 | 7.3 |

## Validator, keyword rules on a 10,000-character document

| mode | text | rules | p50 us | p95 us |
| :--- | :--- | ---: | ---: | ---: |
| literal | no-match | 10 | 37.7 | 39.6 |
| regex | no-match | 10 | 30.2 | 52.1 |
| both | no-match | 10 | 44.0 | 73.2 |
| literal | matches | 10 | 37.8 | 57.1 |
| regex | matches | 10 | 31.3 | 32.4 |
| both | matches | 10 | 42.2 | 46.6 |
| literal | no-match | 100 | 176.9 | 209.1 |
| regex | no-match | 100 | 81.5 | 115.3 |
| both | no-match | 100 | 228.5 | 287.7 |
| literal | matches | 100 | 247.2 | 313.6 |
| regex | matches | 100 | 99.2 | 106.8 |
| both | matches | 100 | 322.8 | 429.1 |
| literal | no-match | 1000 | 1528.8 | 1791.1 |
| regex | no-match | 1000 | 564.1 | 750.9 |
| both | no-match | 1000 | 2319.9 | 3482.7 |
| literal | matches | 1000 | 3792.6 | 6128.1 |
| regex | matches | 1000 | 1179.8 | 2231.2 |
| both | matches | 1000 | 4311.7 | 5412.9 |

Notes
- Memory columns are the vector payload only (N x 384 x 4 or x 2 bytes); graph and process memory are not measured
- `exact scan` is a single-thread f32 scan, not the rayon `KnowledgeStore::linear_search`
- Keyword rules are matched rule by rule (no shared automaton); only conflict items use Aho-Corasick, and that path is not measured here
