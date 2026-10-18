// Copyright (c) 2026 1abcdefggs
// SPDX-License-Identifier: MIT
// https://github.com/1abcdefggs/vect-or-engine

//! vect-or-engine -- stdin/stdout JSON-RPC server (Phase 1+3).
//!
//! Phase 1: tokio async runtime, mpsc-serialised stdout, Arc<tokio::sync::RwLock>.
//! Phase 3: KnowledgeStore (memmap2+f16+HNSW), build_index, SQLite cache.

use std::sync::Arc;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader, BufWriter};
use tokio::sync::{mpsc, RwLock};
use vect_or_engine::{EngineError, KnowledgeStore, Profile, Translator, Validator};

// RPC types
#[derive(Deserialize, Debug)]
struct RpcRequest { method: String, params: Option<Value>, id: Option<String> }

#[derive(Serialize, Debug)]
struct RpcResponse {
    success: bool,
    #[serde(skip_serializing_if = "Option::is_none")] data: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")] error: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")] id: Option<String>,
}

impl RpcResponse {
    fn ok(data: Value, id: Option<String>) -> Self {
        RpcResponse { success: true, data: Some(data), error: None, id }
    }
    fn err(msg: impl Into<String>, id: Option<String>) -> Self {
        RpcResponse { success: false, data: None, error: Some(msg.into()), id }
    }
    fn from_engine_err(e: EngineError, id: Option<String>) -> Self {
        Self::err(e.to_string(), id)
    }
}

// Engine state
struct EngineState {
    #[allow(dead_code)] profile: Profile,
    validator: Validator,
    translator: Translator,
    knowledge_store: KnowledgeStore,
}

impl EngineState {
    fn new(profile: Profile) -> Self {
        let validator = Validator::new(&profile);
        let translator = Translator::new(&profile);
        EngineState { profile, validator, translator, knowledge_store: KnowledgeStore::new() }
    }
}

#[tokio::main]
async fn main() {
    let state: Arc<RwLock<EngineState>> =
        Arc::new(RwLock::new(EngineState::new(Profile::default())));
    let (tx, mut rx) = mpsc::unbounded_channel::<String>();

    // Stdout serialiser task
    tokio::spawn(async move {
        let mut writer = BufWriter::new(tokio::io::stdout());
        while let Some(line) = rx.recv().await {
            let _ = writer.write_all(line.as_bytes()).await;
            let _ = writer.write_all(b"\n").await;
            let _ = writer.flush().await;
        }
    });

    // Stdin reader
    let mut lines = BufReader::new(tokio::io::stdin()).lines();
    while let Ok(Some(line)) = lines.next_line().await {
        let trimmed = line.trim().to_owned();
        if trimmed.is_empty() { continue; }
        let state_c = state.clone();
        let tx_c = tx.clone();
        tokio::spawn(async move {
            let resp = match serde_json::from_str::<RpcRequest>(&trimmed) {
                Ok(req) => handle_request(req, &state_c).await,
                Err(e)  => RpcResponse::err(format!("Parse error: {e}"), None),
            };
            if let Ok(json) = serde_json::to_string(&resp) {
                let _ = tx_c.send(json);
            }
        });
    }
}

