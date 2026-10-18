// Copyright (c) 2026 1abcdefggs
// SPDX-License-Identifier: MIT
// https://github.com/1abcdefggs/vect-or-engine

//! NAPI-rs bindings for vect-or-engine (Phase 2).
//! Exposes the core Rust engine functions to Node.js as a native addon.

#![deny(clippy::all)]

#[macro_use]
extern crate napi_derive;

use napi::bindgen_prelude::*;
use once_cell::sync::Lazy;
use std::sync::Arc;
use tokio::sync::RwLock;

#[path = "src/engine_error.rs"]
pub mod engine_error;
#[path = "src/profile.rs"]
pub mod profile;
#[path = "src/validator.rs"]
pub mod validator;
#[path = "src/translator.rs"]
pub mod translator;
#[path = "src/hnsw_index.rs"]
pub mod hnsw_index;
#[path = "src/knowledge_store.rs"]
pub mod knowledge_store;

pub use engine_error::EngineError;
pub use hnsw_index::HnswIndex;
pub use knowledge_store::{KnowledgeItem, KnowledgeStore, SearchQuery, SearchResult};
pub use profile::Profile;
pub use translator::{TranslationMatch, Translator};
pub use validator::{MatchMode, ValidationMarker, ValidationResult, Validator};

// --- Global Engine State ---

/// The single, global, thread-safe engine instance.
#[allow(dead_code)]
struct Engine {
    profile: Profile,
    validator: Validator,
    translator: Translator,
    knowledge_store: KnowledgeStore,
}

impl Engine {
    fn new(profile: Profile) -> Self {
        let validator = Validator::new(&profile);
        let translator = Translator::new(&profile);
        Self {
            profile,
            validator,
            translator,
            knowledge_store: KnowledgeStore::new(),
        }
    }
}

static ENGINE: Lazy<Arc<RwLock<Engine>>> =
    Lazy::new(|| Arc::new(RwLock::new(Engine::new(Profile::default()))));
/// A simple ping function to verify the NAPI setup.
#[napi]
pub fn ping(name: String) -> String {
  format!("pong from rust: {name}!")
}

// --- N-API Data Structures ---
// These structs mirror the internal engine structs but are annotated for NAPI.

#[napi(object)]
pub struct JsValidationMarker {
  pub line: u32,
  pub severity: String,
  pub message: String,
}

#[napi(object)]
pub struct JsValidationResult {
  pub is_valid: bool,
  pub markers: Vec<JsValidationMarker>,
}

#[napi(object)]
pub struct JsSearchResult {
    pub idx: u32,
    pub score: f64,
    pub id: Option<String>,
    pub metadata: serde_json::Value,
}

impl From<&SearchResult> for JsSearchResult {
    fn from(r: &SearchResult) -> Self {
        Self {
            idx: r.idx as u32,
            score: r.score as f64,
            id: r.id.clone(),
            metadata: serde_json::Value::Object(r.metadata.clone()),
        }
    }
}

#[napi(object)]
pub struct JsKbInfo {
    pub count: u32,
    pub dim: u32,
    #[napi(js_name = "hnswReady")]
    pub hnsw_ready: bool,
}

// --- Conversion Implementations ---
// Convert from internal engine types to N-API JS types.

impl From<ValidationMarker> for JsValidationMarker {
    fn from(marker: ValidationMarker) -> Self {
        Self {
            line: marker.line,
            severity: marker.severity,
            message: marker.message,
        }
    }
}

impl From<ValidationResult> for JsValidationResult {
    fn from(result: ValidationResult) -> Self {
        Self {
            is_valid: result.is_valid,
            markers: result.markers.into_iter().map(Into::into).collect(),
        }
    }
}

// --- Exposed N-API Functions ---

/// Validates a document using the currently loaded profile.
#[napi]
pub fn validate_sync(text: String) -> Result<JsValidationResult> {
    let engine = ENGINE.blocking_read();
    let result = engine.validator.validate(&text);
    Ok(result.into())
}

