#![no_main]
//! Rules: arbitrary profile text and document text must never panic in any match mode.
use libfuzzer_sys::fuzz_target;
use vect_or_engine::{MatchMode, Profile, Validator};

fuzz_target!(|data: &[u8]| {
    let Ok(text) = std::str::from_utf8(data) else { return };
    // The first line is the profile, the rest is the document.
    let (profile_json, document) = text.split_once('\n').unwrap_or((text, ""));
    let Ok(profile) = Profile::load_any_json_str(profile_json) else { return };
    for mode in [MatchMode::default(), MatchMode::parse("regex").unwrap(), MatchMode::parse("both").unwrap()] {
        if let Ok(validator) = Validator::with_mode(&profile, mode) {
            let _ = validator.validate(document);
        }
    }
});