async fn handle_request(req: RpcRequest, state: &RwLock<EngineState>) -> RpcResponse {
    match req.method.as_str() {
        "ping" => RpcResponse::ok(
            serde_json::json!({"status":"ok","engine":"rust-v2-hnsw"}), req.id),

        "load_profile" => {
            let path = match param_str(&req.params, "path") {
                Ok(p) => p, Err(e) => return RpcResponse::from_engine_err(e, req.id),
            };
            // SECURITY: Prevent path traversal attacks.
            if let Err(e) = verify_safe_path(&path) {
                return RpcResponse::from_engine_err(e, req.id);
            }

            match std::fs::read_to_string(&path) {
                Err(e) => RpcResponse::err(format!("Cannot read '{path}': {e}"), req.id),
                Ok(content) => match Profile::load_any_json_str(&content) {
                    Err(e) => RpcResponse::err(format!("Invalid profile JSON: {e}"), req.id),
                    Ok(profile) if profile.is_empty() => {
                        RpcResponse::err(Profile::EMPTY_MESSAGE.to_string(), req.id)
                    }
                    Ok(profile) => {
                        // Swap rules only; keep the loaded KB / index (was reset by EngineState::new).
                        let mut s = state.write().await;
                        s.validator = Validator::new(&profile);
                        s.translator = Translator::new(&profile);
                        s.profile = profile;
                        RpcResponse::ok(serde_json::json!({"loaded":true,"path":path}), req.id)
                    }
                },
            }
        }

        "load_knowledge_base" => {
            let path = match param_str(&req.params, "path") {
                Ok(p) => p, Err(e) => return RpcResponse::from_engine_err(e, req.id),
            };
            // SECURITY: Prevent path traversal attacks.
            if let Err(e) = verify_safe_path(&path) {
                return RpcResponse::from_engine_err(e, req.id);
            }

            let mut s = state.write().await;
            match s.knowledge_store.load_from_path(&path) {
                Ok(count) => RpcResponse::ok(
                    serde_json::json!({"loaded":true,"count":count,"path":path}), req.id),
                Err(e) => RpcResponse::from_engine_err(e, req.id),
            }
        }

        "build_index" => {
            let mut s = state.write().await;
            let count = s.knowledge_store.len();
            if count == 0 {
                return RpcResponse::err("Knowledge base is empty. Load it first.", req.id);
            }
            s.knowledge_store.build_index();
            RpcResponse::ok(serde_json::json!({"indexed":count}), req.id)
        }

        "save_kb_cache" => {
            let path = match param_str(&req.params, "path") {
                Ok(p) => p, Err(e) => return RpcResponse::from_engine_err(e, req.id),
            };
            // SECURITY: Prevent path traversal attacks.
            if let Err(e) = verify_safe_path(&path) {
                return RpcResponse::from_engine_err(e, req.id);
            }

            let s = state.read().await;
            match s.knowledge_store.save_to_sqlite(&path) {
                Ok(()) => RpcResponse::ok(serde_json::json!({"saved":true,"path":path}), req.id),
                Err(e) => RpcResponse::from_engine_err(e, req.id),
            }
        }

        "load_kb_cache" => {
            let path = match param_str(&req.params, "path") {
                Ok(p) => p, Err(e) => return RpcResponse::from_engine_err(e, req.id),
            };
            // SECURITY: Prevent path traversal attacks.
            if let Err(e) = verify_safe_path(&path) {
                return RpcResponse::from_engine_err(e, req.id);
            }

            let mut s = state.write().await;
            match s.knowledge_store.load_from_sqlite(&path) {
                Ok(count) => RpcResponse::ok(
                    serde_json::json!({"loaded":true,"count":count,"path":path}), req.id),
                Err(e) => RpcResponse::from_engine_err(e, req.id),
            }
        }

        "validate" => {
            let text = req.params.as_ref()
                .and_then(|p| p.get("text")).and_then(|v| v.as_str()).unwrap_or("");
            let s = state.read().await;
            match serde_json::to_value(s.validator.validate(text)) {
                Ok(v) => RpcResponse::ok(v, req.id),
                Err(e) => RpcResponse::err(e.to_string(), req.id),
            }
        }

        "translate" => {
            let text = req.params.as_ref()
                .and_then(|p| p.get("text")).and_then(|v| v.as_str()).unwrap_or("");
            let s = state.read().await;
            match serde_json::to_value(s.translator.match_colloquial(text)) {
                Ok(v) => RpcResponse::ok(v, req.id),
                Err(e) => RpcResponse::err(e.to_string(), req.id),
            }
        }

        "search" => {
            let top_k = req.params.as_ref()
                .and_then(|p| p.get("top_k")).and_then(|v| v.as_u64()).unwrap_or(5) as usize;
            let Some(vec_arr) = req.params.as_ref()
                .and_then(|p| p.get("vector")).and_then(|v| v.as_array()) else {
                return RpcResponse::err("Missing or invalid 'vector' parameter", req.id);
            };
            let query: Vec<f32> = vec_arr.iter()
                .filter_map(|v| v.as_f64().map(|f| f as f32)).collect();
            let s = state.read().await;
            if s.knowledge_store.is_empty() {
                return RpcResponse::err("Knowledge base empty. Call load_knowledge_base first.", req.id);
            }
            let results = s.knowledge_store.search(&query, top_k);
            let json_results: Vec<Value> = results.iter().map(|r| r.to_json_value()).collect();
            let mode = if s.knowledge_store.has_index() { "hnsw" } else { "linear" };
            RpcResponse::ok(serde_json::json!({"results":json_results,"mode":mode}), req.id)
        }

        "kb_info" => {
            let s = state.read().await;
            RpcResponse::ok(serde_json::json!({
                "count": s.knowledge_store.len(),
                "dim":   s.knowledge_store.dim,
                "hnsw_ready": s.knowledge_store.has_index(),
            }), req.id)
        }

        unknown => RpcResponse::err(
            EngineError::UnknownMethod(unknown.to_string()).to_string(), req.id),
    }
}

fn param_str(params: &Option<Value>, key: &str) -> Result<String, EngineError> {
    params.as_ref()
        .and_then(|p| p.get(key))
        .and_then(|v| v.as_str())
        .map(|s| s.to_owned())
        .ok_or_else(|| EngineError::MissingParam(key.to_owned()))
}

/// SECURITY: Basic path safety check to prevent traversal attacks.
/// Disallows absolute paths and parent directory components (`..`).
fn verify_safe_path(path_str: &str) -> Result<(), EngineError> {
    let path = std::path::Path::new(path_str);
    if path.is_absolute() {
        return Err(EngineError::InvalidPath(
            "Absolute paths are not allowed".to_string(),
        ));
    }
    if path.components().any(|c| c == std::path::Component::ParentDir) {
        return Err(EngineError::InvalidPath(
            "Directory traversal ('..') is not allowed".to_string(),
        ));
    }
    Ok(())
}
