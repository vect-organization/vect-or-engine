# Engine benchmark

- crate version: 0.3.7
- logical CPUs: 8
- dimension: 384, top-k: 10, queries per row: 1000, seed: 0x5eed20261102
- HNSW parameters for this run: m=16 m0=16 efC=200 efS=64 heur=false

## Vector search (HNSW, single thread)

| shape | N | params | build s | recall@10 | p50 us | p95 us | p99 us | max us | QPS | exact scan p50 us | f32 MiB | f16 MiB |
| :--- | ---: | :--- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| uniform | 1000 | m=16 m0=16 efC=200 efS=64 heur=false | 0.25 | 0.8374 | 191.2 | 331.5 | 508.2 | 1397.0 | 4675 | 117.3 | 1.5 | 0.7 |
| uniform | 10000 | m=16 m0=16 efC=200 efS=64 heur=false | 12.11 | 0.2629 | 693.0 | 1211.5 | 1753.4 | 3167.9 | 1334 | 3123.7 | 14.6 | 7.3 |
| clustered | 1000 | m=16 m0=16 efC=200 efS=64 heur=false | 0.34 | 0.6227 | 38.7 | 143.0 | 271.5 | 493.8 | 16951 | 158.1 | 1.5 | 0.7 |
| clustered | 10000 | m=16 m0=16 efC=200 efS=64 heur=false | 3.54 | 0.6558 | 163.4 | 242.1 | 321.9 | 455.0 | 5775 | 1790.9 | 14.6 | 7.3 |

## Validator, keyword rules on a 10,000-character document

| mode | text | rules | p50 us | p95 us |
| :--- | :--- | ---: | ---: | ---: |
| literal | no-match | 10 | 34.7 | 34.8 |
| regex | no-match | 10 | 26.9 | 32.2 |
| both | no-match | 10 | 40.0 | 46.4 |
| literal | matches | 10 | 34.6 | 35.0 |
| regex | matches | 10 | 28.8 | 30.3 |
| both | matches | 10 | 39.3 | 42.6 |
| literal | no-match | 100 | 163.5 | 171.8 |
| regex | no-match | 100 | 75.0 | 81.2 |
| both | no-match | 100 | 208.4 | 237.9 |
| literal | matches | 100 | 230.0 | 265.6 |
| regex | matches | 100 | 91.8 | 115.9 |
| both | matches | 100 | 299.4 | 354.2 |
| literal | no-match | 1000 | 1432.6 | 1685.5 |
| regex | no-match | 1000 | 539.8 | 1282.1 |
| both | no-match | 1000 | 2772.8 | 4818.5 |
| literal | matches | 1000 | 2675.3 | 3937.8 |
| regex | matches | 1000 | 877.7 | 1664.1 |
| both | matches | 1000 | 3310.0 | 4277.7 |

Notes
- Memory columns are the vector payload only (N x 384 x 4 or x 2 bytes); graph and process memory are not measured
- `exact scan` is a single-thread f32 scan, not the rayon `KnowledgeStore::linear_search`
- Keyword rules are matched rule by rule (no shared automaton); only conflict items use Aho-Corasick, and that path is not measured here
