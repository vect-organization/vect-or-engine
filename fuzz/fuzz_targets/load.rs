#![no_main]
//! Load: arbitrary bytes as a knowledge-base file must give Ok or Err, never a panic.
use libfuzzer_sys::fuzz_target;
use vect_or_engine::KnowledgeStore;

fuzz_target!(|data: &[u8]| {
    let path = std::env::temp_dir().join(format!("vect-fuzz-load-{}.json", std::process::id()));
    if std::fs::write(&path, data).is_err() {
        return;
    }
    let mut store = KnowledgeStore::new();
    let _ = store.load_from_path(&path.to_string_lossy());
    let _ = std::fs::remove_file(&path);
});
