# Engine benchmark

- crate version: 0.3.10
- logical CPUs: 8
- dimension: 384, top-k: 10, queries per row: 500, seed: 0x5eed20261102
- HNSW parameters for this run: m=16 m0=32 efC=200 efS=64 heur=true keep=true

## Vector search (HNSW, single thread)

| shape | N | params | build s | recall@10 | p50 us | p95 us | p99 us | max us | QPS | exact scan p50 us | f32 MiB | f16 MiB |
| :--- | ---: | :--- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| uniform | 1000 | m=16 m0=32 efC=200 efS=64 heur=true keep=true | 1.47 | 0.9734 | 429.9 | 1302.3 | 2606.3 | 4098.1 | 1916 | 303.5 | 1.5 | 0.7 |
| uniform | 5000 | m=16 m0=32 efC=200 efS=64 heur=true keep=true | 10.15 | 0.6614 | 481.7 | 561.5 | 721.7 | 821.9 | 2046 | 851.8 | 7.3 | 3.7 |
| uniform | 10000 | m=16 m0=32 efC=200 efS=64 heur=true keep=true | 22.76 | 0.4910 | 682.8 | 974.4 | 1408.9 | 3225.5 | 1367 | 1968.4 | 14.6 | 7.3 |
| uniform | 20000 | m=16 m0=32 efC=200 efS=64 heur=true keep=true | 49.94 | 0.3112 | 505.6 | 628.9 | 734.1 | 942.0 | 1929 | 2640.0 | 29.3 | 14.6 |
| uniform | 50000 | m=16 m0=32 efC=200 efS=64 heur=true keep=true | 136.96 | 0.1796 | 605.1 | 765.4 | 967.4 | 1107.3 | 1620 | 6582.8 | 73.2 | 36.6 |
| clustered | 1000 | m=16 m0=32 efC=200 efS=64 heur=true keep=true | 0.68 | 1.0000 | 101.5 | 120.5 | 135.6 | 149.0 | 9767 | 99.5 | 1.5 | 0.7 |
| clustered | 5000 | m=16 m0=32 efC=200 efS=64 heur=true keep=true | 5.15 | 1.0000 | 152.4 | 175.7 | 217.0 | 388.0 | 6441 | 733.3 | 7.3 | 3.7 |
| clustered | 10000 | m=16 m0=32 efC=200 efS=64 heur=true keep=true | 11.27 | 1.0000 | 185.7 | 230.2 | 300.2 | 354.2 | 5251 | 1683.4 | 14.6 | 7.3 |
| clustered | 20000 | m=16 m0=32 efC=200 efS=64 heur=true keep=true | 24.26 | 0.9990 | 220.0 | 254.9 | 297.6 | 357.8 | 4482 | 3403.2 | 29.3 | 14.6 |
| clustered | 50000 | m=16 m0=32 efC=200 efS=64 heur=true keep=true | 78.67 | 0.9774 | 282.4 | 318.4 | 353.3 | 534.1 | 3498 | 6543.9 | 73.2 | 36.6 |

## Validator, keyword rules on a 10,000-character document

| mode | text | rules | p50 us | p95 us |
| :--- | :--- | ---: | ---: | ---: |
| literal | no-match | 10 | 21.4 | 21.6 |
| regex | no-match | 10 | 16.3 | 16.3 |
| both | no-match | 10 | 25.9 | 26.1 |
| literal | matches | 10 | 21.6 | 21.9 |
| regex | matches | 10 | 17.8 | 17.9 |
| both | matches | 10 | 25.3 | 25.5 |
| literal | no-match | 100 | 115.2 | 121.4 |
| regex | no-match | 100 | 58.7 | 62.7 |
| both | no-match | 100 | 161.8 | 193.8 |
| literal | matches | 100 | 170.1 | 182.2 |
| regex | matches | 100 | 75.7 | 77.7 |
| both | matches | 100 | 232.5 | 237.3 |
| literal | no-match | 1000 | 1071.9 | 1089.0 |
| regex | no-match | 1000 | 481.6 | 487.1 |
| both | no-match | 1000 | 1551.1 | 1670.3 |
| literal | matches | 1000 | 1505.8 | 1557.7 |
| regex | matches | 1000 | 579.5 | 651.0 |
| both | matches | 1000 | 2149.2 | 2312.6 |

Notes
- Memory columns are the vector payload only (N x 384 x 4 or x 2 bytes); graph and process memory are not measured
- `exact scan` is a single-thread f32 scan, not the rayon `KnowledgeStore::linear_search`
- Keyword rules are matched rule by rule (no shared automaton); only conflict items use Aho-Corasick, and that path is not measured here
