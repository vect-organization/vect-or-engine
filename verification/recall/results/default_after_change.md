# Engine benchmark

- crate version: 0.3.7
- logical CPUs: 8
- dimension: 384, top-k: 10, queries per row: 1000, seed: 0x5eed20261102
- HNSW parameters for this run: m=16 m0=32 efC=200 efS=64 heur=true keep=true

## Vector search (HNSW, single thread)

| shape | N | params | build s | recall@10 | p50 us | p95 us | p99 us | max us | QPS | exact scan p50 us | f32 MiB | f16 MiB |
| :--- | ---: | :--- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| uniform | 1000 | m=16 m0=32 efC=200 efS=64 heur=true keep=true | 5.25 | 0.9751 | 936.9 | 2634.6 | 7836.9 | 13040.0 | 793 | 257.8 | 1.5 | 0.7 |
| uniform | 10000 | m=16 m0=32 efC=200 efS=64 heur=true keep=true | 51.73 | 0.4786 | 810.5 | 1046.1 | 1261.5 | 1459.9 | 1194 | 1738.2 | 14.6 | 7.3 |
| clustered | 1000 | m=16 m0=32 efC=200 efS=64 heur=true keep=true | 1.06 | 0.9998 | 180.4 | 277.1 | 381.3 | 497.1 | 5169 | 117.0 | 1.5 | 0.7 |
| clustered | 10000 | m=16 m0=32 efC=200 efS=64 heur=true keep=true | 23.61 | 0.9990 | 247.2 | 308.2 | 389.0 | 565.3 | 3931 | 1681.3 | 14.6 | 7.3 |

## Validator, keyword rules on a 10,000-character document

| mode | text | rules | p50 us | p95 us |
| :--- | :--- | ---: | ---: | ---: |
| literal | no-match | 10 | 29.8 | 32.2 |
| regex | no-match | 10 | 18.9 | 35.1 |
| both | no-match | 10 | 35.0 | 36.6 |
| literal | matches | 10 | 28.5 | 29.1 |
| regex | matches | 10 | 18.8 | 18.9 |
| both | matches | 10 | 32.5 | 44.8 |
| literal | no-match | 100 | 199.0 | 226.1 |
| regex | no-match | 100 | 64.1 | 72.5 |
| both | no-match | 100 | 251.9 | 333.5 |
| literal | matches | 100 | 279.2 | 305.8 |
| regex | matches | 100 | 82.7 | 98.9 |
| both | matches | 100 | 351.8 | 526.5 |
| literal | no-match | 1000 | 1961.0 | 2372.9 |
| regex | no-match | 1000 | 756.9 | 1510.5 |
| both | no-match | 1000 | 2880.7 | 3725.8 |
| literal | matches | 1000 | 3348.8 | 5366.8 |
| regex | matches | 1000 | 979.4 | 1878.7 |
| both | matches | 1000 | 5974.2 | 9320.0 |

Notes
- Memory columns are the vector payload only (N x 384 x 4 or x 2 bytes); graph and process memory are not measured
- `exact scan` is a single-thread f32 scan, not the rayon `KnowledgeStore::linear_search`
- Keyword rules are matched rule by rule (no shared automaton); only conflict items use Aho-Corasick, and that path is not measured here
