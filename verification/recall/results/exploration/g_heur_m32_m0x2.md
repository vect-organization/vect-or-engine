# Engine benchmark

- crate version: 0.3.7
- logical CPUs: 8
- dimension: 384, top-k: 10, queries per row: 1000, seed: 0x5eed20261102
- HNSW parameters for this run: m=32 m0=64 efC=200 efS=64 heur=true keep=true

## Vector search (HNSW, single thread)

| shape | N | params | build s | recall@10 | p50 us | p95 us | p99 us | max us | QPS | exact scan p50 us | f32 MiB | f16 MiB |
| :--- | ---: | :--- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| uniform | 1000 | m=32 m0=64 efC=200 efS=64 heur=true keep=true | 5.07 | 0.9988 | 328.6 | 565.9 | 707.9 | 1016.4 | 2730 | 122.0 | 1.5 | 0.7 |
| uniform | 10000 | m=32 m0=64 efC=200 efS=64 heur=true keep=true | 198.38 | 0.7130 | 1605.2 | 5045.5 | 9427.3 | 26080.4 | 436 | 3342.8 | 14.6 | 7.3 |
| clustered | 1000 | m=32 m0=64 efC=200 efS=64 heur=true keep=true | 3.42 | 0.9998 | 267.8 | 459.1 | 635.6 | 859.9 | 3437 | 121.0 | 1.5 | 0.7 |
| clustered | 10000 | m=32 m0=64 efC=200 efS=64 heur=true keep=true | 52.13 | 0.9990 | 385.6 | 658.1 | 1033.8 | 2225.9 | 2354 | 1861.1 | 14.6 | 7.3 |

## Validator, keyword rules on a 10,000-character document

| mode | text | rules | p50 us | p95 us |
| :--- | :--- | ---: | ---: | ---: |
| literal | no-match | 10 | 32.3 | 52.2 |
| regex | no-match | 10 | 19.0 | 32.5 |
| both | no-match | 10 | 37.8 | 57.2 |
| literal | matches | 10 | 30.9 | 35.9 |
| regex | matches | 10 | 20.6 | 47.0 |
| both | matches | 10 | 35.2 | 35.4 |
| literal | no-match | 100 | 214.4 | 369.1 |
| regex | no-match | 100 | 69.4 | 127.2 |
| both | no-match | 100 | 271.9 | 420.0 |
| literal | matches | 100 | 305.1 | 506.9 |
| regex | matches | 100 | 89.5 | 145.6 |
| both | matches | 100 | 377.9 | 597.4 |
| literal | no-match | 1000 | 2093.8 | 2881.3 |
| regex | no-match | 1000 | 582.0 | 1054.2 |
| both | no-match | 1000 | 2717.8 | 3582.1 |
| literal | matches | 1000 | 2772.1 | 3601.9 |
| regex | matches | 1000 | 691.0 | 1234.1 |
| both | matches | 1000 | 3571.1 | 4535.2 |

Notes
- Memory columns are the vector payload only (N x 384 x 4 or x 2 bytes); graph and process memory are not measured
- `exact scan` is a single-thread f32 scan, not the rayon `KnowledgeStore::linear_search`
- Keyword rules are matched rule by rule (no shared automaton); only conflict items use Aho-Corasick, and that path is not measured here
