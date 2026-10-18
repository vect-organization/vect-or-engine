// Copyright (c) 2026 1abcdefggs
// SPDX-License-Identifier: MIT
// https://github.com/1abcdefggs/vect-or-engine

use crate::profile::Profile;
use aho_corasick::AhoCorasick;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TranslationMatch {
    pub item_id: String,
    pub item_name: String,
    pub detected_colloquial: String,
    pub templates: HashMap<String, String>, // Level "A", "B", "C", "D" -> text
}

pub struct Translator {
    ac: Option<AhoCorasick>,
    match_metadata: Vec<(String, String, String, HashMap<String, String>)>, // (item_id, item_name, colloquial_phrase, templates)
}

impl Translator {
    pub fn new(profile: &Profile) -> Self {
        let mut patterns = Vec::new();
        let mut match_metadata = Vec::new();

        if let Some(trans_rules) = &profile.conversational_translation_rules {
            for rule in &trans_rules.rules {
                for pattern in &rule.colloquial_patterns {
                    patterns.push(pattern.clone());
                    match_metadata.push((
                        rule.item_id.clone(),
                        rule.item_name.clone(),
                        pattern.clone(),
                        rule.templates.clone(),
                    ));
                }
            }
        }

        let ac = if !patterns.is_empty() {
            AhoCorasick::builder().build(&patterns).ok()
        } else {
            None
        };

        Translator { ac, match_metadata }
    }

    pub fn match_colloquial(&self, text: &str) -> Vec<TranslationMatch> {
        let mut results = Vec::new();

        if let Some(ac) = &self.ac {
            for mat in ac.find_iter(text) {
                let idx = mat.pattern().as_usize();
                if idx < self.match_metadata.len() {
                    let (item_id, item_name, detected, templates) = &self.match_metadata[idx];
                    results.push(TranslationMatch {
                        item_id: item_id.clone(),
                        item_name: item_name.clone(),
                        detected_colloquial: detected.clone(),
                        templates: templates.clone(),
                    });
                }
            }
        }

        results
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn profile_from(json: &str) -> Profile {
        Profile::load_from_json_str(json).expect("test profile JSON must parse")
    }

    const TWO_RULES: &str = r#"{
        "conversational_translation_rules": {
            "rules": [
                {"item_id": "A1", "item_name": "greeting",
                 "colloquial_patterns": ["ちょっと"], "templates": {"A": "a1"}},
                {"item_id": "B2", "item_name": "agreement",
                 "colloquial_patterns": ["やばい", "まじ"], "templates": {"A": "b2"}}
            ]
        }
    }"#;

    #[test]
    fn matches_colloquial_phrase_with_its_item() {
        let t = Translator::new(&profile_from(TWO_RULES));
        let hits = t.match_colloquial("今日はちょっと疲れた");
        assert_eq!(hits.len(), 1);
        assert_eq!(hits[0].item_id, "A1");
        assert_eq!(hits[0].item_name, "greeting");
        assert_eq!(hits[0].detected_colloquial, "ちょっと");
        assert_eq!(hits[0].templates.get("A").map(String::as_str), Some("a1"));
    }

    #[test]
    fn no_phrase_means_no_match() {
        let t = Translator::new(&profile_from(TWO_RULES));
        assert!(t.match_colloquial("丁寧な表現だけの文").is_empty());
    }

    #[test]
    fn each_pattern_maps_to_its_own_rule_across_rules() {
        // Index mapping regression: patterns from the second rule must not
        // resolve to the first rule's item_id.
        let t = Translator::new(&profile_from(TWO_RULES));
        let hits = t.match_colloquial("まじやばい");
        assert_eq!(hits.len(), 2);
        assert!(hits.iter().all(|h| h.item_id == "B2"));
        let phrases: Vec<&str> = hits.iter().map(|h| h.detected_colloquial.as_str()).collect();
        assert!(phrases.contains(&"まじ"));
        assert!(phrases.contains(&"やばい"));
    }

    #[test]
    fn same_phrase_found_multiple_times_is_reported_each_time() {
        let t = Translator::new(&profile_from(TWO_RULES));
        let hits = t.match_colloquial("ちょっと、ちょっと");
        assert_eq!(hits.len(), 2);
    }

    #[test]
    fn profile_without_translation_rules_matches_nothing() {
        let t = Translator::new(&profile_from("{}"));
        assert!(t.match_colloquial("ちょっと").is_empty());
    }

    #[test]
    fn empty_rule_list_builds_no_automaton() {
        let t = Translator::new(&profile_from(
            r#"{"conversational_translation_rules": {"rules": []}}"#,
        ));
        assert!(t.ac.is_none());
        assert!(t.match_colloquial("ちょっと").is_empty());
    }
}
