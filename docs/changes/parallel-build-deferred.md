# engine:parallel-build-deferred

Change-log row: R2 (`docs/changes/v0.4.0.csv`). Decision of the owner, 2026-11-06: the opt-in parallel HNSW build is not added in 0.4.0.

## Decision

- The index build stays sequential and deterministic: the same matrix and parameters give the same graph (`r2_two_builds_equal` in `src/hnsw_index.rs`)
- No parallel mode is added now. R2 is closed as "property kept"

## Reasons

- Reproducibility: inserting nodes concurrently (locks per node) makes the graph depend on thread timing. That is the opposite of what R2 asks for
- Search speed and recall: HNSW graphs built by concurrent insertion are known to lose some recall. The deterministic alternative (build S shards at the same time, search every shard, merge the results) multiplies the search work by S. Neither option is free, and there is no real dictionary to measure them on
- UI: the build already runs off the lock on a blocking thread (P1), on one core, so the editor stays responsive. A parallel build would take the other cores while the user is working
- No user: an index is built only from 10,000 items on (`EXACT_SCAN_MAX_ITEMS`). Every dictionary in use has fewer than 2,000 items, so the code would be unused and untested on real data

## Trigger to reopen

- A real dictionary of 10,000 items or more exists, or
- A build time over 10 s is measured on a user machine

Reference build times (sequential, 384 dimensions, `verification/c2/threshold_parallel_2026-11-06.md`): about 5 s at 2,000 items, 17 s at 5,000 and 31 s at 10,000.

## Design to start from

- Preferred: a sharded build. Split the matrix into S contiguous parts, build each part with the existing sequential code at the same time (deterministic per part, no locks), search the parts and merge the top-k lists. Measure search latency and recall first
- Only if sharding fails the measurement: concurrent insertion with a lock per node, layers drawn up front from the same random sequence, the first ~1,000 nodes inserted sequentially, a shared neighbour-choice function so both builders use the same code
- Constraints: the sequential build stays the default; any parallel mode is opt-in (for example `buildIndex({ parallel: true })`); a dedicated pool with cores - 1 threads so the UI keeps a core; `r2_two_builds_equal` stays unchanged

## Acceptance numbers for a future implementation

- At least 3 times faster on 8 logical CPUs at 10,000 items
- Recall within 0.02 of the sequential build, measured against the exact scan
- Search p50 not worse than 1.5 times the sequential index
