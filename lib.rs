// Copyright (c) 2026 1abcdefggs
// SPDX-License-Identifier: MIT
// https://github.com/1abcdefggs/vect-or-engine

//! Core engine (no N-API dependency). The Node.js bindings live in `src/napi_bindings.rs`
//! and are compiled only with `--features napi`, so the CLI links without `napi_*` symbols.

#![deny(clippy::all)]

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

#[cfg(feature = "napi")]
#[path = "src/napi_bindings.rs"]
mod napi_bindings;