#[napi]
pub async fn validate(text: String) -> Result<JsValidationResult> {
    let engine = ENGINE.read().await;
    let result = engine.validator.validate(&text);
    Ok(result.into())
}

/// `match_mode` is `"literal"` (default), `"regex"` or `"both"` (ADR-0006). To change the mode,
/// call this again with the same profile; the rules are rebuilt and the KB is kept.
#[napi]
pub async fn load_profile(path: String, match_mode: Option<String>) -> Result<()> {
    verify_safe_path(&path)?;
    let mode = match match_mode.as_deref() {
        None => MatchMode::default(),
        Some(name) => MatchMode::parse(name).ok_or_else(|| {
            Error::new(
                Status::InvalidArg,
                format!("Unknown match_mode '{name}' (expected literal, regex or both)"),
            )
        })?,
    };
    let content = std::fs::read_to_string(&path)
        .map_err(|e| Error::new(Status::GenericFailure, format!("Failed to read profile: {e}")))?;
    let profile = Profile::load_any_json_str(&content)
        .map_err(|e| Error::new(Status::GenericFailure, format!("Failed to parse profile: {e}")))?;
    if profile.is_empty() {
        return Err(Error::new(Status::InvalidArg, Profile::EMPTY_MESSAGE.to_string()));
    }
    // Build everything that can fail (e.g. an invalid regex) before touching the engine,
    // so a bad profile leaves the previously loaded rules in place.
    let validator = Validator::with_mode(&profile, mode)
        .map_err(|e| Error::new(Status::InvalidArg, e))?;

    // Swap rules only. Replacing the whole Engine (previous behaviour) also reset
    // knowledge_store, silently discarding a loaded KB and its HNSW index.
    let mut engine = ENGINE.write().await;
    engine.validator = validator;
    engine.translator = Translator::new(&profile);
    engine.profile = profile;
    Ok(())
}

#[napi]
pub async fn load_knowledge_base(path: String) -> Result<u32> {
    verify_safe_path(&path)?;
    let mut engine = ENGINE.write().await;
    let count = engine
        .knowledge_store
        .load_from_path(&path)
        .map_err(|e| Error::new(Status::GenericFailure, e.to_string()))?;
    Ok(count as u32)
}

#[napi]
pub async fn load_kb_cache(path: String) -> Result<u32> {
    verify_safe_path(&path)?;
    let mut engine = ENGINE.write().await;
    let count = engine
        .knowledge_store
        .load_from_sqlite(&path)
        .map_err(|e| Error::new(Status::GenericFailure, e.to_string()))?;
    Ok(count as u32)
}

#[napi]
pub async fn save_kb_cache(path: String) -> Result<()> {
    verify_safe_path(&path)?;
    let engine = ENGINE.read().await;
    engine
        .knowledge_store
        .save_to_sqlite(&path)
        .map_err(|e| Error::new(Status::GenericFailure, e.to_string()))
}

#[napi]
pub async fn build_index() -> Result<u32> {
    // Clone Arc to move it into the blocking task for CPU-intensive work.
    let engine_arc = ENGINE.clone();
    tokio::task::spawn_blocking(move || {
        // Re-lock inside the new thread.
        let mut engine = engine_arc.blocking_write();
        if engine.knowledge_store.is_empty() {
            return Err("Knowledge base is empty.".to_string());
        }
        engine.knowledge_store.build_index();
        Ok(engine.knowledge_store.len() as u32)
    })
    .await
    .map_err(|e| Error::new(Status::GenericFailure, e.to_string()))? // Handle task join error
    .map_err(|e| Error::new(Status::GenericFailure, e)) // Handle our custom error
}

/// ADR-0005 Phase4.3: reject a query from a different vector space instead of letting
/// dot_product_simd truncate to min(len) and return meaningless scores. `expected == 0`
/// means no KB is loaded yet, so there is nothing to compare against.
fn ensure_query_dim(expected: usize, got: usize) -> Result<()> {
    if expected != 0 && got != expected {
        return Err(Error::new(
            Status::InvalidArg,
            EngineError::DimensionMismatch { expected, got }.to_string(),
        ));
    }
    Ok(())
}

