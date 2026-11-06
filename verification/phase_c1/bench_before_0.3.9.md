# Engine benchmark

- crate version: 0.3.9
- logical CPUs: 8
- dimension: 384, top-k: 10, queries per row: 1000, seed: 0x5eed20261102
- HNSW parameters for this run: m=16 m0=32 efC=200 efS=64 heur=true keep=true

## Vector search (HNSW, single thread)

| shape | N | params | build s | recall@10 | p50 us | p95 us | p99 us | max us | QPS | exact scan p50 us | f32 MiB | f16 MiB |
| :--- | ---: | :--- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| uniform | 1000 | m=16 m0=32 efC=200 efS=64 heur=true keep=true | 2.83 | 0.9751 | 358.9 | 684.3 | 988.0 | 1783.6 | 2418 | 122.2 | 1.5 | 0.7 |
| uniform | 3000 | m=16 m0=32 efC=200 efS=64 heur=true keep=true | 9.29 | 0.7908 | 819.5 | 1173.0 | 1373.0 | 2127.5 | 1144 | 567.1 | 4.4 | 2.2 |
| uniform | 10000 | m=16 m0=32 efC=200 efS=64 heur=true keep=true | 33.67 | 0.4833 | 969.4 | 1376.0 | 1680.0 | 2772.6 | 981 | 1732.7 | 14.6 | 7.3 |
| clustered | 1000 | m=16 m0=32 efC=200 efS=64 heur=true keep=true | 0.93 | 0.9996 | 199.5 | 317.5 | 407.9 | 990.6 | 4694 | 105.4 | 1.5 | 0.7 |
| clustered | 3000 | m=16 m0=32 efC=200 efS=64 heur=true keep=true | 3.41 | 0.9993 | 231.7 | 325.5 | 398.4 | 487.3 | 4173 | 326.2 | 4.4 | 2.2 |
| clustered | 10000 | m=16 m0=32 efC=200 efS=64 heur=true keep=true | 15.40 | 0.9981 | 347.5 | 472.3 | 612.6 | 1170.6 | 2770 | 2355.6 | 14.6 | 7.3 |

## Validator, keyword rules on a 10,000-character document

| mode | text | rules | p50 us | p95 us |
| :--- | :--- | ---: | ---: | ---: |
| literal | no-match | 10 | 137.4 | 148.8 |
| regex | no-match | 10 | 47.8 | 100.7 |
| both | no-match | 10 | 76.2 | 106.4 |
| literal | matches | 10 | 65.3 | 71.2 |
| regex | matches | 10 | 50.8 | 94.0 |
| both | matches | 10 | 71.9 | 82.9 |
| literal | no-match | 100 | 326.7 | 622.1 |
| regex | no-match | 100 | 120.8 | 228.7 |
| both | no-match | 100 | 406.7 | 623.2 |
| literal | matches | 100 | 455.3 | 857.3 |
| regex | matches | 100 | 147.0 | 261.3 |
| both | matches | 100 | 562.5 | 949.1 |
| literal | no-match | 1000 | 4328.3 | 6568.9 |
| regex | no-match | 1000 | 1167.5 | 2291.9 |
| both | no-match | 1000 | 4343.7 | 6454.4 |
| literal | matches | 1000 | 4362.1 | 7048.6 |
| regex | matches | 1000 | 1530.7 | 2673.7 |
| both | matches | 1000 | 5551.3 | 10616.4 |

Notes
- Memory columns are the vector payload only (N x 384 x 4 or x 2 bytes); graph and process memory are not measured
- `exact scan` is a single-thread f32 scan, not the rayon `KnowledgeStore::linear_search`
- Keyword rules are matched rule by rule (no shared automaton); only conflict items use Aho-Corasick, and that path is not measured here
