# Validator keyword rules: P6 before and after

Change-log row P6 (one Aho-Corasick pass and one RegexSet pass instead of one scan per trigger word).
Both tables come from `cargo run --release --example bench -- --sizes 1000 --queries 20`
(10,000-character document, 300 timed runs per row, same machine and seed, `CARGO_PROFILE_RELEASE_DEBUG=false`).
The vector rows are not part of this comparison. Neither table replaces the other; later runs are added below.

## Before (commit e2d9021, per-trigger scan)

| mode | text | rules | p50 us | p95 us |
| :--- | :--- | ---: | ---: | ---: |
| literal | no-match | 10 | 31.0 | 31.3 |
| regex | no-match | 10 | 22.3 | 22.4 |
| both | no-match | 10 | 37.7 | 38.0 |
| literal | matches | 10 | 29.6 | 31.3 |
| regex | matches | 10 | 23.2 | 23.4 |
| both | matches | 10 | 34.7 | 34.9 |
| literal | no-match | 100 | 173.9 | 179.2 |
| regex | no-match | 100 | 75.2 | 78.1 |
| both | no-match | 100 | 226.3 | 232.6 |
| literal | matches | 100 | 231.9 | 238.0 |
| regex | matches | 100 | 90.0 | 94.4 |
| both | matches | 100 | 301.1 | 318.0 |
| literal | no-match | 1000 | 1358.8 | 1401.8 |
| regex | no-match | 1000 | 532.6 | 554.0 |
| both | no-match | 1000 | 1704.4 | 1782.7 |
| literal | matches | 1000 | 1711.7 | 1805.1 |
| regex | matches | 1000 | 544.4 | 554.0 |
| both | matches | 1000 | 2211.3 | 2261.2 |

## After (shared index)

| mode | text | rules | p50 us | p95 us |
| :--- | :--- | ---: | ---: | ---: |
| literal | no-match | 10 | 14.3 | 14.4 |
| regex | no-match | 10 | 16.5 | 16.6 |
| both | no-match | 10 | 17.3 | 17.7 |
| literal | matches | 10 | 18.6 | 18.8 |
| regex | matches | 10 | 22.2 | 23.1 |
| both | matches | 10 | 24.2 | 24.6 |
| literal | no-match | 100 | 15.0 | 15.1 |
| regex | no-match | 100 | 40.0 | 40.8 |
| both | no-match | 100 | 42.5 | 43.4 |
| literal | matches | 100 | 22.9 | 23.6 |
| regex | matches | 100 | 60.4 | 62.1 |
| both | matches | 100 | 63.1 | 67.9 |
| literal | no-match | 1000 | 21.2 | 21.4 |
| regex | no-match | 1000 | 69.5 | 70.5 |
| both | no-match | 1000 | 72.0 | 76.0 |
| literal | matches | 1000 | 29.4 | 30.1 |
| regex | matches | 1000 | 89.9 | 94.4 |
| both | matches | 1000 | 95.2 | 100.7 |

