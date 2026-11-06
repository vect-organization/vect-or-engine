// Copyright (c) 2026 1abcdefggs
// SPDX-License-Identifier: MIT
// https://github.com/1abcdefggs/vect-or-engine

#![forbid(unsafe_code)]

//! One flat f32 matrix and the one dot product every search path uses
//! (change-log rows P3, P4, R5, A3).
//!
//! - **P3**: the knowledge base keeps its vectors in a single contiguous `Vec<f32>`
//!   (row-major). The exact scan and the HNSW index share it through an `Arc`, so a vector is
//!   stored once, as exact f32. f16 survives only as the SQLite cache format.
//! - **P4**: `dot` is safe Rust, with sixteen independent accumulators over fixed-size chunks.
//!   There is no `unsafe` and no bounds check in the hot loop; the compiler vectorises it at the
//!   build target's baseline.
//! - **R5**: the summation order is fixed and written down on `dot`, and the exact scan and the
//!   index both call it, so the same two vectors give the same score on every path and CPU.

use std::sync::Arc;

use crate::engine_error::EngineError;

/// Number of independent accumulators in [`dot`].
pub const LANES: usize = 16;

/// Dot product of two vectors; the shorter length decides how many components are used.
///
/// Summation order (fixed, R5): component `i` of each full 16-wide chunk is added to
/// accumulator `i % 16`, chunk after chunk; then the 16 accumulators are added to a running
/// sum in order 0..15; then the remaining tail components (fewer than 16) are added one by
/// one in index order.
#[inline(always)]
pub fn dot(a: &[f32], b: &[f32]) -> f32 {
    let n = a.len().min(b.len());
    let (a, b) = (&a[..n], &b[..n]);
    let mut acc = [0.0f32; LANES];
    let mut ca = a.chunks_exact(LANES);
    let mut cb = b.chunks_exact(LANES);
    for (x, y) in (&mut ca).zip(&mut cb) {
        for k in 0..LANES {
            acc[k] += x[k] * y[k];
        }
    }
    let mut sum = 0.0f32;
    for v in acc {
        sum += v;
    }
    for (x, y) in ca.remainder().iter().zip(cb.remainder()) {
        sum += x * y;
    }
    sum
}

/// Row-major matrix of equal-length f32 vectors.
#[derive(Debug, Clone, Default)]
pub struct VectorMatrix {
    dim: usize,
    data: Vec<f32>,
}

impl VectorMatrix {
    /// Empty matrix with a fixed row length.
    pub fn new(dim: usize) -> Self {
        VectorMatrix { dim, data: Vec::new() }
    }

    /// Empty matrix with room for `rows` rows of `dim` components.
    pub fn with_capacity(dim: usize, rows: usize) -> Self {
        VectorMatrix { dim, data: Vec::with_capacity(dim * rows) }
    }

    /// Wrap an existing flat buffer. `data.len()` must be a multiple of `dim`.
    pub fn from_flat(dim: usize, data: Vec<f32>) -> Result<Self, EngineError> {
        if dim == 0 && !data.is_empty() {
            return Err(EngineError::InvalidInput("a matrix with dimension 0 cannot hold values".into()));
        }
        if dim != 0 && !data.len().is_multiple_of(dim) {
            return Err(EngineError::InvalidInput(format!(
                "{} values do not fill whole rows of {dim} components", data.len()
            )));
        }
        Ok(VectorMatrix { dim, data })
    }

    /// Build from separate rows; every row must have the length of the first one.
    pub fn from_rows(rows: &[Vec<f32>]) -> Result<Self, EngineError> {
        let dim = rows.first().map_or(0, Vec::len);
        let mut m = VectorMatrix::with_capacity(dim, rows.len());
        for row in rows {
            m.push_row(row)?;
        }
        Ok(m)
    }

    /// Append one row. The first row of an empty, dimensionless matrix sets the dimension.
    pub fn push_row(&mut self, row: &[f32]) -> Result<(), EngineError> {
        if self.dim == 0 && self.data.is_empty() {
            self.dim = row.len();
        }
        if row.len() != self.dim {
            return Err(EngineError::DimensionMismatch { expected: self.dim, got: row.len() });
        }
        self.data.extend_from_slice(row);
        Ok(())
    }

    /// Components per row (0 for an empty matrix without a dimension).
    pub fn dim(&self) -> usize {
        self.dim
    }

    /// Number of rows.
    pub fn len(&self) -> usize {
        self.data.len().checked_div(self.dim).unwrap_or(0)
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Row `i`. Panics when `i >= len()`, like slice indexing.
    #[inline(always)]
    pub fn row(&self, i: usize) -> &[f32] {
        &self.data[i * self.dim..(i + 1) * self.dim]
    }

    /// Every row in order.
    pub fn rows(&self) -> impl Iterator<Item = &[f32]> {
        self.data.chunks_exact(self.dim.max(1))
    }

    /// Share the matrix without copying it.
    pub fn into_shared(self) -> Arc<VectorMatrix> {
        Arc::new(self)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn safe_dot_matches_a_plain_sum() {
        let q: Vec<f32> = (0..21).map(|i| i as f32 * 0.5).collect();
        let s: Vec<f32> = (0..21).map(|i| 1.0 + i as f32).collect();
        let want: f32 = q.iter().zip(&s).map(|(a, b)| a * b).sum();
        assert!((dot(&q, &s) - want).abs() < 1e-3);
    }

    /// The documented order, written out with plain loops.
    fn reference_dot(a: &[f32], b: &[f32]) -> f32 {
        let n = a.len().min(b.len());
        let full = n / LANES * LANES;
        let mut acc = [0.0f32; LANES];
        for i in 0..full {
            acc[i % LANES] += a[i] * b[i];
        }
        let mut sum = 0.0f32;
        for v in acc {
            sum += v;
        }
        for i in full..n {
            sum += a[i] * b[i];
        }
        sum
    }

    #[test]
    fn r5_dot_uses_one_fixed_summation_order() {
        // Components of very different size make any change of order visible in the last bits.
        for n in [0usize, 1, 15, 16, 17, 31, 384, 385] {
            let a: Vec<f32> = (0..n).map(|i| ((i * 7919 % 1013) as f32 - 500.0) * 1.0e-3 + 1.0e4 * (i % 3) as f32).collect();
            let b: Vec<f32> = (0..n).map(|i| ((i * 104_729 % 997) as f32 - 400.0) * 1.0e-2).collect();
            assert_eq!(dot(&a, &b).to_bits(), reference_dot(&a, &b).to_bits(), "n = {n}");
            assert_eq!(dot(&a, &b).to_bits(), dot(&b, &a).to_bits(), "dot must be symmetric, n = {n}");
        }
    }

    #[test]
    fn matrix_rows_and_shape_checks() {
        let m = VectorMatrix::from_rows(&[vec![1.0, 2.0], vec![3.0, 4.0]]).unwrap();
        assert_eq!((m.len(), m.dim()), (2, 2));
        assert_eq!(m.row(1), &[3.0, 4.0]);
        assert_eq!(m.rows().count(), 2);
        assert!(VectorMatrix::from_rows(&[vec![1.0, 2.0], vec![3.0]]).is_err());
        assert!(VectorMatrix::from_flat(3, vec![1.0; 7]).is_err());
        assert!(VectorMatrix::default().is_empty());
    }
}
