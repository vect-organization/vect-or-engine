// Copyright (c) 2026 1abcdefggs
// SPDX-License-Identifier: MIT
// https://github.com/1abcdefggs/vect-or-engine

use crate::profile::Profile;
use aho_corasick::AhoCorasick;
use regex::Regex;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationMarker {
    pub severity: String, // "Error" | "Warning" | "Info"
    pub message: String,
    pub line: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationResult {
    pub is_valid: bool,
    pub extracted_value: Option<u32>,
    pub has_keyword_conflict: bool,
    pub keyword_message: Option<String>,
    pub value_message: Option<String>,
    /// SmallVec: zero heap allocation for ≤4 markers (typical case).
    pub markers: Vec<ValidationMarker>,
}

/// How keyword-rule `trigger_words` are matched against the document (ADR-0006).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum MatchMode {
    /// Plain substring match (the behaviour before ADR-0006).
    #[default]
    Literal,
    /// The trigger word is a regular expression.
    Regex,
    /// A trigger word matches if either the literal or the regex form matches (OR).
    Both,
}

impl MatchMode {
    /// Parses the mode name used across the N-API boundary. Unknown names return `None`.
    pub fn parse(name: &str) -> Option<Self> {
        match name {
            "literal" => Some(Self::Literal),
            "regex" => Some(Self::Regex),
            "both" => Some(Self::Both),
            _ => None,
        }
    }
}

/// One compiled trigger word. Regexes are compiled once at load time.
enum Matcher {
    Literal(String),
    Regex(Regex),
    Both(String, Regex),
}

impl Matcher {
    fn compile(word: &str, mode: MatchMode) -> Result<Self, String> {
        let regex = || {
            Regex::new(word).map_err(|e| format!("invalid regex '{word}' in keyword rule: {e}"))
        };
        Ok(match mode {
            MatchMode::Literal => Matcher::Literal(word.to_string()),
            MatchMode::Regex => Matcher::Regex(regex()?),
            // `both` must not reject a word that is only a valid literal (e.g. "C++", "("):
            // if it is not a valid regex, it simply matches as literal text.
            MatchMode::Both => match regex() {
                Ok(re) => Matcher::Both(word.to_string(), re),
                Err(_) => Matcher::Literal(word.to_string()),
            },
        })
    }

    /// Byte offset of the first match, if any. Used for the marker's line number.
    fn find(&self, text: &str) -> Option<usize> {
        match self {
            Matcher::Literal(s) => text.find(s.as_str()),
            Matcher::Regex(re) => re.find(text).map(|m| m.start()),
            Matcher::Both(s, re) => {
                let lit = text.find(s.as_str());
                let rx = re.find(text).map(|m| m.start());
                match (lit, rx) {
                    (Some(a), Some(b)) => Some(a.min(b)),
                    (a, b) => a.or(b),
                }
            }
        }
    }
}

struct CompiledKeywordRule {
    warning_message: String,
    matchers: Vec<Matcher>,
}

pub struct Validator {
    value_regex: Option<Regex>,
    value_warning_template: Option<String>,
    conflict_items_ac: Option<AhoCorasick>,
    conflict_item_reasons: Vec<String>,
    min_value: u32,
    keyword_rules: Vec<CompiledKeywordRule>,
}

impl Validator {
    /// Literal matching. Cannot fail: literal trigger words are never compiled.
    pub fn new(profile: &Profile) -> Self {
        Self::with_mode(profile, MatchMode::Literal)
            .expect("literal keyword matching never fails to compile")
    }

    /// Builds the validator with the given keyword matching mode. Fails on an invalid regex,
    /// so the caller can keep the previously loaded rules.
    pub fn with_mode(profile: &Profile, mode: MatchMode) -> Result<Self, String> {
        let mut patterns = Vec::new();
        let mut reasons = Vec::new();
        let mut min_value = 0;
        let mut value_regex = None;
        let mut value_warning_template = None;
        let mut keyword_rules = Vec::new();

        if let Some(rules) = &profile.assessment_rules {
            if let Some(template) = &rules.document_template {
                if let Some(reqs) = &template.numerical_requirements {
                    if let Some(min_v) = reqs.min_value {
                        min_value = min_v;
                    }
                    if let Some(pattern) = &reqs.value_regex_pattern {
                        if let Ok(re) = Regex::new(pattern) {
                            value_regex = Some(re);
                        }
                    }
                    if let Some(warning_template) = &reqs.value_warning_template {
                        value_warning_template = Some(warning_template.clone());
                    }
                    if let Some(conflicts) = &reqs.conflict_items {
                        for cd in conflicts {
                            patterns.push(cd.item_code.clone());
                            patterns.push(cd.name.clone());
                            reasons.push(format!("{}: {}", cd.name, cd.reason));
                            reasons.push(format!("{}: {}", cd.name, cd.reason));
                        }
                    }
                }
            }
            if let Some(keywords) = &rules.keyword_rules {
                for rule in keywords {
                    let matchers = rule
                        .trigger_words
                        .iter()
                        .map(|w| Matcher::compile(w, mode))
                        .collect::<Result<Vec<_>, _>>()?;
                    keyword_rules.push(CompiledKeywordRule {
                        warning_message: rule.warning_message.clone(),
                        matchers,
                    });
                }
            }
        }

        let conflict_items_ac = if !patterns.is_empty() {
            AhoCorasick::builder().build(&patterns).ok()
        } else {
            None
        };

        Ok(Validator {
            value_regex,
            value_warning_template,
            conflict_items_ac,
            conflict_item_reasons: reasons,
            min_value,
            keyword_rules,
        })
    }

