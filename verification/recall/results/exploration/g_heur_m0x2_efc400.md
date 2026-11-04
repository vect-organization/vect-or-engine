# Engine benchmark

- crate version: 0.3.7
- logical CPUs: 8
- dimension: 384, top-k: 10, queries per row: 1000, seed: 0x5eed20261102
- HNSW parameters for this run: m=16 m0=32 efC=400 efS=64 heur=true keep=true

## Vector search (HNSW, single thread)

| shape | N | params | build s | recall@10 | p50 us | p95 us | p99 us | max us | QPS | exact scan p50 us | f32 MiB | f16 MiB |
| :--- | ---: | :--- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| uniform | 1000 | m=16 m0=32 efC=400 efS=64 heur=true keep=true | 1.87 | 0.9751 | 254.0 | 420.1 | 690.8 | 897.5 | 3555 | 117.3 | 1.5 | 0.7 |
| uniform | 10000 | m=16 m0=32 efC=400 efS=64 heur=true keep=true | 44.49 | 0.4827 | 1080.8 | 1816.9 | 2202.3 | 3402.9 | 859 | 1813.3 | 14.6 | 7.3 |
| clustered | 1000 | m=16 m0=32 efC=400 efS=64 heur=true keep=true | 1.27 | 0.9998 | 235.0 | 464.0 | 691.5 | 1496.8 | 3665 | 221.3 | 1.5 | 0.7 |
| clustered | 10000 | m=16 m0=32 efC=400 efS=64 heur=true keep=true | 27.56 | 0.9990 | 424.5 | 713.2 | 1119.4 | 2263.2 | 2127 | 2331.0 | 14.6 | 7.3 |

## Validator, keyword rules on a 10,000-character document

| mode | text | rules | p50 us | p95 us |
| :--- | :--- | ---: | ---: | ---: |
| literal | no-match | 10 | 40.3 | 40.7 |
| regex | no-match | 10 | 23.8 | 37.0 |
| both | no-match | 10 | 47.5 | 83.2 |
| literal | matches | 10 | 38.8 | 64.0 |
| regex | matches | 10 | 25.8 | 27.4 |
| both | matches | 10 | 44.0 | 64.9 |
| literal | no-match | 100 | 268.6 | 564.3 |
| regex | no-match | 100 | 92.8 | 178.7 |
| both | no-match | 100 | 342.7 | 727.2 |
| literal | matches | 100 | 380.9 | 777.4 |
| regex | matches | 100 | 106.9 | 197.5 |
| both | matches | 100 | 453.6 | 915.8 |
| literal | no-match | 1000 | 2542.5 | 4181.5 |
| regex | no-match | 1000 | 726.5 | 1470.8 |
| both | no-match | 1000 | 3387.3 | 5200.9 |
| literal | matches | 1000 | 3451.9 | 5331.2 |
| regex | matches | 1000 | 858.0 | 1853.4 |
| both | matches | 1000 | 4641.5 | 7396.8 |

Notes
- Memory columns are the vector payload only (N x 384 x 4 or x 2 bytes); graph and process memory are not measured
- `exact scan` is a single-thread f32 scan, not the rayon `KnowledgeStore::linear_search`
- Keyword rules are matched rule by rule (no shared automaton); only conflict items use Aho-Corasick, and that path is not measured here
