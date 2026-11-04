# Engine benchmark

- crate version: 0.3.7
- logical CPUs: 8
- dimension: 384, top-k: 10, queries per row: 1000, seed: 0x5eed20261102
- HNSW parameters for this run: m=16 m0=32 efC=200 efS=64 heur=true keep=true

## Vector search (HNSW, single thread)

| shape | N | params | build s | recall@10 | p50 us | p95 us | p99 us | max us | QPS | exact scan p50 us | f32 MiB | f16 MiB |
| :--- | ---: | :--- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| uniform | 1000 | m=16 m0=32 efC=200 efS=64 heur=true keep=true | 2.75 | 0.9751 | 393.7 | 786.4 | 1167.0 | 2363.3 | 2150 | 217.5 | 1.5 | 0.7 |
| uniform | 10000 | m=16 m0=32 efC=200 efS=64 heur=true keep=true | 55.55 | 0.4786 | 926.8 | 1496.3 | 1806.9 | 11277.4 | 983 | 1826.7 | 14.6 | 7.3 |
| clustered | 1000 | m=16 m0=32 efC=200 efS=64 heur=true keep=true | 1.07 | 0.9998 | 191.0 | 333.4 | 506.1 | 777.8 | 4736 | 120.9 | 1.5 | 0.7 |
| clustered | 10000 | m=16 m0=32 efC=200 efS=64 heur=true keep=true | 17.96 | 0.9990 | 368.2 | 631.8 | 917.9 | 1587.8 | 2523 | 2425.6 | 14.6 | 7.3 |

## Validator, keyword rules on a 10,000-character document

| mode | text | rules | p50 us | p95 us |
| :--- | :--- | ---: | ---: | ---: |
| literal | no-match | 10 | 40.2 | 78.7 |
| regex | no-match | 10 | 23.6 | 23.7 |
| both | no-match | 10 | 47.3 | 78.2 |
| literal | matches | 10 | 38.5 | 66.6 |
| regex | matches | 10 | 25.5 | 51.4 |
| both | matches | 10 | 44.0 | 61.6 |
| literal | no-match | 100 | 260.2 | 508.3 |
| regex | no-match | 100 | 85.7 | 159.8 |
| both | no-match | 100 | 344.6 | 684.5 |
| literal | matches | 100 | 389.8 | 736.4 |
| regex | matches | 100 | 111.7 | 182.6 |
| both | matches | 100 | 477.5 | 930.8 |
| literal | no-match | 1000 | 2946.0 | 4903.2 |
| regex | no-match | 1000 | 785.2 | 1528.3 |
| both | no-match | 1000 | 3106.0 | 4363.3 |
| literal | matches | 1000 | 4509.8 | 10962.2 |
| regex | matches | 1000 | 1262.2 | 2547.7 |
| both | matches | 1000 | 5107.2 | 7440.5 |

Notes
- Memory columns are the vector payload only (N x 384 x 4 or x 2 bytes); graph and process memory are not measured
- `exact scan` is a single-thread f32 scan, not the rayon `KnowledgeStore::linear_search`
- Keyword rules are matched rule by rule (no shared automaton); only conflict items use Aho-Corasick, and that path is not measured here
