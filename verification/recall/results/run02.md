# Engine benchmark

- crate version: 0.3.7
- logical CPUs: 8
- dimension: 384, top-k: 10, queries per row: 1000, seed: 0x5eed20261102
- HNSW parameters for this run: m=16 m0=32 efC=200 efS=64 heur=true keep=true

## Vector search (HNSW, single thread)

| shape | N | params | build s | recall@10 | p50 us | p95 us | p99 us | max us | QPS | exact scan p50 us | f32 MiB | f16 MiB |
| :--- | ---: | :--- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| uniform | 1000 | m=16 m0=32 efC=200 efS=64 heur=true keep=true | 1.93 | 0.9751 | 285.3 | 457.2 | 583.5 | 1061.7 | 3207 | 129.2 | 1.5 | 0.7 |
| uniform | 10000 | m=16 m0=32 efC=200 efS=64 heur=true keep=true | 43.62 | 0.4786 | 1006.8 | 1613.1 | 2087.4 | 2990.3 | 920 | 1883.5 | 14.6 | 7.3 |
| clustered | 1000 | m=16 m0=32 efC=200 efS=64 heur=true keep=true | 1.27 | 0.9998 | 261.7 | 564.2 | 927.0 | 2326.3 | 3189 | 158.1 | 1.5 | 0.7 |
| clustered | 10000 | m=16 m0=32 efC=200 efS=64 heur=true keep=true | 22.00 | 0.9990 | 280.4 | 440.9 | 570.9 | 1690.7 | 3321 | 1981.6 | 14.6 | 7.3 |

## Validator, keyword rules on a 10,000-character document

| mode | text | rules | p50 us | p95 us |
| :--- | :--- | ---: | ---: | ---: |
| literal | no-match | 10 | 35.2 | 53.1 |
| regex | no-match | 10 | 20.6 | 21.0 |
| both | no-match | 10 | 41.2 | 77.5 |
| literal | matches | 10 | 33.7 | 54.6 |
| regex | matches | 10 | 22.3 | 22.8 |
| both | matches | 10 | 38.4 | 65.9 |
| literal | no-match | 100 | 226.7 | 385.9 |
| regex | no-match | 100 | 72.6 | 98.5 |
| both | no-match | 100 | 282.1 | 487.6 |
| literal | matches | 100 | 314.7 | 507.4 |
| regex | matches | 100 | 93.6 | 150.2 |
| both | matches | 100 | 397.7 | 634.9 |
| literal | no-match | 1000 | 2401.7 | 4233.8 |
| regex | no-match | 1000 | 757.9 | 1683.0 |
| both | no-match | 1000 | 3537.2 | 5549.8 |
| literal | matches | 1000 | 3103.5 | 4296.2 |
| regex | matches | 1000 | 789.2 | 1564.0 |
| both | matches | 1000 | 4541.2 | 6788.7 |

Notes
- Memory columns are the vector payload only (N x 384 x 4 or x 2 bytes); graph and process memory are not measured
- `exact scan` is a single-thread f32 scan, not the rayon `KnowledgeStore::linear_search`
- Keyword rules are matched rule by rule (no shared automaton); only conflict items use Aho-Corasick, and that path is not measured here
