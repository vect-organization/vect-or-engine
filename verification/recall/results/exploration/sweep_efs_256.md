# Engine benchmark

- crate version: 0.3.7
- logical CPUs: 8
- dimension: 384, top-k: 10, queries per row: 1000, seed: 0x5eed20261102
- HNSW parameters for this run: m=16 m0=16 efC=200 efS=256 heur=false

## Vector search (HNSW, single thread)

| shape | N | params | build s | recall@10 | p50 us | p95 us | p99 us | max us | QPS | exact scan p50 us | f32 MiB | f16 MiB |
| :--- | ---: | :--- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| uniform | 1000 | m=16 m0=16 efC=200 efS=256 heur=false | 0.78 | 0.9973 | 759.4 | 1395.3 | 2224.6 | 8785.3 | 1180 | 228.0 | 1.5 | 0.7 |
| uniform | 10000 | m=16 m0=16 efC=200 efS=256 heur=false | 10.48 | 0.6265 | 1757.7 | 2818.8 | 3539.5 | 6848.4 | 524 | 1958.5 | 14.6 | 7.3 |
| clustered | 1000 | m=16 m0=16 efC=200 efS=256 heur=false | 0.30 | 0.6439 | 46.0 | 473.6 | 706.8 | 1207.8 | 8158 | 179.5 | 1.5 | 0.7 |
| clustered | 10000 | m=16 m0=16 efC=200 efS=256 heur=false | 2.20 | 0.6588 | 237.9 | 361.7 | 452.9 | 1419.8 | 4031 | 2711.1 | 14.6 | 7.3 |

## Validator, keyword rules on a 10,000-character document

| mode | text | rules | p50 us | p95 us |
| :--- | :--- | ---: | ---: | ---: |
| literal | no-match | 10 | 48.1 | 74.8 |
| regex | no-match | 10 | 51.5 | 92.3 |
| both | no-match | 10 | 55.8 | 106.3 |
| literal | matches | 10 | 48.6 | 119.9 |
| regex | matches | 10 | 39.7 | 64.3 |
| both | matches | 10 | 54.2 | 74.0 |
| literal | no-match | 100 | 274.2 | 403.0 |
| regex | no-match | 100 | 108.4 | 203.7 |
| both | no-match | 100 | 324.9 | 720.2 |
| literal | matches | 100 | 344.4 | 574.9 |
| regex | matches | 100 | 138.2 | 321.1 |
| both | matches | 100 | 433.1 | 778.8 |
| literal | no-match | 1000 | 2841.5 | 5383.5 |
| regex | no-match | 1000 | 1057.1 | 2626.9 |
| both | no-match | 1000 | 7873.5 | 21391.2 |
| literal | matches | 1000 | 6444.1 | 11895.6 |
| regex | matches | 1000 | 2412.2 | 7314.4 |
| both | matches | 1000 | 7417.4 | 14810.1 |

Notes
- Memory columns are the vector payload only (N x 384 x 4 or x 2 bytes); graph and process memory are not measured
- `exact scan` is a single-thread f32 scan, not the rayon `KnowledgeStore::linear_search`
- Keyword rules are matched rule by rule (no shared automaton); only conflict items use Aho-Corasick, and that path is not measured here
