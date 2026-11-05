// Copyright (c) 2026 1abcdefggs
// SPDX-License-Identifier: MIT
// https://github.com/1abcdefggs/vect-or-engine

//! Input limits (change-log item S4). One place for every bound, so a crafted profile or
//! dictionary cannot make the engine stall or exhaust memory. The values are generous for
//! real dictionaries (the largest today has about 2,000 items and 1,000 rules).

#![forbid(unsafe_code)]

/// Largest profile / guideline JSON accepted (bytes).
pub const MAX_PROFILE_BYTES: usize = 16 * 1024 * 1024;
/// Largest knowledge-base JSON accepted (bytes).
pub const MAX_KB_BYTES: u64 = 1024 * 1024 * 1024;
/// Most keyword rules plus conversational rules in one profile.
pub const MAX_RULES: usize = 10_000;
/// Most patterns (trigger words, conflict items, colloquial patterns) in one profile.
pub const MAX_PATTERNS: usize = 100_000;
/// Longest single pattern or regex (bytes).
pub const MAX_PATTERN_BYTES: usize = 4096;
/// Compiled-regex size limit (bytes), passed to `RegexBuilder::size_limit`.
pub const MAX_REGEX_SIZE: usize = 1024 * 1024;
/// Largest `top_k` a search honours; larger requests are capped.
pub const MAX_TOP_K: usize = 1000;
/// Largest absolute vector component that survives f16 storage (f16::MAX).
pub const F16_MAX_ABS: f32 = 65504.0;
