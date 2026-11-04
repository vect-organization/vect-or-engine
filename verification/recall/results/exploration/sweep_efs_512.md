# Engine benchmark

- crate version: 0.3.7
- logical CPUs: 8
- dimension: 384, top-k: 10, queries per row: 1000, seed: 0x5eed20261102
- HNSW parameters for this run: m=16 m0=16 efC=200 efS=512 heur=false

## Vector search (HNSW, single thread)

| shape | N | params | build s | recall@10 | p50 us | p95 us | p99 us | max us | QPS | exact scan p50 us | f32 MiB | f16 MiB |
| :--- | ---: | :--- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| uniform | 1000 | m=16 m0=16 efC=200 efS=512 heur=false | 1.01 | 0.9997 | 1166.0 | 2242.1 | 3163.4 | 11985.0 | 768 | 287.9 | 1.5 | 0.7 |
| uniform | 10000 | m=16 m0=16 efC=200 efS=512 heur=false | 11.49 | 0.8289 | 2489.9 | 3384.8 | 3916.2 | 5039.7 | 386 | 1757.3 | 14.6 | 7.3 |
| clustered | 1000 | m=16 m0=16 efC=200 efS=512 heur=false | 0.18 | 0.6455 | 27.3 | 452.7 | 536.4 | 819.1 | 9737 | 121.9 | 1.5 | 0.7 |
| clustered | 10000 | m=16 m0=16 efC=200 efS=512 heur=false | 2.02 | 0.6588 | 307.5 | 480.0 | 754.2 | 2626.4 | 3054 | 3091.1 | 14.6 | 7.3 |

## Validator, keyword rules on a 10,000-character document

| mode | text | rules | p50 us | p95 us |
| :--- | :--- | ---: | ---: | ---: |
| literal | no-match | 10 | 57.6 | 108.2 |
| regex | no-match | 10 | 44.1 | 75.4 |
| both | no-match | 10 | 67.9 | 132.8 |
| literal | matches | 10 | 56.6 | 119.9 |
| regex | matches | 10 | 47.4 | 77.9 |
| both | matches | 10 | 64.9 | 132.8 |
| literal | no-match | 100 | 253.5 | 359.4 |
| regex | no-match | 100 | 116.1 | 190.8 |
| both | no-match | 100 | 314.0 | 473.1 |
| literal | matches | 100 | 338.1 | 625.3 |
| regex | matches | 100 | 133.1 | 224.7 |
| both | matches | 100 | 410.5 | 698.3 |
| literal | no-match | 1000 | 3126.2 | 8798.7 |
| regex | no-match | 1000 | 1473.7 | 3649.0 |
| both | no-match | 1000 | 5752.7 | 11518.3 |
| literal | matches | 1000 | 10753.2 | 36209.9 |
| regex | matches | 1000 | 3795.4 | 11754.0 |
| both | matches | 1000 | 8216.5 | 31610.6 |

Notes
- Memory columns are the vector payload only (N x 384 x 4 or x 2 bytes); graph and process memory are not measured
- `exact scan` is a single-thread f32 scan, not the rayon `KnowledgeStore::linear_search`
- Keyword rules are matched rule by rule (no shared automaton); only conflict items use Aho-Corasick, and that path is not measured here
