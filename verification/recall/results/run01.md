# Engine benchmark

- crate version: 0.3.7
- logical CPUs: 8
- dimension: 384, top-k: 10, queries per row: 1000, seed: 0x5eed20261102
- HNSW parameters for this run: m=16 m0=32 efC=200 efS=64 heur=true keep=true

## Vector search (HNSW, single thread)

| shape | N | params | build s | recall@10 | p50 us | p95 us | p99 us | max us | QPS | exact scan p50 us | f32 MiB | f16 MiB |
| :--- | ---: | :--- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| uniform | 1000 | m=16 m0=32 efC=200 efS=64 heur=true keep=true | 2.44 | 0.9751 | 845.2 | 1666.0 | 2616.9 | 22290.3 | 1015 | 292.1 | 1.5 | 0.7 |
| uniform | 10000 | m=16 m0=32 efC=200 efS=64 heur=true keep=true | 40.54 | 0.4786 | 1059.5 | 1553.1 | 2171.5 | 37698.6 | 879 | 2163.7 | 14.6 | 7.3 |
| clustered | 1000 | m=16 m0=32 efC=200 efS=64 heur=true keep=true | 1.37 | 0.9998 | 273.7 | 455.1 | 598.1 | 980.6 | 3321 | 167.2 | 1.5 | 0.7 |
| clustered | 10000 | m=16 m0=32 efC=200 efS=64 heur=true keep=true | 16.83 | 0.9990 | 315.4 | 438.6 | 568.0 | 1464.7 | 3016 | 1850.2 | 14.6 | 7.3 |

## Validator, keyword rules on a 10,000-character document

| mode | text | rules | p50 us | p95 us |
| :--- | :--- | ---: | ---: | ---: |
| literal | no-match | 10 | 33.5 | 53.1 |
| regex | no-match | 10 | 19.8 | 31.4 |
| both | no-match | 10 | 39.5 | 45.2 |
| literal | matches | 10 | 32.3 | 50.6 |
| regex | matches | 10 | 21.4 | 28.4 |
| both | matches | 10 | 36.8 | 40.9 |
| literal | no-match | 100 | 223.9 | 337.1 |
| regex | no-match | 100 | 70.0 | 88.8 |
| both | no-match | 100 | 271.3 | 365.2 |
| literal | matches | 100 | 302.2 | 416.7 |
| regex | matches | 100 | 90.2 | 137.2 |
| both | matches | 100 | 366.3 | 441.2 |
| literal | no-match | 1000 | 1991.4 | 2337.9 |
| regex | no-match | 1000 | 542.9 | 821.3 |
| both | no-match | 1000 | 2661.1 | 3158.7 |
| literal | matches | 1000 | 2606.4 | 2974.1 |
| regex | matches | 1000 | 675.7 | 968.2 |
| both | matches | 1000 | 3565.3 | 4297.8 |

Notes
- Memory columns are the vector payload only (N x 384 x 4 or x 2 bytes); graph and process memory are not measured
- `exact scan` is a single-thread f32 scan, not the rayon `KnowledgeStore::linear_search`
- Keyword rules are matched rule by rule (no shared automaton); only conflict items use Aho-Corasick, and that path is not measured here
