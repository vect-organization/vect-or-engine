# Engine benchmark

- crate version: 0.3.7
- logical CPUs: 8
- dimension: 768, top-k: 10, queries per row: 1000, seed: 0x5eed20261102
- HNSW parameters for this run: m=16 m0=32 efC=200 efS=64 heur=true keep=true

## Vector search (HNSW, single thread)

| shape | N | params | build s | recall@10 | p50 us | p95 us | p99 us | max us | QPS | exact scan p50 us | f32 MiB | f16 MiB |
| :--- | ---: | :--- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| uniform | 1000 | m=16 m0=32 efC=200 efS=64 heur=true keep=true | 5.68 | 0.9584 | 555.3 | 908.3 | 1365.1 | 20762.3 | 1589 | 408.8 | 2.9 | 1.5 |
| uniform | 10000 | m=16 m0=32 efC=200 efS=64 heur=true keep=true | 86.98 | 0.3854 | 1434.6 | 2254.0 | 2684.8 | 3842.0 | 651 | 3552.6 | 29.3 | 14.6 |
| clustered | 1000 | m=16 m0=32 efC=200 efS=64 heur=true keep=true | 2.08 | 0.9999 | 367.1 | 662.7 | 1000.0 | 2083.7 | 2452 | 316.8 | 2.9 | 1.5 |
| clustered | 10000 | m=16 m0=32 efC=200 efS=64 heur=true keep=true | 41.10 | 0.9992 | 858.7 | 1970.1 | 4833.0 | 35004.8 | 968 | 9912.7 | 29.3 | 14.6 |

## Validator, keyword rules on a 10,000-character document

| mode | text | rules | p50 us | p95 us |
| :--- | :--- | ---: | ---: | ---: |
| literal | no-match | 10 | 113.8 | 161.3 |
| regex | no-match | 10 | 59.6 | 89.6 |
| both | no-match | 10 | 147.0 | 304.0 |
| literal | matches | 10 | 119.4 | 173.3 |
| regex | matches | 10 | 66.7 | 89.9 |
| both | matches | 10 | 129.8 | 255.6 |
| literal | no-match | 100 | 713.1 | 1237.2 |
| regex | no-match | 100 | 250.8 | 526.9 |
| both | no-match | 100 | 602.5 | 1510.0 |
| literal | matches | 100 | 597.0 | 1252.3 |
| regex | matches | 100 | 165.4 | 328.6 |
| both | matches | 100 | 661.3 | 1367.0 |
| literal | no-match | 1000 | 3172.0 | 5558.8 |
| regex | no-match | 1000 | 874.5 | 2143.3 |
| both | no-match | 1000 | 4672.6 | 8303.6 |
| literal | matches | 1000 | 5006.6 | 8166.3 |
| regex | matches | 1000 | 1445.4 | 2727.4 |
| both | matches | 1000 | 5275.0 | 10065.0 |

Notes
- Memory columns are the vector payload only (N x 768 x 4 or x 2 bytes); graph and process memory are not measured
- `exact scan` is a single-thread f32 scan, not the rayon `KnowledgeStore::linear_search`
- Keyword rules are matched rule by rule (no shared automaton); only conflict items use Aho-Corasick, and that path is not measured here
