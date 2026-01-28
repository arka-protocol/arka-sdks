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

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    // =========================================================================
    // Severity Tests
    // =========================================================================

    #[test]
    fn test_severity_default() {
        let severity = Severity::default();
        assert_eq!(severity, Severity::Medium);
    }

    #[test]
    fn test_severity_serialization() {
        assert_eq!(serde_json::to_string(&Severity::Low).unwrap(), "\"LOW\"");
        assert_eq!(serde_json::to_string(&Severity::Medium).unwrap(), "\"MEDIUM\"");
        assert_eq!(serde_json::to_string(&Severity::High).unwrap(), "\"HIGH\"");
        assert_eq!(serde_json::to_string(&Severity::Critical).unwrap(), "\"CRITICAL\"");
    }

    #[test]
    fn test_severity_deserialization() {
        assert_eq!(serde_json::from_str::<Severity>("\"LOW\"").unwrap(), Severity::Low);
        assert_eq!(serde_json::from_str::<Severity>("\"MEDIUM\"").unwrap(), Severity::Medium);
        assert_eq!(serde_json::from_str::<Severity>("\"HIGH\"").unwrap(), Severity::High);
        assert_eq!(serde_json::from_str::<Severity>("\"CRITICAL\"").unwrap(), Severity::Critical);
    }

    // =========================================================================
    // Decision Tests
    // =========================================================================

    #[test]
    fn test_decision_serialization() {
        assert_eq!(serde_json::to_string(&Decision::Allow).unwrap(), "\"ALLOW\"");
        assert_eq!(serde_json::to_string(&Decision::Deny).unwrap(), "\"DENY\"");
        assert_eq!(serde_json::to_string(&Decision::Flag).unwrap(), "\"FLAG\"");
    }

    #[test]
    fn test_decision_deserialization() {
        assert_eq!(serde_json::from_str::<Decision>("\"ALLOW\"").unwrap(), Decision::Allow);
        assert_eq!(serde_json::from_str::<Decision>("\"DENY\"").unwrap(), Decision::Deny);
        assert_eq!(serde_json::from_str::<Decision>("\"FLAG\"").unwrap(), Decision::Flag);
    }

    // =========================================================================
    // DecisionStatus Tests
    // =========================================================================

    #[test]
    fn test_decision_status_serialization() {
        assert_eq!(serde_json::to_string(&DecisionStatus::Allow).unwrap(), "\"ALLOW\"");
        assert_eq!(serde_json::to_string(&DecisionStatus::AllowWithFlags).unwrap(), "\"ALLOW_WITH_FLAGS\"");
        assert_eq!(serde_json::to_string(&DecisionStatus::Deny).unwrap(), "\"DENY\"");
    }

    #[test]
    fn test_decision_status_deserialization() {
        assert_eq!(serde_json::from_str::<DecisionStatus>("\"ALLOW\"").unwrap(), DecisionStatus::Allow);
        assert_eq!(serde_json::from_str::<DecisionStatus>("\"ALLOW_WITH_FLAGS\"").unwrap(), DecisionStatus::AllowWithFlags);
        assert_eq!(serde_json::from_str::<DecisionStatus>("\"DENY\"").unwrap(), DecisionStatus::Deny);
    }

    // =========================================================================
    // EvaluationResult Tests
    // =========================================================================

    #[test]
    fn test_evaluation_result_serialization() {
        assert_eq!(serde_json::to_string(&EvaluationResult::Pass).unwrap(), "\"PASS\"");
        assert_eq!(serde_json::to_string(&EvaluationResult::Fail).unwrap(), "\"FAIL\"");
        assert_eq!(serde_json::to_string(&EvaluationResult::Skip).unwrap(), "\"SKIP\"");
        assert_eq!(serde_json::to_string(&EvaluationResult::Error).unwrap(), "\"ERROR\"");
    }

    // =========================================================================
    // PluginManifest Tests
    // =========================================================================

    #[test]
    fn test_plugin_manifest_serialization() {
        let manifest = PluginManifest {
            id: "test-plugin".to_string(),
            name: "Test Plugin".to_string(),
            version: "1.0.0".to_string(),
            author: "Test Author".to_string(),
            description: "A test plugin".to_string(),
            entity_types: vec!["Entity1".to_string()],
            event_types: vec!["EVENT_1".to_string()],
            dependencies: vec![],
            pact_core_version: "0.1.0".to_string(),
            config_schema: None,
        };

        let json = serde_json::to_value(&manifest).unwrap();
        assert_eq!(json["id"], "test-plugin");
        assert_eq!(json["name"], "Test Plugin");
        assert_eq!(json["version"], "1.0.0");
    }

    #[test]
    fn test_plugin_manifest_deserialization() {
        let json = r#"{
            "id": "test-plugin",
            "name": "Test Plugin",
            "version": "1.0.0",
            "author": "Test Author",
            "description": "A test plugin",
            "entity_types": ["Entity1"],
            "event_types": ["EVENT_1"],
            "pact_core_version": "0.1.0"
        }"#;

        let manifest: PluginManifest = serde_json::from_str(json).unwrap();
        assert_eq!(manifest.id, "test-plugin");
        assert_eq!(manifest.entity_types.len(), 1);
        assert!(manifest.dependencies.is_empty());
        assert!(manifest.config_schema.is_none());
    }

    // =========================================================================
    // DomainEvent Tests
    // =========================================================================

    #[test]
    fn test_domain_event_minimal() {
        let event = DomainEvent {
            event_type: "TEST_EVENT".to_string(),
            payload: HashMap::new(),
            entity_id: None,
            jurisdiction: None,
            occurred_at: None,
            metadata: HashMap::new(),
        };

        let json = serde_json::to_value(&event).unwrap();
        assert_eq!(json["type"], "TEST_EVENT");
        assert!(json.get("entity_id").is_none());
    }

    #[test]
    fn test_domain_event_full() {
        let mut payload = HashMap::new();
        payload.insert("amount".to_string(), json!(1000));

        let mut metadata = HashMap::new();
        metadata.insert("source".to_string(), json!("test"));

        let event = DomainEvent {
            event_type: "TRANSACTION_CREATED".to_string(),
            payload,
            entity_id: Some("entity-123".to_string()),
            jurisdiction: Some("US".to_string()),
            occurred_at: Some(chrono::Utc::now()),
            metadata,
        };

        let json = serde_json::to_value(&event).unwrap();
        assert_eq!(json["type"], "TRANSACTION_CREATED");
        assert_eq!(json["entity_id"], "entity-123");
        assert_eq!(json["jurisdiction"], "US");
        assert_eq!(json["payload"]["amount"], 1000);
    }

    // =========================================================================
    // PactEvent Tests
    // =========================================================================

    #[test]
    fn test_pact_event_serialization_roundtrip() {
        let now = chrono::Utc::now();
        let mut payload = HashMap::new();
        payload.insert("key".to_string(), json!("value"));

        let event = PactEvent {
            id: "evt_123456".to_string(),
            source: "test-plugin".to_string(),
            event_type: "TEST_EVENT".to_string(),
            entity_id: Some("entity-1".to_string()),
            entity_type: Some("TestEntity".to_string()),
            jurisdiction: Some("US".to_string()),
            payload,
            occurred_at: now,
            received_at: now,
            metadata: HashMap::new(),
        };

        let json_str = serde_json::to_string(&event).unwrap();
        let deserialized: PactEvent = serde_json::from_str(&json_str).unwrap();

        assert_eq!(deserialized.id, event.id);
        assert_eq!(deserialized.source, event.source);
        assert_eq!(deserialized.event_type, event.event_type);
        assert_eq!(deserialized.entity_id, event.entity_id);
    }

    // =========================================================================
    // PactEntity Tests
    // =========================================================================

    #[test]
    fn test_pact_entity_serialization() {
        let now = chrono::Utc::now();
        let mut data = HashMap::new();
        data.insert("name".to_string(), json!("Test Entity"));
        data.insert("status".to_string(), json!("active"));

        let entity = PactEntity {
            id: "ent_123".to_string(),
            entity_type: "Account".to_string(),
            data,
            created_at: now,
            updated_at: now,
            jurisdiction: Some("US".to_string()),
            metadata: HashMap::new(),
        };

        let json = serde_json::to_value(&entity).unwrap();
        assert_eq!(json["id"], "ent_123");
        assert_eq!(json["type"], "Account");
        assert_eq!(json["data"]["name"], "Test Entity");
    }

    // =========================================================================
    // PactEntityType Tests
    // =========================================================================

    #[test]
    fn test_pact_entity_type() {
        let mut schema = HashMap::new();
        schema.insert("type".to_string(), json!("object"));

        let entity_type = PactEntityType {
            name: "Account".to_string(),
            description: "A user account".to_string(),
            schema,
            required_fields: vec!["id".to_string(), "name".to_string()],
        };

        let json = serde_json::to_value(&entity_type).unwrap();
        assert_eq!(json["name"], "Account");
        assert_eq!(json["required_fields"].as_array().unwrap().len(), 2);
    }

    // =========================================================================
    // Condition Tests
    // =========================================================================

    #[test]
    fn test_condition_compare_serialization() {
        let condition = Condition::Compare {
            field: "amount".to_string(),
            operator: "gt".to_string(),
            value: json!(1000),
        };

        let json = serde_json::to_value(&condition).unwrap();
        assert_eq!(json["type"], "compare");
        assert_eq!(json["field"], "amount");
        assert_eq!(json["operator"], "gt");
        assert_eq!(json["value"], 1000);
    }

    #[test]
    fn test_condition_and_serialization() {
        let condition = Condition::And {
            conditions: vec![
                Condition::Compare {
                    field: "a".to_string(),
                    operator: "eq".to_string(),
                    value: json!(1),
                },
                Condition::Compare {
                    field: "b".to_string(),
                    operator: "eq".to_string(),
                    value: json!(2),
                },
            ],
        };

        let json = serde_json::to_value(&condition).unwrap();
        assert_eq!(json["type"], "and");
        assert_eq!(json["conditions"].as_array().unwrap().len(), 2);
    }

    #[test]
    fn test_condition_or_serialization() {
        let condition = Condition::Or {
            conditions: vec![
                Condition::Exists {
                    field: "optional_field".to_string(),
                },
            ],
        };

        let json = serde_json::to_value(&condition).unwrap();
        assert_eq!(json["type"], "or");
    }

    #[test]
    fn test_condition_not_serialization() {
        let condition = Condition::Not {
            condition: Box::new(Condition::Exists {
                field: "blocked".to_string(),
            }),
        };

        let json = serde_json::to_value(&condition).unwrap();
        assert_eq!(json["type"], "not");
        assert_eq!(json["condition"]["type"], "exists");
    }

    #[test]
    fn test_condition_exists_serialization() {
        let condition = Condition::Exists {
            field: "email".to_string(),
        };

        let json = serde_json::to_value(&condition).unwrap();
        assert_eq!(json["type"], "exists");
        assert_eq!(json["field"], "email");
    }

    #[test]
    fn test_condition_in_serialization() {
        let condition = Condition::In {
            field: "status".to_string(),
            values: vec![json!("active"), json!("pending")],
        };

        let json = serde_json::to_value(&condition).unwrap();
        assert_eq!(json["type"], "in");
        assert_eq!(json["field"], "status");
        assert_eq!(json["values"].as_array().unwrap().len(), 2);
    }

    #[test]
    fn test_condition_range_serialization() {
        let condition = Condition::Range {
            field: "amount".to_string(),
            min: Some(100.0),
            max: Some(10000.0),
            min_inclusive: true,
            max_inclusive: false,
        };

        let json = serde_json::to_value(&condition).unwrap();
        assert_eq!(json["type"], "range");
        assert_eq!(json["field"], "amount");
        assert_eq!(json["min"], 100.0);
        assert_eq!(json["max"], 10000.0);
    }

    #[test]
    fn test_condition_expression_serialization() {
        let condition = Condition::Expression {
            expression: "amount > 1000 && status == 'active'".to_string(),
            language: "cel".to_string(),
        };

        let json = serde_json::to_value(&condition).unwrap();
        assert_eq!(json["type"], "expression");
        assert_eq!(json["language"], "cel");
    }

    #[test]
    fn test_condition_deserialization() {
        let json = r#"{
            "type": "compare",
            "field": "amount",
            "operator": "gte",
            "value": 500
        }"#;

        let condition: Condition = serde_json::from_str(json).unwrap();
        match condition {
            Condition::Compare { field, operator, value } => {
                assert_eq!(field, "amount");
                assert_eq!(operator, "gte");
                assert_eq!(value, json!(500));
            }
            _ => panic!("Expected Compare condition"),
        }
    }

    // =========================================================================
    // Consequence Tests
    // =========================================================================

    #[test]
    fn test_consequence_serialization() {
        let consequence = Consequence {
            decision: Decision::Deny,
            code: "LIMIT_EXCEEDED".to_string(),
            message: "Transaction limit exceeded".to_string(),
            metadata: HashMap::new(),
        };

        let json = serde_json::to_value(&consequence).unwrap();
        assert_eq!(json["decision"], "DENY");
        assert_eq!(json["code"], "LIMIT_EXCEEDED");
    }

    // =========================================================================
    // PactRule Tests
    // =========================================================================

    #[test]
    fn test_pact_rule_serialization() {
        let rule = PactRule {
            id: "rule_123".to_string(),
            name: "High Value Transaction".to_string(),
            description: "Flag high value transactions".to_string(),
            jurisdiction: Some("US".to_string()),
            severity: Severity::High,
            condition: Condition::Compare {
                field: "amount".to_string(),
                operator: "gt".to_string(),
                value: json!(10000),
            },
            consequence: Consequence {
                decision: Decision::Flag,
                code: "HIGH_VALUE".to_string(),
                message: "High value transaction detected".to_string(),
                metadata: HashMap::new(),
            },
            tags: vec!["finance".to_string(), "aml".to_string()],
            effective_from: None,
            effective_to: None,
            metadata: HashMap::new(),
        };

        let json = serde_json::to_value(&rule).unwrap();
        assert_eq!(json["id"], "rule_123");
        assert_eq!(json["severity"], "HIGH");
        assert_eq!(json["tags"].as_array().unwrap().len(), 2);
    }

    #[test]
    fn test_pact_rule_deserialization() {
        let json = r#"{
            "id": "rule_456",
            "name": "Test Rule",
            "description": "A test rule",
            "severity": "MEDIUM",
            "condition": {
                "type": "exists",
                "field": "risk_score"
            },
            "consequence": {
                "decision": "ALLOW",
                "code": "APPROVED",
                "message": "Transaction approved"
            }
        }"#;

        let rule: PactRule = serde_json::from_str(json).unwrap();
        assert_eq!(rule.id, "rule_456");
        assert_eq!(rule.severity, Severity::Medium);
        assert!(rule.tags.is_empty());
    }

    // =========================================================================
    // RuleEvaluation Tests
    // =========================================================================

    #[test]
    fn test_rule_evaluation_serialization() {
        let evaluation = RuleEvaluation {
            rule_id: "rule_123".to_string(),
            rule_name: "Test Rule".to_string(),
            result: EvaluationResult::Pass,
            code: Some("PASSED".to_string()),
            message: Some("Rule passed successfully".to_string()),
            duration_ms: 5,
        };

        let json = serde_json::to_value(&evaluation).unwrap();
        assert_eq!(json["result"], "PASS");
        assert_eq!(json["duration_ms"], 5);
    }

    // =========================================================================
    // PactDecision Tests
    // =========================================================================

    #[test]
    fn test_pact_decision_serialization() {
        let now = chrono::Utc::now();
        let decision = PactDecision {
            id: "dec_123".to_string(),
            event_id: "evt_456".to_string(),
            status: DecisionStatus::AllowWithFlags,
            rule_evaluations: vec![
                RuleEvaluation {
                    rule_id: "rule_1".to_string(),
                    rule_name: "Rule 1".to_string(),
                    result: EvaluationResult::Pass,
                    code: None,
                    message: None,
                    duration_ms: 2,
                },
            ],
            created_at: now,
            metadata: HashMap::new(),
        };

        let json = serde_json::to_value(&decision).unwrap();
        assert_eq!(json["status"], "ALLOW_WITH_FLAGS");
        assert_eq!(json["rule_evaluations"].as_array().unwrap().len(), 1);
    }

    // =========================================================================
    // ValidationResult Tests
    // =========================================================================

    #[test]
    fn test_validation_result_success() {
        let result = ValidationResult::success();
        assert!(result.valid);
        assert!(result.errors.is_empty());
        assert!(result.warnings.is_empty());
    }

    #[test]
    fn test_validation_result_failure() {
        let errors = vec![
            ValidationError {
                field: "email".to_string(),
                message: "Invalid email format".to_string(),
                code: "INVALID_FORMAT".to_string(),
            },
        ];
        let result = ValidationResult::failure(errors);
        assert!(!result.valid);
        assert_eq!(result.errors.len(), 1);
        assert_eq!(result.errors[0].field, "email");
    }

    #[test]
    fn test_validation_error_serialization() {
        let error = ValidationError {
            field: "amount".to_string(),
            message: "Amount must be positive".to_string(),
            code: "INVALID_VALUE".to_string(),
        };

        let json = serde_json::to_value(&error).unwrap();
        assert_eq!(json["field"], "amount");
        assert_eq!(json["code"], "INVALID_VALUE");
    }

    #[test]
    fn test_validation_warning_serialization() {
        let warning = ValidationWarning {
            field: "optional_field".to_string(),
            message: "Field is deprecated".to_string(),
        };

        let json = serde_json::to_value(&warning).unwrap();
        assert_eq!(json["field"], "optional_field");
    }
}
