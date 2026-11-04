# Engine benchmark

- crate version: 0.3.7
- logical CPUs: 8
- dimension: 384, top-k: 10, queries per row: 1000, seed: 0x5eed20261102
- HNSW parameters for this run: m=16 m0=32 efC=200 efS=64 heur=false

## Vector search (HNSW, single thread)

| shape | N | params | build s | recall@10 | p50 us | p95 us | p99 us | max us | QPS | exact scan p50 us | f32 MiB | f16 MiB |
| :--- | ---: | :--- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| uniform | 1000 | m=16 m0=32 efC=200 efS=64 heur=false | 0.43 | 0.9691 | 267.9 | 459.8 | 602.0 | 1144.7 | 3413 | 126.3 | 1.5 | 0.7 |
| uniform | 10000 | m=16 m0=32 efC=200 efS=64 heur=false | 16.36 | 0.4742 | 1129.0 | 1703.1 | 2155.6 | 4092.5 | 847 | 1744.7 | 14.6 | 7.3 |
| clustered | 1000 | m=16 m0=32 efC=200 efS=64 heur=false | 0.45 | 0.9998 | 203.3 | 300.7 | 401.9 | 519.4 | 4753 | 132.8 | 1.5 | 0.7 |
| clustered | 10000 | m=16 m0=32 efC=200 efS=64 heur=false | 3.29 | 0.6858 | 168.3 | 226.0 | 273.6 | 425.9 | 5821 | 1755.7 | 14.6 | 7.3 |

## Validator, keyword rules on a 10,000-character document

| mode | text | rules | p50 us | p95 us |
| :--- | :--- | ---: | ---: | ---: |
| literal | no-match | 10 | 34.8 | 51.1 |
| regex | no-match | 10 | 27.0 | 38.0 |
| both | no-match | 10 | 40.0 | 47.2 |
| literal | matches | 10 | 34.6 | 37.7 |
| regex | matches | 10 | 28.8 | 37.7 |
| both | matches | 10 | 39.1 | 41.3 |
| literal | no-match | 100 | 157.5 | 202.4 |
| regex | no-match | 100 | 72.0 | 78.9 |
| both | no-match | 100 | 208.4 | 243.4 |
| literal | matches | 100 | 228.8 | 273.6 |
| regex | matches | 100 | 92.0 | 114.7 |
| both | matches | 100 | 299.3 | 379.5 |
| literal | no-match | 1000 | 1447.0 | 1680.5 |
| regex | no-match | 1000 | 538.1 | 706.5 |
| both | no-match | 1000 | 1926.1 | 2325.8 |
| literal | matches | 1000 | 1954.3 | 2228.8 |
| regex | matches | 1000 | 625.9 | 853.9 |
| both | matches | 1000 | 2594.3 | 2924.1 |

Notes
- Memory columns are the vector payload only (N x 384 x 4 or x 2 bytes); graph and process memory are not measured
- `exact scan` is a single-thread f32 scan, not the rayon `KnowledgeStore::linear_search`
- Keyword rules are matched rule by rule (no shared automaton); only conflict items use Aho-Corasick, and that path is not measured here
