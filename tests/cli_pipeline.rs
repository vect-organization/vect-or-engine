// Copyright (c) 2026 1abcdefggs
// SPDX-License-Identifier: MIT
// https://github.com/1abcdefggs/vect-or-engine

//! Change-log rows D9 and D10: the CLI checks the query dimension, rejects non-numeric
//! entries, and handles piped requests in order, answering every one before it exits.

#![cfg(feature = "cli")]

use std::io::Write;
use std::process::{Command, Stdio};

/// Run the CLI in a scratch directory (the CLI accepts relative paths only) and return
/// one parsed JSON reply per input line.
fn run_cli(tag: &str, kb: &str, lines: &[&str]) -> Vec<serde_json::Value> {
    let dir = std::env::temp_dir().join(format!("vect-cli-{tag}-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    std::fs::write(dir.join("kb.json"), kb).unwrap();

    let mut child = Command::new(env!("CARGO_BIN_EXE_vect_or_engine_cli"))
        .current_dir(&dir)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
        .unwrap();
    {
        let mut stdin = child.stdin.take().unwrap();
        for l in lines {
            writeln!(stdin, "{l}").unwrap();
        }
        // Dropping stdin sends EOF right after the last request.
    }
    let out = child.wait_with_output().unwrap();
    std::fs::remove_dir_all(&dir).ok();
    String::from_utf8(out.stdout)
        .unwrap()
        .lines()
        .map(|l| serde_json::from_str(l).unwrap())
        .collect()
}

const KB: &str = r#"[{"id":"a","vector":[1.0,0.0]},{"id":"b","vector":[0.0,1.0]}]"#;

#[test]
fn d10_cli_pipelined_load_then_build() {
    let replies = run_cli(
        "d10",
        KB,
        &[
            r#"{"method":"load_knowledge_base","params":{"path":"kb.json"},"id":"1"}"#,
            r#"{"method":"build_index","id":"2"}"#,
            r#"{"method":"search","params":{"vector":[1.0,0.0],"top_k":1},"id":"3"}"#,
            r#"{"method":"kb_info","id":"4"}"#,
        ],
    );
    // Every request is answered (nothing lost at EOF) and in the order sent.
    let ids: Vec<&str> = replies.iter().map(|r| r["id"].as_str().unwrap()).collect();
    assert_eq!(ids, ["1", "2", "3", "4"], "{replies:?}");
    assert!(replies.iter().all(|r| r["success"] == true), "{replies:?}");
    // Two items are far under the index threshold, so the exact scan answers.
    assert_eq!(replies[2]["data"]["mode"], "linear");
    assert_eq!(replies[2]["data"]["results"][0]["id"], "a");
}

#[test]
fn d9_cli_search_checks_dim() {
    let replies = run_cli(
        "d9",
        KB,
        &[
            r#"{"method":"load_knowledge_base","params":{"path":"kb.json"},"id":"1"}"#,
            r#"{"method":"search","params":{"vector":[1.0,0.0,0.0]},"id":"2"}"#,
            r#"{"method":"search","params":{"vector":[1.0,"x"]},"id":"3"}"#,
            r#"{"method":"search","params":{"vector":[1.0,0.0]},"id":"4"}"#,
        ],
    );
    assert_eq!(replies[1]["success"], false, "{replies:?}");
    assert!(replies[1]["error"].as_str().unwrap().contains("expected 2"), "{replies:?}");
    assert_eq!(replies[2]["success"], false, "{replies:?}");
    assert!(replies[2]["error"].as_str().unwrap().contains("not a number"), "{replies:?}");
    assert_eq!(replies[3]["success"], true, "{replies:?}");
}
