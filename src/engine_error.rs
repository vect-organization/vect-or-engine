// Copyright (c) 2026 1abcdefggs
// SPDX-License-Identifier: MIT
// https://github.com/1abcdefggs/vect-or-engine

#![forbid(unsafe_code)]

//! Centralized error types for vect-or-engine.
//! All error variants are typed, allowing callers to branch on specific failures.

use thiserror::Error;

#[derive(Error, Debug)]
pub enum EngineError {
    // ── I/O ────────────────────────────────────────────────────────────────
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    // ── Serialization ──────────────────────────────────────────────────────
    #[error("JSON parse error: {0}")]
    Json(#[from] serde_json::Error),

    // ── Concurrency ────────────────────────────────────────────────────────
    #[error("State lock poisoned")]
    Lock,

    // ── Protocol ───────────────────────────────────────────────────────────
    #[error("Unknown RPC method: '{0}'")]
    UnknownMethod(String),

    #[error("Missing required parameter: '{0}'")]
    MissingParam(String),

    #[error("Invalid path provided: {0}")]
    InvalidPath(String),

    #[error("Invalid input: {0}")]
    InvalidInput(String),

    // ── Search ─────────────────────────────────────────────────────────────
    #[error("HNSW index not built. Call 'build_index' first.")]
    IndexNotBuilt,

    #[error("Vector dimension mismatch: expected {expected}, got {got}")]
    DimensionMismatch { expected: usize, got: usize },

    // ── Storage ────────────────────────────────────────────────────────────
    #[error("Database error: {0}")]
    Database(String),
}

/// Convenience conversion so EngineError can be turned into a JSON-safe string.
impl From<EngineError> for String {
    fn from(e: EngineError) -> String {
        e.to_string()
    }
}
