// Copyright (c) 2026 1abcdefggs
// SPDX-License-Identifier: MIT
// https://github.com/1abcdefggs/vect-or-engine

#![forbid(unsafe_code)]

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Profile {
    pub metadata_standards: Option<MetadataStandards>,
    pub assessment_rules: Option<AssessmentRules>,
    pub conversational_translation_rules: Option<ConversationalTranslationRules>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct MetadataStandards {
    pub authority: Option<String>,
    pub version: Option<String>,
    pub description: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AssessmentRules {
    pub document_template: Option<DocumentTemplate>,
    pub keyword_rules: Option<Vec<KeywordRule>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct KeywordRule {
    pub trigger_words: Vec<String>,
    pub warning_message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct DocumentTemplate {
    pub numerical_requirements: Option<NumericalRequirements>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct NumericalRequirements {
    pub target_category: Option<String>,
    pub min_value: Option<u32>,
    pub value_regex_pattern: Option<String>,
    pub value_warning_template: Option<String>,
    pub conflict_items: Option<Vec<ConflictItem>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ConflictItem {
    pub item_code: String,
    pub name: String,
    pub reason: String,
    pub max_allowed_value: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ConversationalTranslationRules {
    pub description: Option<String>,
    pub rules: Vec<ConversationalRule>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ConversationalRule {
    pub item_id: String,
    pub item_name: String,
    pub colloquial_patterns: Vec<String>,
    pub templates: HashMap<String, String>, // "A", "B", "C", "D" -> text
}

impl Profile {
    /// Error text shared by the N-API and CLI loaders when `is_empty()` holds.
    pub const EMPTY_MESSAGE: &'static str =
        "Profile parsed but contains no rules (unsupported shape?)";

    pub fn load_from_json_str(json_str: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(json_str)
    }

    /// Format-detecting loader (ADR-0005 Q5: Engine Profile is Module 0, other shapes are
    /// adapted into it). A document with a top-level `rules` array and a `domain` key is a
    /// vect-or-generate guideline (guideline.schema.json) and goes through
    /// `from_guideline_json_str`; anything else is parsed as a native Profile. Without this,
    /// guideline files deserialize into an all-None Profile with no error and no rule fires.
    pub fn load_any_json_str(json_str: &str) -> Result<Self, serde_json::Error> {
        use serde::de::Error as _;
        if json_str.len() > crate::limits::MAX_PROFILE_BYTES {
            return Err(serde_json::Error::custom(format!(
                "profile is larger than {} bytes",
                crate::limits::MAX_PROFILE_BYTES
            )));
        }
        let value: serde_json::Value = serde_json::from_str(json_str)?;
        let is_guideline = value.get("rules").is_some_and(|r| r.is_array())
            && value.get("domain").is_some();
        let profile = if is_guideline {
            // A guideline that fails to convert is an error, not a silent fall-back to an
            // empty native profile.
            Self::from_guideline_json_str(json_str)?
        } else {
            serde_json::from_value(value)?
        };
        profile.validate().map_err(serde_json::Error::custom)?;
        Ok(profile)
    }

    /// Checks a loaded profile before any matcher is built (change-log D3, D7, S4):
    /// no empty or oversized patterns, a bounded number of rules, and a value regex that
    /// compiles within the size limit. Errors name the offending rule.
    pub fn validate(&self) -> Result<(), String> {
        use crate::limits::{MAX_PATTERNS, MAX_PATTERN_BYTES, MAX_REGEX_SIZE, MAX_RULES};
        let mut patterns = 0usize;
        let mut rules = 0usize;
        let mut check = |what: &str, text: &str| -> Result<(), String> {
            if text.is_empty() {
                return Err(format!("empty pattern in {what}"));
            }
            if text.len() > MAX_PATTERN_BYTES {
                return Err(format!("pattern in {what} is longer than {MAX_PATTERN_BYTES} bytes"));
            }
            patterns += 1;
            if patterns > MAX_PATTERNS {
                return Err(format!("more than {MAX_PATTERNS} patterns in the profile"));
            }
            Ok(())
        };

        if let Some(a) = &self.assessment_rules {
            for (i, rule) in a.keyword_rules.iter().flatten().enumerate() {
                rules += 1;
                if rule.trigger_words.is_empty() {
                    return Err(format!("keyword rule #{i} has no trigger words"));
                }
                for w in &rule.trigger_words {
                    check(&format!("keyword rule #{i} ('{}')", rule.warning_message), w)?;
                }
            }
            if let Some(req) = a.document_template.as_ref().and_then(|t| t.numerical_requirements.as_ref()) {
                for (i, c) in req.conflict_items.iter().flatten().enumerate() {
                    check(&format!("conflict item #{i} item_code"), &c.item_code)?;
                    check(&format!("conflict item #{i} ('{}') name", c.item_code), &c.name)?;
                }
                if let Some(pat) = &req.value_regex_pattern {
                    if pat.len() > MAX_PATTERN_BYTES {
                        return Err(format!("value_regex_pattern is longer than {MAX_PATTERN_BYTES} bytes"));
                    }
                    regex::RegexBuilder::new(pat)
                        .size_limit(MAX_REGEX_SIZE)
                        .build()
                        .map_err(|e| format!("invalid value_regex_pattern '{pat}': {e}"))?;
                }
            }
        }
        if let Some(c) = &self.conversational_translation_rules {
            for rule in &c.rules {
                rules += 1;
                for p in &rule.colloquial_patterns {
                    check(&format!("conversational rule '{}'", rule.item_id), p)?;
                }
            }
        }
        if rules > MAX_RULES {
            return Err(format!("more than {MAX_RULES} rules in the profile"));
        }
        Ok(())
    }

    /// True when no section carries any rule (used to surface silent no-op profiles).
    pub fn is_empty(&self) -> bool {
        let no_kw = self
            .assessment_rules
            .as_ref()
            .and_then(|a| a.keyword_rules.as_ref())
            .is_none_or(|k| k.is_empty());
        // Destructuring (not field access) so that adding a field to DocumentTemplate
        // fails to compile here until is_empty is updated.
        let no_num = match self.assessment_rules.as_ref().and_then(|a| a.document_template.as_ref()) {
            Some(DocumentTemplate { numerical_requirements }) => numerical_requirements.is_none(),
            None => true,
        };
        let no_conv = self
            .conversational_translation_rules
            .as_ref()
            .is_none_or(|c| c.rules.is_empty());
        no_kw && no_num && no_conv
    }

    /// Adapter for vect-or-generate's `guideline_*.json` output
    /// (schemas/guideline.schema.json: domain/version/rules[]).
    /// `Profile::load_from_json_str` alone silently returns an empty Profile
    /// for this shape (field names don't match, all Profile fields are
    /// Option so serde just leaves them None with no error) — ADR-0005
    /// Phase1.3. This adapter maps rules[] by severity into the matching
    /// G.rule/G.policy-equivalent Profile sub-structures instead.
    pub fn from_guideline_json_str(json_str: &str) -> Result<Self, serde_json::Error> {
        let raw: GuidelineDoc = serde_json::from_str(json_str)?;
        let mut keyword_rules = Vec::new();
        let mut conversational_rules = Vec::new();

        for rule in raw.rules {
            use serde::de::Error as _;
            // Severity is case-insensitive; an unknown value is rejected, not guessed.
            let severity = rule.severity.trim().to_ascii_lowercase();
            if !matches!(severity.as_str(), "error" | "warning" | "info" | "hint") {
                return Err(serde_json::Error::custom(format!(
                    "rule '{}' has unknown severity '{}' (expected error, warning, info or hint)",
                    rule.id, rule.severity
                )));
            }
            match severity.as_str() {
                // severity: error -> G.rule (strict enforcement)
                "error" => {
                    keyword_rules.push(KeywordRule {
                        trigger_words: vec![rule.pattern.clone()],
                        warning_message: rule.message.clone(),
                    });
                }
                // severity: warning/info/hint -> G.policy (abstract guidance)
                _ => {
                    let mut templates = HashMap::new();
                    templates.insert(
                        "A".to_string(),
                        rule.suggest.clone().unwrap_or_else(|| rule.message.clone()),
                    );
                    conversational_rules.push(ConversationalRule {
                        item_id: rule.id.clone(),
                        item_name: rule.category.clone().unwrap_or_default(),
                        colloquial_patterns: vec![rule.pattern.clone()],
                        templates,
                    });
                }
            }
        }

        Ok(Profile {
            metadata_standards: Some(MetadataStandards {
                authority: Some("vect-or-generate".to_string()),
                version: Some(raw.version),
                description: raw.description,
            }),
            assessment_rules: Some(AssessmentRules {
                document_template: None,
                keyword_rules: if keyword_rules.is_empty() { None } else { Some(keyword_rules) },
            }),
            conversational_translation_rules: if conversational_rules.is_empty() {
                None
            } else {
                Some(ConversationalTranslationRules {
                    description: Some(format!("Converted from guideline domain: {}", raw.domain)),
                    rules: conversational_rules,
                })
            },
        })
    }
}

/// Mirrors schemas/guideline.schema.json (vect-or-generate). Kept separate
/// from `Profile` because the two shapes are not field-compatible.
#[derive(Debug, Clone, Deserialize)]
struct GuidelineDoc {
    domain: String,
    version: String,
    #[serde(default)]
    description: Option<String>,
    #[serde(default)]
    rules: Vec<GuidelineRule>,
}

#[derive(Debug, Clone, Deserialize)]
struct GuidelineRule {
    id: String,
    #[serde(default)]
    category: Option<String>,
    pattern: String,
    message: String,
    severity: String,
    #[serde(default)]
    suggest: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    const GUIDELINE: &str = r#"{
        "domain": "sample", "version": "0.3.5", "schemaVersion": "0.4.0",
        "rules": [
            {"id":"R1","category":"terminology","pattern":"BANNED_TERM","message":"banned","severity":"error","suggest":"PREFERRED_TERM"},
            {"id":"R2","category":"style","pattern":"WEAK_PHRASE","message":"weak","severity":"warning","suggest":"STRONG_PHRASE"}
        ]
    }"#;

    #[test]
    fn native_loader_silently_drops_guideline_shape() {
        // Documents the original bug: no error, but nothing loaded.
        let p = Profile::load_from_json_str(GUIDELINE).unwrap();
        assert!(p.is_empty());
    }

    #[test]
    fn any_loader_adapts_guideline_by_severity() {
        let p = Profile::load_any_json_str(GUIDELINE).unwrap();
        let kw = p.assessment_rules.unwrap().keyword_rules.unwrap();
        assert_eq!(kw.len(), 1);
        assert_eq!(kw[0].trigger_words, vec!["BANNED_TERM".to_string()]);
        let conv = p.conversational_translation_rules.unwrap().rules;
        assert_eq!(conv.len(), 1);
        assert_eq!(conv[0].templates.get("A").map(String::as_str), Some("STRONG_PHRASE"));
    }

    #[test]
    fn any_loader_keeps_native_profiles() {
        let native = r#"{"assessment_rules":{"keyword_rules":[{"trigger_words":["x"],"warning_message":"m"}]}}"#;
        let p = Profile::load_any_json_str(native).unwrap();
        assert!(!p.is_empty());
    }

    fn load_err(json: &str) -> String {
        Profile::load_any_json_str(json).expect_err("must be rejected").to_string()
    }

    #[test]
    fn d3_empty_pattern_rejected() {
        let e = load_err(r#"{"assessment_rules":{"keyword_rules":[{"trigger_words":[""],"warning_message":"m"}]}}"#);
        assert!(e.contains("empty pattern") && e.contains("keyword rule #0"), "{e}");
        let e = load_err(r#"{"assessment_rules":{"document_template":{"numerical_requirements":{"conflict_items":[{"item_code":"F20","name":"","reason":"r"}]}}}}"#);
        assert!(e.contains("empty pattern") && e.contains("F20"), "{e}");
        let e = load_err(r#"{"conversational_translation_rules":{"rules":[{"item_id":"X1","item_name":"n","colloquial_patterns":[""],"templates":{}}]}}"#);
        assert!(e.contains("empty pattern") && e.contains("X1"), "{e}");
    }

    #[test]
    fn d6_severity_is_case_insensitive_and_unknown_is_rejected() {
        let g = |sev: &str| format!(
            r#"{{"domain":"d","version":"1","rules":[{{"id":"R1","pattern":"BAD","message":"m","severity":"{sev}"}}]}}"#
        );
        let p = Profile::load_any_json_str(&g("Error")).unwrap();
        assert_eq!(p.assessment_rules.unwrap().keyword_rules.unwrap().len(), 1);
        let e = load_err(&g("fatal"));
        assert!(e.contains("unknown severity") && e.contains("R1"), "{e}");
    }

    #[test]
    fn d7_invalid_value_regex_rejected() {
        let e = load_err(r#"{"assessment_rules":{"document_template":{"numerical_requirements":{"value_regex_pattern":"(unclosed"}}}}"#);
        assert!(e.contains("value_regex_pattern"), "{e}");
    }

    #[test]
    fn s4_limits_enforced() {
        let long = "a".repeat(crate::limits::MAX_PATTERN_BYTES + 1);
        let e = load_err(&format!(r#"{{"assessment_rules":{{"keyword_rules":[{{"trigger_words":["{long}"],"warning_message":"m"}}]}}}}"#));
        assert!(e.contains("longer than"), "{e}");
        let e = load_err(&format!(r#"{{"assessment_rules":{{"document_template":{{"numerical_requirements":{{"value_regex_pattern":"a{{1000}}{{1000}}{{1000}}"}}}}}}}}"#));
        assert!(e.contains("value_regex_pattern"), "{e}");
    }

    #[test]
    fn document_template_without_numerical_requirements_is_empty() {
        let p = Profile::load_from_json_str(
            r#"{"assessment_rules":{"document_template":{}}}"#,
        )
        .unwrap();
        assert!(p.is_empty());
    }

    #[test]
    fn numerical_requirements_make_profile_non_empty() {
        let p = Profile::load_from_json_str(
            r#"{"assessment_rules":{"document_template":{"numerical_requirements":{"min_value":3}}}}"#,
        )
        .unwrap();
        assert!(!p.is_empty());
    }
}
