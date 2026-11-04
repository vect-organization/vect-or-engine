# Engine benchmark

- crate version: 0.3.7
- logical CPUs: 8
- dimension: 384, top-k: 10, queries per row: 1000, seed: 0x5eed20261102
- HNSW parameters for this run: m=16 m0=32 efC=100 efS=64 heur=true keep=true

## Vector search (HNSW, single thread)

| shape | N | params | build s | recall@10 | p50 us | p95 us | p99 us | max us | QPS | exact scan p50 us | f32 MiB | f16 MiB |
| :--- | ---: | :--- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| uniform | 1000 | m=16 m0=32 efC=100 efS=64 heur=true keep=true | 1.78 | 0.9749 | 272.5 | 519.2 | 749.6 | 1384.0 | 3178 | 127.5 | 1.5 | 0.7 |
| uniform | 10000 | m=16 m0=32 efC=100 efS=64 heur=true keep=true | 35.87 | 0.4700 | 1091.0 | 1706.0 | 2117.3 | 2810.9 | 869 | 1944.8 | 14.6 | 7.3 |
| clustered | 1000 | m=16 m0=32 efC=100 efS=64 heur=true keep=true | 1.04 | 0.9998 | 191.6 | 309.0 | 523.7 | 875.1 | 4747 | 120.9 | 1.5 | 0.7 |
| clustered | 10000 | m=16 m0=32 efC=100 efS=64 heur=true keep=true | 15.64 | 0.9990 | 240.8 | 370.0 | 496.9 | 14668.6 | 3671 | 1812.7 | 14.6 | 7.3 |

## Validator, keyword rules on a 10,000-character document

| mode | text | rules | p50 us | p95 us |
| :--- | :--- | ---: | ---: | ---: |
| literal | no-match | 10 | 32.3 | 54.5 |
| regex | no-match | 10 | 19.0 | 20.7 |
| both | no-match | 10 | 38.0 | 57.4 |
| literal | matches | 10 | 31.5 | 52.5 |
| regex | matches | 10 | 21.0 | 22.5 |
| both | matches | 10 | 35.7 | 77.3 |
| literal | no-match | 100 | 215.0 | 364.1 |
| regex | no-match | 100 | 69.7 | 102.1 |
| both | no-match | 100 | 271.9 | 451.7 |
| literal | matches | 100 | 303.2 | 402.2 |
| regex | matches | 100 | 89.7 | 116.9 |
| both | matches | 100 | 363.1 | 526.7 |
| literal | no-match | 1000 | 3163.3 | 5912.4 |
| regex | no-match | 1000 | 1325.4 | 3055.0 |
| both | no-match | 1000 | 4252.5 | 9615.9 |
| literal | matches | 1000 | 2881.3 | 3697.7 |
| regex | matches | 1000 | 671.0 | 1075.0 |
| both | matches | 1000 | 3501.4 | 4302.7 |

Notes
- Memory columns are the vector payload only (N x 384 x 4 or x 2 bytes); graph and process memory are not measured
- `exact scan` is a single-thread f32 scan, not the rayon `KnowledgeStore::linear_search`
- Keyword rules are matched rule by rule (no shared automaton); only conflict items use Aho-Corasick, and that path is not measured here
