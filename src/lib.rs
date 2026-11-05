// Copyright (c) 2026 1abcdefggs
// SPDX-License-Identifier: MIT
// https://github.com/1abcdefggs/vect-or-engine

//! Core engine (no N-API dependency). The Node.js bindings live in `src/napi_bindings.rs`
//! and are compiled only with `--features napi`, so the CLI links without `napi_*` symbols.

#![deny(clippy::all)]

pub mod engine_error;
pub mod limits;
pub mod matrix;
pub mod profile;
pub mod validator;
pub mod translator;
pub mod hnsw_index;
pub mod knowledge_store;

pub use engine_error::EngineError;
pub use hnsw_index::HnswIndex;
pub use matrix::{dot, VectorMatrix};
pub use knowledge_store::{BuiltIndex, IndexJob, KnowledgeItem, KnowledgeStore, NormReport, SearchQuery, SearchResult};
pub use profile::Profile;
pub use translator::{TranslationMatch, Translator};
pub use validator::{normalize_with_offsets, MatchMode, ValidationMarker, ValidationResult, Validator};

#[cfg(feature = "napi")]
mod napi_bindings;