#[napi]
pub async fn search(query: Float32Array, top_k: u32) -> Result<Vec<JsSearchResult>> {
    let engine = ENGINE.read().await;
    // The caller (vect-or-edit) falls back to Module 0 (P.scan dictionary matching) on this error.
    ensure_query_dim(engine.knowledge_store.dim, query.len())?;
    // Pass query by reference to avoid allocation.
    let results = engine.knowledge_store.search(&query, top_k as usize);
    Ok(results.iter().map(Into::into).collect())
}

#[napi]
pub async fn kb_info() -> Result<JsKbInfo> {
    let engine = ENGINE.read().await;
    Ok(JsKbInfo {
        count: engine.knowledge_store.len() as u32,
        dim: engine.knowledge_store.dim as u32,
        hnsw_ready: engine.knowledge_store.has_index(),
    })
}

/// SECURITY: Basic path safety check.
/// Ensures non-empty and validates against null-byte poisoning.
fn verify_safe_path(path_str: &str) -> Result<()> {
    if path_str.trim().is_empty() {
        return Err(Error::new(
            Status::InvalidArg,
            EngineError::InvalidPath("Path cannot be empty".to_string()).to_string(),
        ));
    }
    if path_str.contains('\0') {
        return Err(Error::new(
            Status::InvalidArg,
            EngineError::InvalidPath("Path cannot contain null bytes".to_string()).to_string(),
        ));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::knowledge_store::SlimItem;

    #[test]
    fn query_dim_check_skips_when_no_kb_is_loaded() {
        assert!(ensure_query_dim(0, 3).is_ok());
    }

    #[test]
    fn query_dim_check_accepts_matching_dim() {
        assert!(ensure_query_dim(384, 384).is_ok());
    }

    #[test]
    fn query_dim_check_rejects_mismatch() {
        let err = ensure_query_dim(384, 768).unwrap_err();
        assert!(err.reason.contains("expected 384"), "reason: {}", err.reason);
        assert!(err.reason.contains("got 768"), "reason: {}", err.reason);
    }

    fn write_temp_profile(name: &str, body: &str) -> String {
        let path = std::env::temp_dir()
            .join(format!("vect_engine_{name}_{}.json", std::process::id()));
        std::fs::write(&path, body).expect("temp profile must be writable");
        path.to_string_lossy().into_owned()
    }

    // Single test on purpose: it mutates the global ENGINE, and parallel tests would race on it.
    #[test]
    fn load_profile_keeps_kb_and_rejects_empty_profile() {
        let rt = tokio::runtime::Runtime::new().expect("tokio runtime");
        rt.block_on(async {
            {
                let mut e = ENGINE.write().await;
                e.knowledge_store.dim = 3;
                e.knowledge_store.items.push(SlimItem {
                    id: Some("kb-1".to_string()),
                    metadata: serde_json::Map::new(),
                });
            }

            // 0.3.5 regression: swapping rules must keep the loaded KB and its dimension.
            let valid = write_temp_profile(
                "valid",
                r#"{"conversational_translation_rules":{"rules":[{"item_id":"X","item_name":"n","colloquial_patterns":["foo"],"templates":{"A":"t"}}]}}"#,
            );
            load_profile(valid, None).await.expect("valid profile must load");
            {
                let e = ENGINE.read().await;
                assert_eq!(e.knowledge_store.dim, 3, "dim was reset by load_profile");
                assert_eq!(e.knowledge_store.items.len(), 1, "KB items were dropped");
                assert!(!e.translator.match_colloquial("foo").is_empty());
            }

            // An empty profile must be refused and must not touch the loaded state.
            let empty = write_temp_profile("empty", "{}");
            let err = load_profile(empty, None).await.expect_err("empty profile must be rejected");
            assert_eq!(err.reason, Profile::EMPTY_MESSAGE);
            let e = ENGINE.read().await;
            assert_eq!(e.knowledge_store.dim, 3);
            assert_eq!(e.knowledge_store.items.len(), 1);
        });
    }
}