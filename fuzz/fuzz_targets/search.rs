#![no_main]
//! Search: arbitrary vectors and a query must never panic, and the answer is bounded by top_k.
use libfuzzer_sys::fuzz_target;
use vect_or_engine::HnswIndex;

fuzz_target!(|data: &[u8]| {
    if data.len() < 3 {
        return;
    }
    let dim = (data[0] % 8) as usize + 1;
    let top_k = data[1] as usize % 16;
    let floats: Vec<f32> = data[2..]
        .chunks_exact(4)
        .map(|c| f32::from_le_bytes([c[0], c[1], c[2], c[3]]))
        .collect();
    let rows: Vec<Vec<f32>> = floats.chunks_exact(dim).map(|c| c.to_vec()).collect();
    if rows.is_empty() {
        return;
    }
    if let Ok(index) = HnswIndex::build(&rows) {
        let hits = index.search(&rows[0], top_k);
        assert!(hits.len() <= top_k);
    }
});
