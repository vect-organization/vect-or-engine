# Engine benchmark

- crate version: 0.3.7
- logical CPUs: 8
- dimension: 768, top-k: 10, queries per row: 1000, seed: 0x5eed20261102
- HNSW parameters for this run: m=16 m0=32 efC=200 efS=64 heur=true keep=true

## Vector search (HNSW, single thread)

| shape | N | params | build s | recall@10 | p50 us | p95 us | p99 us | max us | QPS | exact scan p50 us | f32 MiB | f16 MiB |
| :--- | ---: | :--- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| uniform | 1000 | m=16 m0=32 efC=200 efS=64 heur=true keep=true | 3.73 | 0.9584 | 576.3 | 1050.8 | 1413.6 | 4352.4 | 1548 | 365.5 | 2.9 | 1.5 |
| uniform | 10000 | m=16 m0=32 efC=200 efS=64 heur=true keep=true | 94.16 | 0.3854 | 1771.6 | 2736.3 | 3593.4 | 7417.0 | 526 | 4071.1 | 29.3 | 14.6 |
| clustered | 1000 | m=16 m0=32 efC=200 efS=64 heur=true keep=true | 2.76 | 0.9999 | 490.1 | 879.3 | 1247.1 | 3740.2 | 1818 | 523.4 | 2.9 | 1.5 |
| clustered | 10000 | m=16 m0=32 efC=200 efS=64 heur=true keep=true | 36.69 | 0.9992 | 375.0 | 488.7 | 603.3 | 765.1 | 2590 | 3660.2 | 29.3 | 14.6 |

## Validator, keyword rules on a 10,000-character document

| mode | text | rules | p50 us | p95 us |
| :--- | :--- | ---: | ---: | ---: |
| literal | no-match | 10 | 31.6 | 41.9 |
| regex | no-match | 10 | 18.4 | 18.6 |
| both | no-match | 10 | 37.1 | 55.4 |
| literal | matches | 10 | 28.9 | 29.8 |
| regex | matches | 10 | 20.0 | 27.2 |
| both | matches | 10 | 33.1 | 54.7 |
| literal | no-match | 100 | 211.8 | 361.8 |
| regex | no-match | 100 | 67.2 | 75.6 |
| both | no-match | 100 | 267.5 | 349.0 |
| literal | matches | 100 | 293.2 | 465.6 |
| regex | matches | 100 | 86.5 | 103.6 |
| both | matches | 100 | 366.4 | 482.7 |
| literal | no-match | 1000 | 2439.7 | 3844.9 |
| regex | no-match | 1000 | 940.2 | 1799.4 |
| both | no-match | 1000 | 4463.5 | 6798.0 |
| literal | matches | 1000 | 4884.4 | 9504.3 |
| regex | matches | 1000 | 894.7 | 1906.6 |
| both | matches | 1000 | 4310.0 | 6165.8 |

Notes
- Memory columns are the vector payload only (N x 768 x 4 or x 2 bytes); graph and process memory are not measured
- `exact scan` is a single-thread f32 scan, not the rayon `KnowledgeStore::linear_search`
- Keyword rules are matched rule by rule (no shared automaton); only conflict items use Aho-Corasick, and that path is not measured here
