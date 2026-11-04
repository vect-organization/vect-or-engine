# Engine benchmark

- crate version: 0.3.7
- logical CPUs: 8
- dimension: 384, top-k: 10, queries per row: 1000, seed: 0x5eed20261102
- HNSW parameters for this run: m=16 m0=16 efC=200 efS=256 heur=true

## Vector search (HNSW, single thread)

| shape | N | params | build s | recall@10 | p50 us | p95 us | p99 us | max us | QPS | exact scan p50 us | f32 MiB | f16 MiB |
| :--- | ---: | :--- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| uniform | 1000 | m=16 m0=16 efC=200 efS=256 heur=true | 0.85 | 0.9977 | 588.6 | 867.0 | 1025.2 | 1288.2 | 1588 | 163.9 | 1.5 | 0.7 |
| uniform | 10000 | m=16 m0=16 efC=200 efS=256 heur=true | 14.86 | 0.6382 | 1678.9 | 2864.5 | 4171.8 | 9099.5 | 537 | 2100.5 | 14.6 | 7.3 |
| clustered | 1000 | m=16 m0=16 efC=200 efS=256 heur=true | 0.63 | 0.9998 | 405.0 | 627.1 | 892.2 | 1196.7 | 2312 | 126.6 | 1.5 | 0.7 |
| clustered | 10000 | m=16 m0=16 efC=200 efS=256 heur=true | 10.32 | 0.9988 | 775.3 | 1145.4 | 1403.3 | 2778.4 | 1249 | 1712.7 | 14.6 | 7.3 |

## Validator, keyword rules on a 10,000-character document

| mode | text | rules | p50 us | p95 us |
| :--- | :--- | ---: | ---: | ---: |
| literal | no-match | 10 | 33.5 | 44.4 |
| regex | no-match | 10 | 25.9 | 26.0 |
| both | no-match | 10 | 38.6 | 41.3 |
| literal | matches | 10 | 33.3 | 47.4 |
| regex | matches | 10 | 27.8 | 28.0 |
| both | matches | 10 | 37.7 | 38.1 |
| literal | no-match | 100 | 157.3 | 178.5 |
| regex | no-match | 100 | 72.0 | 93.8 |
| both | no-match | 100 | 208.6 | 253.8 |
| literal | matches | 100 | 223.8 | 254.3 |
| regex | matches | 100 | 88.4 | 114.5 |
| both | matches | 100 | 287.6 | 339.2 |
| literal | no-match | 1000 | 1426.6 | 1743.7 |
| regex | no-match | 1000 | 536.8 | 750.7 |
| both | no-match | 1000 | 1883.7 | 2359.2 |
| literal | matches | 1000 | 1950.6 | 2313.3 |
| regex | matches | 1000 | 626.3 | 888.1 |
| both | matches | 1000 | 2626.6 | 3078.4 |

Notes
- Memory columns are the vector payload only (N x 384 x 4 or x 2 bytes); graph and process memory are not measured
- `exact scan` is a single-thread f32 scan, not the rayon `KnowledgeStore::linear_search`
- Keyword rules are matched rule by rule (no shared automaton); only conflict items use Aho-Corasick, and that path is not measured here
