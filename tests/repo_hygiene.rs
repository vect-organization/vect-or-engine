// Copyright (c) 2026 1abcdefggs
// SPDX-License-Identifier: MIT
// https://github.com/1abcdefggs/vect-or-engine

//! Guards for change-log rows D13 and S7: removed dependencies stay removed, and the
//! documentation does not describe code that no longer exists.

use std::fs;

fn read(path: &str) -> String {
    fs::read_to_string(format!("{}/{path}", env!("CARGO_MANIFEST_DIR"))).expect(path)
}

#[test]
fn s7_removed_dependencies_stay_removed() {
    let manifest = read("Cargo.toml");
    for name in ["anyhow", "once_cell", "memmap2"] {
        assert!(!manifest.contains(&format!("{name} =")), "{name} is back in Cargo.toml");
    }
    // tokio is optional: the core build (no default features) must not pull it in.
    assert!(manifest.contains("optional = true"), "tokio must stay optional");
}

#[test]
fn d13_docs_match_code() {
    let readme = read("README.md");
    for stale in ["memmap2", "4-way unrolled", "mmap"] {
        assert!(!readme.contains(stale), "README still mentions '{stale}'");
    }
    for file in ["src/main.rs", "src/knowledge_store.rs", "src/hnsw_index.rs"] {
        let src = read(file);
        assert!(!src.contains("memmap2"), "{file} still mentions memmap2");
        assert!(!src.contains("get_unchecked"), "{file} uses get_unchecked");
    }
}
