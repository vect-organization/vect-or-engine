// Copyright (c) 2026 1abcdefggs
// SPDX-License-Identifier: MIT
// https://github.com/1abcdefggs/vect-or-engine

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
        let value: serde_json::Value = serde_json::from_str(json_str)?;
        let is_guideline = value.get("rules").map_or(false, |r| r.is_array())
            && value.get("domain").is_some();
        if is_guideline {
            if let Ok(p) = Self::from_guideline_json_str(json_str) {
                return Ok(p);
            }
        }
        serde_json::from_value(value)
    }

    /// True when no section carries any rule (used to surface silent no-op profiles).
    pub fn is_empty(&self) -> bool {
        let no_kw = self
            .assessment_rules
            .as_ref()
            .and_then(|a| a.keyword_rules.as_ref())
            .map_or(true, |k| k.is_empty());
        // Destructuring (not field access) so that adding a field to DocumentTemplate
        // fails to compile here until is_empty is updated.
        let no_num = match self.assessment_rules.as_ref().and_then(|a| a.document_template.as_ref()) {
            Some(DocumentTemplate { numerical_requirements }) => numerical_requirements.is_none(),
            None => true,
        };
        let no_conv = self
            .conversational_translation_rules
            .as_ref()
            .map_or(true, |c| c.rules.is_empty());
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
            match rule.severity.as_str() {
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