    pub fn validate(&self, text: &str) -> ValidationResult {
        // Pre-compute line start offsets once, reused for all markers.
        let line_starts: Vec<usize> = std::iter::once(0)
            .chain(text.char_indices().filter(|(_, c)| *c == '\n').map(|(i, _)| i + 1))
            .collect();

        // Map a byte offset to a 1-based line number.
        let offset_to_line = |offset: usize| -> u32 {
            line_starts.partition_point(|&s| s <= offset) as u32
        };

        let mut markers: Vec<ValidationMarker> = Vec::new();
        let mut extracted_value = None;
        let mut value_message = None;
        let mut has_keyword_conflict = false;
        let mut keyword_message = None;

        // 1. Extract numerical value
        if let Some(re) = &self.value_regex {
            if let Some(caps) = re.captures(text) {
                if let Some(m) = caps.get(1) {
                    if let Ok(val) = m.as_str().parse::<u32>() {
                        extracted_value = Some(val);
                        if val < self.min_value {
                            let msg = if let Some(template) = &self.value_warning_template {
                                template.replace("{min}", &self.min_value.to_string())
                                        .replace("{actual}", &val.to_string())
                            } else {
                                format!("Value requirement not met (Required: {}, Actual: {})", self.min_value, val)
                            };
                            value_message = Some(msg.clone());
                            let line = offset_to_line(m.start());
                            markers.push(ValidationMarker {
                                severity: "Warning".to_string(),
                                message: msg,
                                line,
                            });
                        }
                    }
                }
            }
        }

        // 2. Conflict item check using Aho-Corasick (O(N) search)
        if let Some(ac) = &self.conflict_items_ac {
            for mat in ac.find_iter(text) {
                let pattern_idx = mat.pattern().as_usize();
                if pattern_idx < self.conflict_item_reasons.len() {
                    let reason = &self.conflict_item_reasons[pattern_idx];
                    let msg = format!("Conflict detected: {}", reason);
                    let line = offset_to_line(mat.start());
                    markers.push(ValidationMarker {
                        severity: "Warning".to_string(),
                        message: msg,
                        line,
                    });
                }
            }
        }

        // 3. Keyword Rule Check (Dynamic)
        for rule in &self.keyword_rules {
            if !rule.matchers.is_empty() && rule.matchers.iter().all(|m| m.find(text).is_some()) {
                has_keyword_conflict = true;
                let msg = rule.warning_message.clone();
                keyword_message = Some(msg.clone());
                // Find the line of the first trigger word.
                let line = rule.matchers.first()
                    .and_then(|m| m.find(text))
                    .map(|o| offset_to_line(o))
                    .unwrap_or(1);
                markers.push(ValidationMarker {
                    severity: "Error".to_string(),
                    message: msg,
                    line,
                });
            }
        }

        let is_valid = markers.iter().all(|m| m.severity != "Error");

        ValidationResult {
            is_valid,
            extracted_value,
            has_keyword_conflict,
            keyword_message,
            value_message,
            markers,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::profile::Profile;

    #[test]
    fn test_validation_duration_and_conflict() {
        let json_data = r#"{
            "assessment_rules": {
                "document_template": {
                    "numerical_requirements": {
                        "min_value": 50,
                        "value_regex_pattern": "Budget: ([0-9]+)K",
                        "value_warning_template": "Budget too low. Required: {min}K, Actual: {actual}K.",
                        "conflict_items": [
                            {
                                "item_code": "VEN-001",
                                "name": "Banned Vendor",
                                "reason": "This vendor is blocked by compliance."
                            }
                        ]
                    }
                },
                "keyword_rules": [
                    {
                        "trigger_words": ["Confidential", "Public Leak"],
                        "warning_message": "Confidentiality breach detected."
                    }
                ]
            }
        }"#;

        let profile = Profile::load_from_json_str(json_data).unwrap();
        let validator = Validator::new(&profile);

        // Test numerical warning
        let res1 = validator.validate("The proposed Budget: 40K is ready.");
        assert_eq!(res1.extracted_value, Some(40));
        assert!(res1.value_message.as_deref().unwrap().contains("Required: 50K"));

        // Test conflict item warning
        let res2 = validator.validate("We will use Banned Vendor for this.");
        assert!(!res2.markers.is_empty());
        assert!(res2.markers[0].message.contains("Banned Vendor"));

        // Test keyword warning
        let res3 = validator.validate("This is Confidential and causes a Public Leak.");
        assert!(!res3.is_valid);
        assert!(res3.has_keyword_conflict);
        assert!(res3.keyword_message.as_deref().unwrap().contains("breach"));
    }

    /// One keyword rule whose single trigger word is `word` (given as raw JSON-escaped text).
    fn keyword_profile(json_word: &str) -> Profile {
        Profile::load_from_json_str(&format!(
            r#"{{"assessment_rules":{{"keyword_rules":[{{"trigger_words":["{json_word}"],"warning_message":"hit"}}]}}}}"#
        ))
        .expect("test profile JSON must parse")
    }

    #[test]
    fn match_mode_parse_accepts_only_known_names() {
        assert_eq!(MatchMode::parse("literal"), Some(MatchMode::Literal));
        assert_eq!(MatchMode::parse("regex"), Some(MatchMode::Regex));
        assert_eq!(MatchMode::parse("both"), Some(MatchMode::Both));
        assert_eq!(MatchMode::parse("Regex"), None);
        assert_eq!(MatchMode::parse(""), None);
        assert_eq!(MatchMode::default(), MatchMode::Literal);
    }

    #[test]
    fn literal_mode_treats_metacharacters_as_plain_text() {
        // "." is an ordinary character here, so "G91" must not hit "G.1".
        let v = Validator::with_mode(&keyword_profile("G.1"), MatchMode::Literal).unwrap();
        assert!(v.validate("code G.1 here").has_keyword_conflict);
        assert!(!v.validate("code G91 here").has_keyword_conflict);
    }

    #[test]
    fn regex_mode_matches_pattern_and_reports_its_line() {
        // \d+ does not occur literally in the text, so only the regex form can hit.
        let v = Validator::with_mode(&keyword_profile(r"G\\d+"), MatchMode::Regex).unwrap();
        let res = v.validate("intro line\nsee G42 for details");
        assert!(res.has_keyword_conflict);
        assert_eq!(res.markers[0].line, 2);
        assert!(!v.validate("no code here").has_keyword_conflict);
    }

    #[test]
    fn regex_mode_dot_matches_any_character() {
        // In regex mode "." is a wildcard, so "abc" hits. Literal mode would miss it.
        let v = Validator::with_mode(&keyword_profile("a.c"), MatchMode::Regex).unwrap();
        assert!(v.validate("abc").has_keyword_conflict);
        let lit = Validator::with_mode(&keyword_profile("a.c"), MatchMode::Literal).unwrap();
        assert!(!lit.validate("abc").has_keyword_conflict);
    }

    #[test]
    fn both_mode_matches_when_either_form_matches() {
        let v = Validator::with_mode(&keyword_profile("a.c"), MatchMode::Both).unwrap();
        assert!(v.validate("abc").has_keyword_conflict, "regex side should hit");
        assert!(v.validate("a.c").has_keyword_conflict, "literal side should hit");
        assert!(!v.validate("ac").has_keyword_conflict, "neither side matches");
    }

    #[test]
    fn both_mode_accepts_words_that_are_only_valid_literals() {
        // "C++" and "(" are not valid regexes, but they must still match as literal text.
        let cpp = Validator::with_mode(&keyword_profile("C++"), MatchMode::Both)
            .expect("both mode must accept a non-regex literal");
        assert!(cpp.validate("we use C++ here").has_keyword_conflict);

        let paren = Validator::with_mode(&keyword_profile("("), MatchMode::Both)
            .expect("both mode must accept a lone parenthesis");
        assert!(paren.validate("see (note)").has_keyword_conflict);
    }

    #[test]
    fn invalid_regex_is_rejected_at_build_time() {
        let err = Validator::with_mode(&keyword_profile("("), MatchMode::Regex)
            .err()
            .expect("unbalanced parenthesis must fail to compile");
        assert!(err.contains("invalid regex"), "message: {err}");
        // Literal mode never compiles trigger words, so the same text is accepted there.
        assert!(Validator::with_mode(&keyword_profile("("), MatchMode::Literal).is_ok());
    }
}
