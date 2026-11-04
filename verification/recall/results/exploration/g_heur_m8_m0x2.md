# Engine benchmark

- crate version: 0.3.7
- logical CPUs: 8
- dimension: 384, top-k: 10, queries per row: 1000, seed: 0x5eed20261102
- HNSW parameters for this run: m=8 m0=16 efC=200 efS=64 heur=true keep=true

## Vector search (HNSW, single thread)

| shape | N | params | build s | recall@10 | p50 us | p95 us | p99 us | max us | QPS | exact scan p50 us | f32 MiB | f16 MiB |
| :--- | ---: | :--- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| uniform | 1000 | m=8 m0=16 efC=200 efS=64 heur=true keep=true | 0.82 | 0.8440 | 324.6 | 624.1 | 937.1 | 1970.3 | 2695 | 174.2 | 1.5 | 0.7 |
| uniform | 10000 | m=8 m0=16 efC=200 efS=64 heur=true keep=true | 14.36 | 0.2687 | 558.2 | 920.5 | 1324.8 | 2138.4 | 1650 | 2013.1 | 14.6 | 7.3 |
| clustered | 1000 | m=8 m0=16 efC=200 efS=64 heur=true keep=true | 0.56 | 0.9998 | 175.4 | 335.9 | 474.9 | 870.9 | 5026 | 154.4 | 1.5 | 0.7 |
| clustered | 10000 | m=8 m0=16 efC=200 efS=64 heur=true keep=true | 10.25 | 0.9963 | 309.3 | 570.3 | 862.1 | 8912.7 | 2804 | 2580.0 | 14.6 | 7.3 |

## Validator, keyword rules on a 10,000-character document

| mode | text | rules | p50 us | p95 us |
| :--- | :--- | ---: | ---: | ---: |
| literal | no-match | 10 | 44.8 | 74.0 |
| regex | no-match | 10 | 26.4 | 27.0 |
| both | no-match | 10 | 85.0 | 134.9 |
| literal | matches | 10 | 43.1 | 94.7 |
| regex | matches | 10 | 30.2 | 54.2 |
| both | matches | 10 | 82.2 | 111.4 |
| literal | no-match | 100 | 293.0 | 700.4 |
| regex | no-match | 100 | 154.1 | 349.2 |
| both | no-match | 100 | 693.2 | 1865.3 |
| literal | matches | 100 | 475.6 | 1054.1 |
| regex | matches | 100 | 127.6 | 241.9 |
| both | matches | 100 | 673.3 | 1258.7 |
| literal | no-match | 1000 | 4218.3 | 7369.8 |
| regex | no-match | 1000 | 1207.6 | 3247.0 |
| both | no-match | 1000 | 4077.5 | 7076.4 |
| literal | matches | 1000 | 4330.8 | 7550.2 |
| regex | matches | 1000 | 1427.2 | 3575.0 |
| both | matches | 1000 | 8108.9 | 12266.3 |

Notes
- Memory columns are the vector payload only (N x 384 x 4 or x 2 bytes); graph and process memory are not measured
- `exact scan` is a single-thread f32 scan, not the rayon `KnowledgeStore::linear_search`
- Keyword rules are matched rule by rule (no shared automaton); only conflict items use Aho-Corasick, and that path is not measured here
