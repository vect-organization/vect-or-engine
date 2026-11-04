# Engine benchmark

- crate version: 0.3.7
- logical CPUs: 8
- dimension: 384, top-k: 10, queries per row: 1000, seed: 0x5eed20261102
- HNSW parameters for this run: m=16 m0=16 efC=200 efS=128 heur=false

## Vector search (HNSW, single thread)

| shape | N | params | build s | recall@10 | p50 us | p95 us | p99 us | max us | QPS | exact scan p50 us | f32 MiB | f16 MiB |
| :--- | ---: | :--- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| uniform | 1000 | m=16 m0=16 efC=200 efS=128 heur=false | 0.27 | 0.9637 | 329.0 | 624.0 | 1013.9 | 1872.4 | 2700 | 143.7 | 1.5 | 0.7 |
| uniform | 10000 | m=16 m0=16 efC=200 efS=128 heur=false | 11.21 | 0.4272 | 1317.7 | 2111.8 | 2794.5 | 5559.0 | 712 | 2290.7 | 14.6 | 7.3 |
| clustered | 1000 | m=16 m0=16 efC=200 efS=128 heur=false | 0.23 | 0.6349 | 31.6 | 190.0 | 305.2 | 412.9 | 16743 | 136.6 | 1.5 | 0.7 |
| clustered | 10000 | m=16 m0=16 efC=200 efS=128 heur=false | 1.58 | 0.6584 | 213.9 | 345.9 | 509.7 | 1670.9 | 4287 | 2547.6 | 14.6 | 7.3 |

## Validator, keyword rules on a 10,000-character document

| mode | text | rules | p50 us | p95 us |
| :--- | :--- | ---: | ---: | ---: |
| literal | no-match | 10 | 48.5 | 102.9 |
| regex | no-match | 10 | 64.0 | 88.7 |
| both | no-match | 10 | 57.3 | 144.1 |
| literal | matches | 10 | 71.9 | 189.0 |
| regex | matches | 10 | 44.2 | 119.6 |
| both | matches | 10 | 96.6 | 203.5 |
| literal | no-match | 100 | 277.9 | 525.2 |
| regex | no-match | 100 | 119.6 | 253.6 |
| both | no-match | 100 | 409.9 | 1362.4 |
| literal | matches | 100 | 903.6 | 5389.9 |
| regex | matches | 100 | 510.2 | 3945.9 |
| both | matches | 100 | 1754.1 | 14477.9 |
| literal | no-match | 1000 | 9358.0 | 46559.8 |
| regex | no-match | 1000 | 2637.6 | 10983.5 |
| both | no-match | 1000 | 7483.8 | 18751.6 |
| literal | matches | 1000 | 6881.7 | 23576.0 |
| regex | matches | 1000 | 4779.3 | 11524.7 |
| both | matches | 1000 | 7186.4 | 17185.0 |

Notes
- Memory columns are the vector payload only (N x 384 x 4 or x 2 bytes); graph and process memory are not measured
- `exact scan` is a single-thread f32 scan, not the rayon `KnowledgeStore::linear_search`
- Keyword rules are matched rule by rule (no shared automaton); only conflict items use Aho-Corasick, and that path is not measured here
