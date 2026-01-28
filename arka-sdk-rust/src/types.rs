//! PACT SDK Type Definitions
//!
//! Core types for building PACT domain plugins.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Rule severity levels.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum Severity {
    Low,
    Medium,
    High,
    Critical,
}

impl Default for Severity {
    fn default() -> Self {
        Self::Medium
    }
}

/// Rule consequence decisions.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum Decision {
    Allow,
    Deny,
    Flag,
}

/// Overall decision status.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum DecisionStatus {
    Allow,
    AllowWithFlags,
    Deny,
}

/// Rule evaluation result.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum EvaluationResult {
    Pass,
    Fail,
    Skip,
    Error,
}

/// Plugin manifest describing capabilities.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginManifest {
    pub id: String,
    pub name: String,
    pub version: String,
    pub author: String,
    pub description: String,
    pub entity_types: Vec<String>,
    pub event_types: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub dependencies: Vec<String>,
    pub pact_core_version: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub config_schema: Option<HashMap<String, serde_json::Value>>,
}

/// Domain-specific event before conversion.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DomainEvent {
    #[serde(rename = "type")]
    pub event_type: String,
    pub payload: HashMap<String, serde_json::Value>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub entity_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub jurisdiction: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub occurred_at: Option<DateTime<Utc>>,
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub metadata: HashMap<String, serde_json::Value>,
}

/// Canonical PACT event.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PactEvent {
    pub id: String,
    pub source: String,
    #[serde(rename = "type")]
    pub event_type: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub entity_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub entity_type: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub jurisdiction: Option<String>,
    pub payload: HashMap<String, serde_json::Value>,
    pub occurred_at: DateTime<Utc>,
    pub received_at: DateTime<Utc>,
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub metadata: HashMap<String, serde_json::Value>,
}

/// PACT entity.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PactEntity {
    pub id: String,
    #[serde(rename = "type")]
    pub entity_type: String,
    pub data: HashMap<String, serde_json::Value>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub jurisdiction: Option<String>,
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub metadata: HashMap<String, serde_json::Value>,
}

/// Entity type definition with schema.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PactEntityType {
    pub name: String,
    #[serde(default)]
    pub description: String,
    pub schema: HashMap<String, serde_json::Value>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub required_fields: Vec<String>,
}

/// Condition types for rules.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum Condition {
    Compare {
        field: String,
        operator: String,
        value: serde_json::Value,
    },
    And {
        conditions: Vec<Condition>,
    },
    Or {
        conditions: Vec<Condition>,
    },
    Not {
        condition: Box<Condition>,
    },
    Exists {
        field: String,
    },
    In {
        field: String,
        values: Vec<serde_json::Value>,
    },
    Range {
        field: String,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        min: Option<f64>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        max: Option<f64>,
        #[serde(default = "default_true")]
        min_inclusive: bool,
        #[serde(default = "default_true")]
        max_inclusive: bool,
    },
    Expression {
        expression: String,
        #[serde(default = "default_language")]
        language: String,
    },
}

fn default_true() -> bool {
    true
}

fn default_language() -> String {
    "cel".to_string()
}

/// Rule consequence.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Consequence {
    pub decision: Decision,
    pub code: String,
    pub message: String,
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub metadata: HashMap<String, serde_json::Value>,
}

/// PACT rule.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PactRule {
    pub id: String,
    pub name: String,
    #[serde(default)]
    pub description: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub jurisdiction: Option<String>,
    #[serde(default)]
    pub severity: Severity,
    pub condition: Condition,
    pub consequence: Consequence,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub tags: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub effective_from: Option<DateTime<Utc>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub effective_to: Option<DateTime<Utc>>,
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub metadata: HashMap<String, serde_json::Value>,
}

/// Result of evaluating a rule.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuleEvaluation {
    pub rule_id: String,
    pub rule_name: String,
    pub result: EvaluationResult,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub code: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
    #[serde(default)]
    pub duration_ms: i64,
}

/// Decision from rule evaluation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PactDecision {
    pub id: String,
    pub event_id: String,
    pub status: DecisionStatus,
    #[serde(default)]
    pub rule_evaluations: Vec<RuleEvaluation>,
    pub created_at: DateTime<Utc>,
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub metadata: HashMap<String, serde_json::Value>,
}

/// Validation error.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationError {
    pub field: String,
    pub message: String,
    pub code: String,
}

/// Validation warning.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationWarning {
    pub field: String,
    pub message: String,
}

/// Result of validation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationResult {
    pub valid: bool,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub errors: Vec<ValidationError>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub warnings: Vec<ValidationWarning>,
}

impl ValidationResult {
    /// Creates a successful validation result.
    pub fn success() -> Self {
        Self {
            valid: true,
            errors: vec![],
            warnings: vec![],
        }
    }

    /// Creates a failed validation result.
    pub fn failure(errors: Vec<ValidationError>) -> Self {
        Self {
            valid: false,
            errors,
            warnings: vec![],
        }
    }
}
