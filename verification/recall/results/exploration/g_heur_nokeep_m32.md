# Engine benchmark

- crate version: 0.3.7
- logical CPUs: 8
- dimension: 384, top-k: 10, queries per row: 1000, seed: 0x5eed20261102
- HNSW parameters for this run: m=16 m0=32 efC=200 efS=64 heur=true keep=false

## Vector search (HNSW, single thread)

| shape | N | params | build s | recall@10 | p50 us | p95 us | p99 us | max us | QPS | exact scan p50 us | f32 MiB | f16 MiB |
| :--- | ---: | :--- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| uniform | 1000 | m=16 m0=32 efC=200 efS=64 heur=true keep=false | 0.90 | 0.9669 | 464.7 | 953.3 | 1228.0 | 2247.6 | 1788 | 205.6 | 1.5 | 0.7 |
| uniform | 10000 | m=16 m0=32 efC=200 efS=64 heur=true keep=false | 41.76 | 0.4805 | 976.2 | 1540.2 | 2088.3 | 2263.7 | 963 | 1944.6 | 14.6 | 7.3 |
| clustered | 1000 | m=16 m0=32 efC=200 efS=64 heur=true keep=false | 0.37 | 0.9998 | 208.5 | 358.9 | 589.3 | 933.6 | 4301 | 126.5 | 1.5 | 0.7 |
| clustered | 10000 | m=16 m0=32 efC=200 efS=64 heur=true keep=false | 10.72 | 0.9986 | 376.2 | 624.0 | 991.7 | 1741.9 | 2410 | 1975.8 | 14.6 | 7.3 |

## Validator, keyword rules on a 10,000-character document

| mode | text | rules | p50 us | p95 us |
| :--- | :--- | ---: | ---: | ---: |
| literal | no-match | 10 | 35.0 | 39.7 |
| regex | no-match | 10 | 20.7 | 45.7 |
| both | no-match | 10 | 41.2 | 64.2 |
| literal | matches | 10 | 33.7 | 33.9 |
| regex | matches | 10 | 22.6 | 38.7 |
| both | matches | 10 | 38.3 | 77.2 |
| literal | no-match | 100 | 223.6 | 346.2 |
| regex | no-match | 100 | 72.2 | 106.0 |
| both | no-match | 100 | 283.6 | 414.4 |
| literal | matches | 100 | 317.8 | 496.3 |
| regex | matches | 100 | 94.2 | 162.2 |
| both | matches | 100 | 410.7 | 768.4 |
| literal | no-match | 1000 | 2228.9 | 3119.8 |
| regex | no-match | 1000 | 632.0 | 1143.9 |
| both | no-match | 1000 | 3012.6 | 4101.8 |
| literal | matches | 1000 | 2893.5 | 3871.1 |
| regex | matches | 1000 | 749.3 | 1303.9 |
| both | matches | 1000 | 3628.3 | 4486.9 |

Notes
- Memory columns are the vector payload only (N x 384 x 4 or x 2 bytes); graph and process memory are not measured
- `exact scan` is a single-thread f32 scan, not the rayon `KnowledgeStore::linear_search`
- Keyword rules are matched rule by rule (no shared automaton); only conflict items use Aho-Corasick, and that path is not measured here
