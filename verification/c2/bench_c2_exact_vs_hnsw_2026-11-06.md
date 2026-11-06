# Engine benchmark

- crate version: 0.3.10
- logical CPUs: 8
- dimension: 384, top-k: 10, queries per row: 500, seed: 0x5eed20261102
- HNSW parameters for this run: m=16 m0=32 efC=200 efS=64 heur=true keep=true

## Vector search (HNSW, single thread)

| shape | N | params | build s | recall@10 | p50 us | p95 us | p99 us | max us | QPS | exact scan p50 us | f32 MiB | f16 MiB |
| :--- | ---: | :--- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| uniform | 1000 | m=16 m0=32 efC=200 efS=64 heur=true keep=true | 1.60 | 0.9734 | 257.8 | 350.4 | 403.0 | 458.7 | 3779 | 122.3 | 1.5 | 0.7 |
| uniform | 5000 | m=16 m0=32 efC=200 efS=64 heur=true keep=true | 11.82 | 0.6608 | 615.8 | 757.8 | 866.1 | 1035.9 | 1576 | 865.8 | 7.3 | 3.7 |
| uniform | 10000 | m=16 m0=32 efC=200 efS=64 heur=true keep=true | 38.85 | 0.4896 | 868.4 | 1205.7 | 1562.9 | 1715.6 | 1117 | 1908.9 | 14.6 | 7.3 |
| uniform | 20000 | m=16 m0=32 efC=200 efS=64 heur=true keep=true | 77.36 | 0.3204 | 903.2 | 1289.2 | 1397.7 | 1534.0 | 1058 | 3521.3 | 29.3 | 14.6 |
| uniform | 50000 | m=16 m0=32 efC=200 efS=64 heur=true keep=true | 205.44 | 0.1836 | 1012.6 | 1368.5 | 1607.6 | 1827.2 | 950 | 8538.7 | 73.2 | 36.6 |
| clustered | 1000 | m=16 m0=32 efC=200 efS=64 heur=true keep=true | 0.78 | 1.0000 | 178.8 | 199.5 | 241.3 | 257.9 | 5551 | 106.4 | 1.5 | 0.7 |
| clustered | 5000 | m=16 m0=32 efC=200 efS=64 heur=true keep=true | 6.37 | 1.0000 | 215.5 | 293.2 | 310.6 | 345.0 | 4405 | 797.3 | 7.3 | 3.7 |
| clustered | 10000 | m=16 m0=32 efC=200 efS=64 heur=true keep=true | 15.89 | 1.0000 | 284.9 | 352.9 | 411.9 | 469.7 | 3430 | 2007.5 | 14.6 | 7.3 |
| clustered | 20000 | m=16 m0=32 efC=200 efS=64 heur=true keep=true | 31.31 | 0.9990 | 297.6 | 336.7 | 403.9 | 562.8 | 3342 | 3403.8 | 29.3 | 14.6 |
| clustered | 50000 | m=16 m0=32 efC=200 efS=64 heur=true keep=true | 86.34 | 0.9774 | 423.1 | 576.3 | 667.5 | 788.9 | 2243 | 7826.6 | 73.2 | 36.6 |

## Validator, keyword rules on a 10,000-character document

| mode | text | rules | p50 us | p95 us |
| :--- | :--- | ---: | ---: | ---: |
| literal | no-match | 10 | 32.7 | 32.9 |
| regex | no-match | 10 | 24.1 | 24.3 |
| both | no-match | 10 | 37.8 | 39.8 |
| literal | matches | 10 | 30.1 | 30.3 |
| regex | matches | 10 | 24.8 | 25.0 |
| both | matches | 10 | 35.6 | 35.8 |
| literal | no-match | 100 | 170.5 | 177.4 |
| regex | no-match | 100 | 79.6 | 84.1 |
| both | no-match | 100 | 225.9 | 242.7 |
| literal | matches | 100 | 221.7 | 231.2 |
| regex | matches | 100 | 98.6 | 104.2 |
| both | matches | 100 | 291.4 | 305.0 |
| literal | no-match | 1000 | 1283.1 | 1348.0 |
| regex | no-match | 1000 | 532.7 | 539.0 |
| both | no-match | 1000 | 1604.8 | 1656.9 |
| literal | matches | 1000 | 1538.1 | 1612.1 |
| regex | matches | 1000 | 689.4 | 1273.4 |
| both | matches | 1000 | 2357.4 | 2583.7 |

Notes
- Memory columns are the vector payload only (N x 384 x 4 or x 2 bytes); graph and process memory are not measured
- `exact scan` is a single-thread f32 scan, not the rayon `KnowledgeStore::linear_search`
- Keyword rules are matched rule by rule (no shared automaton); only conflict items use Aho-Corasick, and that path is not measured here
