# Engine benchmark

- crate version: 0.3.7
- logical CPUs: 8
- dimension: 384, top-k: 10, queries per row: 1000, seed: 0x5eed20261102
- HNSW parameters for this run: m=16 m0=32 efC=200 efS=64 heur=true keep=true

## Vector search (HNSW, single thread)

| shape | N | params | build s | recall@10 | p50 us | p95 us | p99 us | max us | QPS | exact scan p50 us | f32 MiB | f16 MiB |
| :--- | ---: | :--- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| uniform | 1000 | m=16 m0=32 efC=200 efS=64 heur=true keep=true | 1.15 | 0.9751 | 168.0 | 287.2 | 1611.3 | 2616.1 | 4906 | 79.1 | 1.5 | 0.7 |
| uniform | 10000 | m=16 m0=32 efC=200 efS=64 heur=true keep=true | 39.92 | 0.4786 | 926.6 | 1380.1 | 1677.8 | 2651.0 | 1019 | 2289.6 | 14.6 | 7.3 |
| clustered | 1000 | m=16 m0=32 efC=200 efS=64 heur=true keep=true | 1.08 | 0.9998 | 178.8 | 247.0 | 337.5 | 822.2 | 5358 | 116.9 | 1.5 | 0.7 |
| clustered | 10000 | m=16 m0=32 efC=200 efS=64 heur=true keep=true | 15.80 | 0.9990 | 248.2 | 306.8 | 369.3 | 506.1 | 3931 | 1620.4 | 14.6 | 7.3 |

## Validator, keyword rules on a 10,000-character document

| mode | text | rules | p50 us | p95 us |
| :--- | :--- | ---: | ---: | ---: |
| literal | no-match | 10 | 28.7 | 28.8 |
| regex | no-match | 10 | 17.1 | 25.6 |
| both | no-match | 10 | 33.9 | 34.0 |
| literal | matches | 10 | 27.7 | 30.7 |
| regex | matches | 10 | 18.4 | 18.5 |
| both | matches | 10 | 31.6 | 35.1 |
| literal | no-match | 100 | 190.9 | 214.9 |
| regex | no-match | 100 | 62.3 | 72.7 |
| both | no-match | 100 | 242.4 | 288.4 |
| literal | matches | 100 | 269.6 | 294.4 |
| regex | matches | 100 | 80.2 | 86.1 |
| both | matches | 100 | 337.3 | 605.3 |
| literal | no-match | 1000 | 2102.5 | 2867.2 |
| regex | no-match | 1000 | 628.5 | 895.5 |
| both | no-match | 1000 | 2672.5 | 3333.3 |
| literal | matches | 1000 | 2568.7 | 2909.2 |
| regex | matches | 1000 | 637.5 | 842.3 |
| both | matches | 1000 | 3243.7 | 3637.6 |

Notes
- Memory columns are the vector payload only (N x 384 x 4 or x 2 bytes); graph and process memory are not measured
- `exact scan` is a single-thread f32 scan, not the rayon `KnowledgeStore::linear_search`
- Keyword rules are matched rule by rule (no shared automaton); only conflict items use Aho-Corasick, and that path is not measured here
