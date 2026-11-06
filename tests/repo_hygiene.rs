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
fn d11_store_fields_are_private() {
    let src = read("src/knowledge_store.rs");
    for field in ["pub items:", "pub dim:", "pub vectors_f16:", "pub hnsw:", "pub generation:"] {
        assert!(!src.contains(field), "KnowledgeStore has a public field again: {field}");
    }
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

#[test]
fn s3_sqlite_is_optional_and_off_by_default() {
    let manifest = read("Cargo.toml");
    let line = manifest.lines().find(|l| l.starts_with("rusqlite")).expect("rusqlite line");
    assert!(line.contains("optional = true"), "rusqlite must stay optional: {line}");
    let features = manifest.split("[features]").nth(1).unwrap().split("[dependencies]").next().unwrap();
    let default = features.lines().find(|l| l.starts_with("default")).unwrap();
    assert!(!default.contains("cache"), "the cache feature must not be a default feature");
    assert!(features.contains("cache = [\"dep:rusqlite\"]"));
}

#[test]
fn s6_every_path_taking_export_uses_the_one_path_check() {
    let src = read("src/napi_bindings.rs");
    let mut checked = 0;
    for part in src.split("#[napi]").skip(1) {
        let head = part.lines().find(|l| l.contains("fn ")).unwrap_or("");
        if head.contains("path: String") {
            let body: String = part.lines().take(12).collect::<Vec<_>>().join("\n");
            assert!(body.contains("verify_safe_path(&path)?"), "no path check in: {head}");
            checked += 1;
        }
    }
    assert!(checked >= 4, "expected the profile, KB and two cache exports, found {checked}");
}

#[test]
fn s9_mcp_server_is_gone() {
    // The experimental server was removed; its design notes stay under docs/.
    let root = env!("CARGO_MANIFEST_DIR");
    assert!(!std::path::Path::new(&format!("{root}/mcp")).exists(), "mcp/ is back");
    assert!(!read("package.json").contains("start-mcp"), "start-mcp is back in package.json");
    assert!(!read("README.md").contains("start-mcp"), "README describes the removed server");
}
