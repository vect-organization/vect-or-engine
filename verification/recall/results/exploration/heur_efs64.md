# Engine benchmark

- crate version: 0.3.7
- logical CPUs: 8
- dimension: 384, top-k: 10, queries per row: 1000, seed: 0x5eed20261102
- HNSW parameters for this run: m=16 m0=16 efC=200 efS=64 heur=true

## Vector search (HNSW, single thread)

| shape | N | params | build s | recall@10 | p50 us | p95 us | p99 us | max us | QPS | exact scan p50 us | f32 MiB | f16 MiB |
| :--- | ---: | :--- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| uniform | 1000 | m=16 m0=16 efC=200 efS=64 heur=true | 0.96 | 0.8450 | 391.0 | 770.9 | 1173.1 | 2087.2 | 2181 | 218.4 | 1.5 | 0.7 |
| uniform | 10000 | m=16 m0=16 efC=200 efS=64 heur=true | 17.46 | 0.2711 | 623.8 | 868.5 | 1092.2 | 1971.1 | 1525 | 2394.9 | 14.6 | 7.3 |
| clustered | 1000 | m=16 m0=16 efC=200 efS=64 heur=true | 0.74 | 0.9998 | 245.5 | 405.8 | 536.9 | 1794.5 | 3753 | 173.9 | 1.5 | 0.7 |
| clustered | 10000 | m=16 m0=16 efC=200 efS=64 heur=true | 14.91 | 0.9964 | 215.2 | 278.4 | 334.6 | 541.1 | 4523 | 1629.5 | 14.6 | 7.3 |

## Validator, keyword rules on a 10,000-character document

| mode | text | rules | p50 us | p95 us |
| :--- | :--- | ---: | ---: | ---: |
| literal | no-match | 10 | 31.2 | 33.9 |
| regex | no-match | 10 | 24.2 | 24.3 |
| both | no-match | 10 | 35.9 | 39.3 |
| literal | matches | 10 | 31.2 | 43.7 |
| regex | matches | 10 | 25.6 | 25.7 |
| both | matches | 10 | 35.0 | 37.4 |
| literal | no-match | 100 | 146.5 | 156.7 |
| regex | no-match | 100 | 67.5 | 71.5 |
| both | no-match | 100 | 194.4 | 219.2 |
| literal | matches | 100 | 220.0 | 256.4 |
| regex | matches | 100 | 88.6 | 105.2 |
| both | matches | 100 | 288.7 | 344.9 |
| literal | no-match | 1000 | 1416.9 | 1597.5 |
| regex | no-match | 1000 | 524.0 | 675.6 |
| both | no-match | 1000 | 2038.5 | 3559.1 |
| literal | matches | 1000 | 2896.7 | 4458.3 |
| regex | matches | 1000 | 918.4 | 1603.4 |
| both | matches | 1000 | 3416.0 | 4265.3 |

Notes
- Memory columns are the vector payload only (N x 384 x 4 or x 2 bytes); graph and process memory are not measured
- `exact scan` is a single-thread f32 scan, not the rayon `KnowledgeStore::linear_search`
- Keyword rules are matched rule by rule (no shared automaton); only conflict items use Aho-Corasick, and that path is not measured here
