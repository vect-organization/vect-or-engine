# Engine benchmark

- crate version: 0.3.9
- logical CPUs: 8
- dimension: 384, top-k: 10, queries per row: 1000, seed: 0x5eed20261102
- HNSW parameters for this run: m=16 m0=32 efC=200 efS=64 heur=true keep=true

## Vector search (HNSW, single thread)

| shape | N | params | build s | recall@10 | p50 us | p95 us | p99 us | max us | QPS | exact scan p50 us | f32 MiB | f16 MiB |
| :--- | ---: | :--- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| uniform | 1000 | m=16 m0=32 efC=200 efS=64 heur=true keep=true | 1.44 | 0.9752 | 625.3 | 1308.3 | 1987.2 | 7185.8 | 1477 | 228.8 | 1.5 | 0.7 |
| uniform | 3000 | m=16 m0=32 efC=200 efS=64 heur=true keep=true | 7.76 | 0.7906 | 507.6 | 582.4 | 712.9 | 808.2 | 1938 | 370.2 | 4.4 | 2.2 |
| uniform | 10000 | m=16 m0=32 efC=200 efS=64 heur=true keep=true | 23.45 | 0.4833 | 958.3 | 1393.1 | 1737.9 | 3295.4 | 1009 | 1925.5 | 14.6 | 7.3 |
| clustered | 1000 | m=16 m0=32 efC=200 efS=64 heur=true keep=true | 1.00 | 1.0000 | 234.6 | 314.2 | 387.1 | 742.2 | 4146 | 116.1 | 1.5 | 0.7 |
| clustered | 3000 | m=16 m0=32 efC=200 efS=64 heur=true keep=true | 3.25 | 1.0000 | 235.4 | 313.6 | 393.5 | 479.3 | 4102 | 467.0 | 4.4 | 2.2 |
| clustered | 10000 | m=16 m0=32 efC=200 efS=64 heur=true keep=true | 11.44 | 1.0000 | 187.7 | 216.8 | 254.7 | 296.1 | 5253 | 1374.4 | 14.6 | 7.3 |

## Validator, keyword rules on a 10,000-character document

| mode | text | rules | p50 us | p95 us |
| :--- | :--- | ---: | ---: | ---: |
| literal | no-match | 10 | 24.7 | 25.0 |
| regex | no-match | 10 | 17.8 | 17.9 |
| both | no-match | 10 | 30.0 | 30.2 |
| literal | matches | 10 | 24.3 | 24.6 |
| regex | matches | 10 | 19.5 | 19.7 |
| both | matches | 10 | 28.3 | 28.5 |
| literal | no-match | 100 | 143.8 | 150.2 |
| regex | no-match | 100 | 67.2 | 71.5 |
| both | no-match | 100 | 201.4 | 206.5 |
| literal | matches | 100 | 218.0 | 306.0 |
| regex | matches | 100 | 91.0 | 99.0 |
| both | matches | 100 | 305.5 | 363.7 |
| literal | no-match | 1000 | 1460.5 | 1595.4 |
| regex | no-match | 1000 | 556.7 | 661.1 |
| both | no-match | 1000 | 1936.2 | 2133.6 |
| literal | matches | 1000 | 2224.7 | 2437.9 |
| regex | matches | 1000 | 699.7 | 839.2 |
| both | matches | 1000 | 2839.6 | 3389.6 |

Notes
- Memory columns are the vector payload only (N x 384 x 4 or x 2 bytes); graph and process memory are not measured
- `exact scan` is a single-thread f32 scan, not the rayon `KnowledgeStore::linear_search`
- Keyword rules are matched rule by rule (no shared automaton); only conflict items use Aho-Corasick, and that path is not measured here
