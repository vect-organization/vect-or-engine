# Engine benchmark

- crate version: 0.3.7
- logical CPUs: 8
- dimension: 384, top-k: 10, queries per row: 1000, seed: 0x5eed20261102
- HNSW parameters for this run: m=16 m0=16 efC=200 efS=64 heur=true keep=false

## Vector search (HNSW, single thread)

| shape | N | params | build s | recall@10 | p50 us | p95 us | p99 us | max us | QPS | exact scan p50 us | f32 MiB | f16 MiB |
| :--- | ---: | :--- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| uniform | 1000 | m=16 m0=16 efC=200 efS=64 heur=true keep=false | 0.61 | 0.8448 | 249.3 | 468.0 | 759.4 | 1170.6 | 3542 | 145.7 | 1.5 | 0.7 |
| uniform | 10000 | m=16 m0=16 efC=200 efS=64 heur=true keep=false | 13.50 | 0.2706 | 859.1 | 1927.5 | 3725.2 | 21439.4 | 981 | 4764.9 | 14.6 | 7.3 |
| clustered | 1000 | m=16 m0=16 efC=200 efS=64 heur=true keep=false | 0.93 | 0.9998 | 350.5 | 685.3 | 1308.6 | 4678.7 | 2387 | 256.9 | 1.5 | 0.7 |
| clustered | 10000 | m=16 m0=16 efC=200 efS=64 heur=true keep=false | 8.04 | 0.9949 | 242.0 | 391.0 | 632.7 | 1084.2 | 3776 | 2581.7 | 14.6 | 7.3 |

## Validator, keyword rules on a 10,000-character document

| mode | text | rules | p50 us | p95 us |
| :--- | :--- | ---: | ---: | ---: |
| literal | no-match | 10 | 62.2 | 100.0 |
| regex | no-match | 10 | 38.5 | 60.0 |
| both | no-match | 10 | 50.8 | 90.3 |
| literal | matches | 10 | 41.4 | 75.1 |
| regex | matches | 10 | 29.0 | 63.3 |
| both | matches | 10 | 59.7 | 124.2 |
| literal | no-match | 100 | 299.9 | 550.9 |
| regex | no-match | 100 | 92.8 | 136.2 |
| both | no-match | 100 | 356.9 | 602.7 |
| literal | matches | 100 | 379.8 | 654.3 |
| regex | matches | 100 | 107.5 | 207.2 |
| both | matches | 100 | 446.8 | 721.1 |
| literal | no-match | 1000 | 2232.5 | 3150.1 |
| regex | no-match | 1000 | 621.0 | 1092.1 |
| both | no-match | 1000 | 3359.1 | 5251.6 |
| literal | matches | 1000 | 6380.8 | 10398.4 |
| regex | matches | 1000 | 2323.0 | 4342.5 |
| both | matches | 1000 | 8053.5 | 11531.1 |

Notes
- Memory columns are the vector payload only (N x 384 x 4 or x 2 bytes); graph and process memory are not measured
- `exact scan` is a single-thread f32 scan, not the rayon `KnowledgeStore::linear_search`
- Keyword rules are matched rule by rule (no shared automaton); only conflict items use Aho-Corasick, and that path is not measured here
