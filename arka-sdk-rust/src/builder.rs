//! Rule and Condition Builders
//!
//! Fluent builder APIs for creating PACT rules and conditions.

use chrono::{DateTime, Utc};
use std::collections::HashMap;
use uuid::Uuid;

use crate::types::*;

/// Builder for creating conditions.
pub struct ConditionBuilder {
    field: Option<String>,
    condition: Option<Condition>,
}

impl ConditionBuilder {
    /// Creates a new ConditionBuilder.
    pub fn new() -> Self {
        Self {
            field: None,
            condition: None,
        }
    }

    /// Sets the field path.
    pub fn field(mut self, path: &str) -> Self {
        self.field = Some(path.to_string());
        self
    }

    /// Creates an equality comparison.
    pub fn eq(mut self, value: serde_json::Value) -> Self {
        self.condition = Some(Condition::Compare {
            field: self.field.clone().unwrap_or_default(),
            operator: "eq".to_string(),
            value,
        });
        self
    }

    /// Creates a not-equal comparison.
    pub fn ne(mut self, value: serde_json::Value) -> Self {
        self.condition = Some(Condition::Compare {
            field: self.field.clone().unwrap_or_default(),
            operator: "ne".to_string(),
            value,
        });
        self
    }

    /// Creates a greater-than comparison.
    pub fn gt(mut self, value: serde_json::Value) -> Self {
        self.condition = Some(Condition::Compare {
            field: self.field.clone().unwrap_or_default(),
            operator: "gt".to_string(),
            value,
        });
        self
    }

    /// Creates a greater-than-or-equal comparison.
    pub fn gte(mut self, value: serde_json::Value) -> Self {
        self.condition = Some(Condition::Compare {
            field: self.field.clone().unwrap_or_default(),
            operator: "gte".to_string(),
            value,
        });
        self
    }

    /// Creates a less-than comparison.
    pub fn lt(mut self, value: serde_json::Value) -> Self {
        self.condition = Some(Condition::Compare {
            field: self.field.clone().unwrap_or_default(),
            operator: "lt".to_string(),
            value,
        });
        self
    }

    /// Creates a less-than-or-equal comparison.
    pub fn lte(mut self, value: serde_json::Value) -> Self {
        self.condition = Some(Condition::Compare {
            field: self.field.clone().unwrap_or_default(),
            operator: "lte".to_string(),
            value,
        });
        self
    }

    /// Creates a contains comparison.
    pub fn contains(mut self, value: &str) -> Self {
        self.condition = Some(Condition::Compare {
            field: self.field.clone().unwrap_or_default(),
            operator: "contains".to_string(),
            value: serde_json::Value::String(value.to_string()),
        });
        self
    }

    /// Creates a regex match comparison.
    pub fn matches(mut self, pattern: &str) -> Self {
        self.condition = Some(Condition::Compare {
            field: self.field.clone().unwrap_or_default(),
            operator: "regex".to_string(),
            value: serde_json::Value::String(pattern.to_string()),
        });
        self
    }

    /// Creates an exists condition.
    pub fn exists(mut self) -> Self {
        self.condition = Some(Condition::Exists {
            field: self.field.clone().unwrap_or_default(),
        });
        self
    }

    /// Creates an in-set condition.
    pub fn in_values(mut self, values: Vec<serde_json::Value>) -> Self {
        self.condition = Some(Condition::In {
            field: self.field.clone().unwrap_or_default(),
            values,
        });
        self
    }

    /// Creates a range condition.
    pub fn between(mut self, min: Option<f64>, max: Option<f64>) -> Self {
        self.condition = Some(Condition::Range {
            field: self.field.clone().unwrap_or_default(),
            min,
            max,
            min_inclusive: true,
            max_inclusive: true,
        });
        self
    }

    /// Creates an expression condition.
    pub fn expression(mut self, expr: &str, language: Option<&str>) -> Self {
        self.condition = Some(Condition::Expression {
            expression: expr.to_string(),
            language: language.unwrap_or("cel").to_string(),
        });
        self
    }

    /// Builds and returns the condition.
    pub fn build(self) -> Option<Condition> {
        self.condition
    }
}

impl Default for ConditionBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// Creates an AND condition from multiple conditions.
pub fn and(conditions: Vec<Condition>) -> Condition {
    Condition::And { conditions }
}

/// Creates an OR condition from multiple conditions.
pub fn or(conditions: Vec<Condition>) -> Condition {
    Condition::Or { conditions }
}

/// Creates a NOT condition.
pub fn not(condition: Condition) -> Condition {
    Condition::Not {
        condition: Box::new(condition),
    }
}

/// Builder for creating rules.
pub struct RuleBuilder {
    id: Option<String>,
    name: Option<String>,
    description: Option<String>,
    jurisdiction: Option<String>,
    severity: Severity,
    condition: Option<Condition>,
    consequence: Option<Consequence>,
    tags: Vec<String>,
    effective_from: Option<DateTime<Utc>>,
    effective_to: Option<DateTime<Utc>>,
    metadata: HashMap<String, serde_json::Value>,
}

impl RuleBuilder {
    /// Creates a new RuleBuilder.
    pub fn new() -> Self {
        Self {
            id: None,
            name: None,
            description: None,
            jurisdiction: None,
            severity: Severity::Medium,
            condition: None,
            consequence: None,
            tags: Vec::new(),
            effective_from: None,
            effective_to: None,
            metadata: HashMap::new(),
        }
    }

    /// Sets the rule ID.
    pub fn id(mut self, id: &str) -> Self {
        self.id = Some(id.to_string());
        self
    }

    /// Sets the rule name.
    pub fn name(mut self, name: &str) -> Self {
        self.name = Some(name.to_string());
        self
    }

    /// Sets the rule description.
    pub fn description(mut self, desc: &str) -> Self {
        self.description = Some(desc.to_string());
        self
    }

    /// Sets the jurisdiction.
    pub fn jurisdiction(mut self, jurisdiction: &str) -> Self {
        self.jurisdiction = Some(jurisdiction.to_string());
        self
    }

    /// Sets the severity.
    pub fn severity(mut self, severity: Severity) -> Self {
        self.severity = severity;
        self
    }

    /// Sets the condition.
    pub fn when(mut self, condition: Condition) -> Self {
        self.condition = Some(condition);
        self
    }

    /// Sets a simple field comparison condition.
    pub fn when_field(mut self, field: &str, operator: &str, value: serde_json::Value) -> Self {
        self.condition = Some(Condition::Compare {
            field: field.to_string(),
            operator: operator.to_string(),
            value,
        });
        self
    }

    /// Sets a DENY consequence.
    pub fn then_deny(mut self, code: &str, message: &str) -> Self {
        self.consequence = Some(Consequence {
            decision: Decision::Deny,
            code: code.to_string(),
            message: message.to_string(),
            metadata: HashMap::new(),
        });
        self
    }

    /// Sets a FLAG consequence.
    pub fn then_flag(mut self, code: &str, message: &str) -> Self {
        self.consequence = Some(Consequence {
            decision: Decision::Flag,
            code: code.to_string(),
            message: message.to_string(),
            metadata: HashMap::new(),
        });
        self
    }

    /// Sets an ALLOW consequence.
    pub fn then_allow(mut self, code: &str, message: &str) -> Self {
        self.consequence = Some(Consequence {
            decision: Decision::Allow,
            code: code.to_string(),
            message: message.to_string(),
            metadata: HashMap::new(),
        });
        self
    }

    /// Adds tags.
    pub fn tags(mut self, tags: Vec<&str>) -> Self {
        self.tags.extend(tags.into_iter().map(String::from));
        self
    }

    /// Sets the effective start date.
    pub fn effective_from(mut self, date: DateTime<Utc>) -> Self {
        self.effective_from = Some(date);
        self
    }

    /// Sets the effective end date.
    pub fn effective_to(mut self, date: DateTime<Utc>) -> Self {
        self.effective_to = Some(date);
        self
    }

    /// Adds metadata.
    pub fn metadata(mut self, key: &str, value: serde_json::Value) -> Self {
        self.metadata.insert(key.to_string(), value);
        self
    }

    /// Builds and returns the rule.
    pub fn build(self) -> Result<PactRule, &'static str> {
        let name = self.name.ok_or("Rule name is required")?;
        let condition = self.condition.ok_or("Rule condition is required")?;
        let consequence = self.consequence.ok_or("Rule consequence is required")?;

        let id = self.id.unwrap_or_else(|| {
            format!("rule_{}", &Uuid::new_v4().to_string()[..12])
        });

        let description = self.description.unwrap_or_else(|| name.clone());

        Ok(PactRule {
            id,
            name,
            description,
            jurisdiction: self.jurisdiction,
            severity: self.severity,
            condition,
            consequence,
            tags: self.tags,
            effective_from: self.effective_from,
            effective_to: self.effective_to,
            metadata: self.metadata,
        })
    }
}

impl Default for RuleBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// Creates a new condition builder.
pub fn condition() -> ConditionBuilder {
    ConditionBuilder::new()
}

/// Creates a new rule builder.
pub fn rule() -> RuleBuilder {
    RuleBuilder::new()
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    // =========================================================================
    // ConditionBuilder Tests
    // =========================================================================

    #[test]
    fn test_condition_builder_new() {
        let builder = ConditionBuilder::new();
        assert!(builder.build().is_none());
    }

    #[test]
    fn test_condition_builder_default() {
        let builder = ConditionBuilder::default();
        assert!(builder.build().is_none());
    }

    #[test]
    fn test_condition_builder_eq() {
        let cond = condition()
            .field("amount")
            .eq(json!(1000))
            .build()
            .unwrap();

        match cond {
            Condition::Compare { field, operator, value } => {
                assert_eq!(field, "amount");
                assert_eq!(operator, "eq");
                assert_eq!(value, json!(1000));
            }
            _ => panic!("Expected Compare condition"),
        }
    }

    #[test]
    fn test_condition_builder_ne() {
        let cond = condition()
            .field("status")
            .ne(json!("blocked"))
            .build()
            .unwrap();

        match cond {
            Condition::Compare { field, operator, value } => {
                assert_eq!(field, "status");
                assert_eq!(operator, "ne");
                assert_eq!(value, json!("blocked"));
            }
            _ => panic!("Expected Compare condition"),
        }
    }

    #[test]
    fn test_condition_builder_gt() {
        let cond = condition()
            .field("price")
            .gt(json!(100))
            .build()
            .unwrap();

        match cond {
            Condition::Compare { operator, .. } => {
                assert_eq!(operator, "gt");
            }
            _ => panic!("Expected Compare condition"),
        }
    }

    #[test]
    fn test_condition_builder_gte() {
        let cond = condition()
            .field("age")
            .gte(json!(18))
            .build()
            .unwrap();

        match cond {
            Condition::Compare { operator, .. } => {
                assert_eq!(operator, "gte");
            }
            _ => panic!("Expected Compare condition"),
        }
    }

    #[test]
    fn test_condition_builder_lt() {
        let cond = condition()
            .field("count")
            .lt(json!(10))
            .build()
            .unwrap();

        match cond {
            Condition::Compare { operator, .. } => {
                assert_eq!(operator, "lt");
            }
            _ => panic!("Expected Compare condition"),
        }
    }

    #[test]
    fn test_condition_builder_lte() {
        let cond = condition()
            .field("limit")
            .lte(json!(1000))
            .build()
            .unwrap();

        match cond {
            Condition::Compare { operator, .. } => {
                assert_eq!(operator, "lte");
            }
            _ => panic!("Expected Compare condition"),
        }
    }

    #[test]
    fn test_condition_builder_contains() {
        let cond = condition()
            .field("description")
            .contains("urgent")
            .build()
            .unwrap();

        match cond {
            Condition::Compare { field, operator, value } => {
                assert_eq!(field, "description");
                assert_eq!(operator, "contains");
                assert_eq!(value, json!("urgent"));
            }
            _ => panic!("Expected Compare condition"),
        }
    }

    #[test]
    fn test_condition_builder_matches() {
        let cond = condition()
            .field("email")
            .matches(r"^[\w\.]+@[\w\.]+$")
            .build()
            .unwrap();

        match cond {
            Condition::Compare { operator, .. } => {
                assert_eq!(operator, "regex");
            }
            _ => panic!("Expected Compare condition"),
        }
    }

    #[test]
    fn test_condition_builder_exists() {
        let cond = condition()
            .field("optional_field")
            .exists()
            .build()
            .unwrap();

        match cond {
            Condition::Exists { field } => {
                assert_eq!(field, "optional_field");
            }
            _ => panic!("Expected Exists condition"),
        }
    }

    #[test]
    fn test_condition_builder_in_values() {
        let cond = condition()
            .field("country")
            .in_values(vec![json!("US"), json!("CA"), json!("MX")])
            .build()
            .unwrap();

        match cond {
            Condition::In { field, values } => {
                assert_eq!(field, "country");
                assert_eq!(values.len(), 3);
            }
            _ => panic!("Expected In condition"),
        }
    }

    #[test]
    fn test_condition_builder_between() {
        let cond = condition()
            .field("temperature")
            .between(Some(0.0), Some(100.0))
            .build()
            .unwrap();

        match cond {
            Condition::Range { field, min, max, min_inclusive, max_inclusive } => {
                assert_eq!(field, "temperature");
                assert_eq!(min, Some(0.0));
                assert_eq!(max, Some(100.0));
                assert!(min_inclusive);
                assert!(max_inclusive);
            }
            _ => panic!("Expected Range condition"),
        }
    }

    #[test]
    fn test_condition_builder_between_partial() {
        let cond = condition()
            .field("score")
            .between(Some(0.0), None)
            .build()
            .unwrap();

        match cond {
            Condition::Range { min, max, .. } => {
                assert_eq!(min, Some(0.0));
                assert!(max.is_none());
            }
            _ => panic!("Expected Range condition"),
        }
    }

    #[test]
    fn test_condition_builder_expression() {
        let cond = condition()
            .expression("amount > threshold", None)
            .build()
            .unwrap();

        match cond {
            Condition::Expression { expression, language } => {
                assert_eq!(expression, "amount > threshold");
                assert_eq!(language, "cel");
            }
            _ => panic!("Expected Expression condition"),
        }
    }

    #[test]
    fn test_condition_builder_expression_custom_language() {
        let cond = condition()
            .expression("return amount > 100", Some("lua"))
            .build()
            .unwrap();

        match cond {
            Condition::Expression { language, .. } => {
                assert_eq!(language, "lua");
            }
            _ => panic!("Expected Expression condition"),
        }
    }

    // =========================================================================
    // Logical Operator Tests
    // =========================================================================

    #[test]
    fn test_and_operator() {
        let c1 = condition().field("a").eq(json!(1)).build().unwrap();
        let c2 = condition().field("b").eq(json!(2)).build().unwrap();

        let combined = and(vec![c1, c2]);

        match combined {
            Condition::And { conditions } => {
                assert_eq!(conditions.len(), 2);
            }
            _ => panic!("Expected And condition"),
        }
    }

    #[test]
    fn test_or_operator() {
        let c1 = condition().field("status").eq(json!("active")).build().unwrap();
        let c2 = condition().field("status").eq(json!("pending")).build().unwrap();

        let combined = or(vec![c1, c2]);

        match combined {
            Condition::Or { conditions } => {
                assert_eq!(conditions.len(), 2);
            }
            _ => panic!("Expected Or condition"),
        }
    }

    #[test]
    fn test_not_operator() {
        let c = condition().field("blocked").exists().build().unwrap();
        let negated = not(c);

        match negated {
            Condition::Not { condition } => {
                match *condition {
                    Condition::Exists { field } => {
                        assert_eq!(field, "blocked");
                    }
                    _ => panic!("Expected Exists inside Not"),
                }
            }
            _ => panic!("Expected Not condition"),
        }
    }

    #[test]
    fn test_complex_nested_conditions() {
        let high_amount = condition().field("amount").gt(json!(10000)).build().unwrap();
        let is_international = condition().field("international").eq(json!(true)).build().unwrap();
        let is_flagged = condition().field("flagged").eq(json!(true)).build().unwrap();

        let combined = or(vec![
            and(vec![high_amount, is_international]),
            is_flagged,
        ]);

        match combined {
            Condition::Or { conditions } => {
                assert_eq!(conditions.len(), 2);
                match &conditions[0] {
                    Condition::And { conditions } => {
                        assert_eq!(conditions.len(), 2);
                    }
                    _ => panic!("Expected And condition"),
                }
            }
            _ => panic!("Expected Or condition"),
        }
    }

    // =========================================================================
    // RuleBuilder Tests
    // =========================================================================

    #[test]
    fn test_rule_builder_new() {
        let builder = RuleBuilder::new();
        assert!(builder.build().is_err());
    }

    #[test]
    fn test_rule_builder_default() {
        let builder = RuleBuilder::default();
        assert!(builder.build().is_err());
    }

    #[test]
    fn test_rule_builder_minimal() {
        let r = rule()
            .name("Test Rule")
            .when_field("amount", "gt", json!(1000))
            .then_deny("LIMIT_EXCEEDED", "Amount exceeds limit")
            .build()
            .unwrap();

        assert_eq!(r.name, "Test Rule");
        assert!(!r.id.is_empty());
        assert!(r.id.starts_with("rule_"));
        assert_eq!(r.severity, Severity::Medium);
    }

    #[test]
    fn test_rule_builder_with_id() {
        let r = rule()
            .id("custom-rule-id")
            .name("Custom Rule")
            .when_field("status", "eq", json!("blocked"))
            .then_deny("BLOCKED", "Account is blocked")
            .build()
            .unwrap();

        assert_eq!(r.id, "custom-rule-id");
    }

    #[test]
    fn test_rule_builder_with_description() {
        let r = rule()
            .name("Rule Name")
            .description("A detailed description")
            .when_field("a", "eq", json!(1))
            .then_allow("OK", "Allowed")
            .build()
            .unwrap();

        assert_eq!(r.description, "A detailed description");
    }

    #[test]
    fn test_rule_builder_description_defaults_to_name() {
        let r = rule()
            .name("My Rule")
            .when_field("a", "eq", json!(1))
            .then_allow("OK", "Ok")
            .build()
            .unwrap();

        assert_eq!(r.description, "My Rule");
    }

    #[test]
    fn test_rule_builder_with_jurisdiction() {
        let r = rule()
            .name("US Rule")
            .jurisdiction("US")
            .when_field("amount", "gt", json!(10000))
            .then_flag("US_REPORTING", "US reporting threshold")
            .build()
            .unwrap();

        assert_eq!(r.jurisdiction, Some("US".to_string()));
    }

    #[test]
    fn test_rule_builder_with_severity() {
        let r = rule()
            .name("Critical Rule")
            .severity(Severity::Critical)
            .when_field("a", "eq", json!(1))
            .then_deny("CRITICAL", "Critical error")
            .build()
            .unwrap();

        assert_eq!(r.severity, Severity::Critical);
    }

    #[test]
    fn test_rule_builder_with_condition() {
        let cond = and(vec![
            condition().field("a").gt(json!(10)).build().unwrap(),
            condition().field("b").lt(json!(100)).build().unwrap(),
        ]);

        let r = rule()
            .name("Complex Rule")
            .when(cond)
            .then_flag("COMPLEX", "Complex condition matched")
            .build()
            .unwrap();

        match &r.condition {
            Condition::And { conditions } => {
                assert_eq!(conditions.len(), 2);
            }
            _ => panic!("Expected And condition"),
        }
    }

    #[test]
    fn test_rule_builder_then_deny() {
        let r = rule()
            .name("Deny Rule")
            .when_field("blocked", "eq", json!(true))
            .then_deny("BLOCKED", "Access denied")
            .build()
            .unwrap();

        assert_eq!(r.consequence.decision, Decision::Deny);
        assert_eq!(r.consequence.code, "BLOCKED");
        assert_eq!(r.consequence.message, "Access denied");
    }

    #[test]
    fn test_rule_builder_then_flag() {
        let r = rule()
            .name("Flag Rule")
            .when_field("suspicious", "eq", json!(true))
            .then_flag("SUSPICIOUS", "Suspicious activity")
            .build()
            .unwrap();

        assert_eq!(r.consequence.decision, Decision::Flag);
    }

    #[test]
    fn test_rule_builder_then_allow() {
        let r = rule()
            .name("Allow Rule")
            .when_field("verified", "eq", json!(true))
            .then_allow("VERIFIED", "Verified user")
            .build()
            .unwrap();

        assert_eq!(r.consequence.decision, Decision::Allow);
    }

    #[test]
    fn test_rule_builder_with_tags() {
        let r = rule()
            .name("Tagged Rule")
            .tags(vec!["finance", "aml", "compliance"])
            .when_field("a", "eq", json!(1))
            .then_allow("OK", "Ok")
            .build()
            .unwrap();

        assert_eq!(r.tags.len(), 3);
        assert!(r.tags.contains(&"finance".to_string()));
        assert!(r.tags.contains(&"aml".to_string()));
    }

    #[test]
    fn test_rule_builder_with_effective_dates() {
        let from = Utc::now();
        let to = from + chrono::Duration::days(30);

        let r = rule()
            .name("Time-limited Rule")
            .effective_from(from)
            .effective_to(to)
            .when_field("a", "eq", json!(1))
            .then_allow("OK", "Ok")
            .build()
            .unwrap();

        assert!(r.effective_from.is_some());
        assert!(r.effective_to.is_some());
        assert!(r.effective_to.unwrap() > r.effective_from.unwrap());
    }

    #[test]
    fn test_rule_builder_with_metadata() {
        let r = rule()
            .name("Metadata Rule")
            .metadata("source", json!("external"))
            .metadata("version", json!(1))
            .when_field("a", "eq", json!(1))
            .then_allow("OK", "Ok")
            .build()
            .unwrap();

        assert_eq!(r.metadata.len(), 2);
        assert_eq!(r.metadata.get("source"), Some(&json!("external")));
        assert_eq!(r.metadata.get("version"), Some(&json!(1)));
    }

    #[test]
    fn test_rule_builder_missing_name() {
        let result = rule()
            .when_field("a", "eq", json!(1))
            .then_allow("OK", "Ok")
            .build();

        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Rule name is required");
    }

    #[test]
    fn test_rule_builder_missing_condition() {
        let result = rule()
            .name("No Condition Rule")
            .then_allow("OK", "Ok")
            .build();

        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Rule condition is required");
    }

    #[test]
    fn test_rule_builder_missing_consequence() {
        let result = rule()
            .name("No Consequence Rule")
            .when_field("a", "eq", json!(1))
            .build();

        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Rule consequence is required");
    }

    // =========================================================================
    // Helper Function Tests
    // =========================================================================

    #[test]
    fn test_condition_helper() {
        let builder = condition();
        let cond = builder.field("test").eq(json!("value")).build();
        assert!(cond.is_some());
    }

    #[test]
    fn test_rule_helper() {
        let builder = rule();
        let r = builder
            .name("Test")
            .when_field("x", "eq", json!(1))
            .then_allow("OK", "Ok")
            .build();
        assert!(r.is_ok());
    }

    // =========================================================================
    // Integration Tests
    // =========================================================================

    #[test]
    fn test_build_aml_rule() {
        let high_value = condition().field("amount").gt(json!(10000)).build().unwrap();
        let is_international = condition().field("international").eq(json!(true)).build().unwrap();
        let high_risk_country = condition()
            .field("destination_country")
            .in_values(vec![json!("XX"), json!("YY"), json!("ZZ")])
            .build()
            .unwrap();

        let aml_condition = or(vec![
            high_value,
            and(vec![is_international, high_risk_country]),
        ]);

        let aml_rule = rule()
            .id("aml-001")
            .name("AML High Risk Transaction")
            .description("Flag transactions that match AML criteria")
            .jurisdiction("US")
            .severity(Severity::High)
            .when(aml_condition)
            .then_flag("AML_REVIEW_REQUIRED", "Transaction requires AML review")
            .tags(vec!["aml", "compliance", "financial-crime"])
            .metadata("regulation", json!("BSA"))
            .metadata("threshold_usd", json!(10000))
            .build()
            .unwrap();

        assert_eq!(aml_rule.id, "aml-001");
        assert_eq!(aml_rule.severity, Severity::High);
        assert_eq!(aml_rule.consequence.decision, Decision::Flag);
        assert_eq!(aml_rule.tags.len(), 3);
    }

    #[test]
    fn test_build_kyc_rule() {
        let kyc_rule = rule()
            .id("kyc-001")
            .name("KYC Verification Required")
            .description("Require KYC verification for high-value accounts")
            .when(and(vec![
                condition().field("account_balance").gte(json!(50000)).build().unwrap(),
                not(condition().field("kyc_verified").eq(json!(true)).build().unwrap()),
            ]))
            .then_deny("KYC_REQUIRED", "KYC verification required for this operation")
            .severity(Severity::Medium)
            .tags(vec!["kyc", "compliance"])
            .build()
            .unwrap();

        assert_eq!(kyc_rule.id, "kyc-001");
        assert_eq!(kyc_rule.consequence.decision, Decision::Deny);
    }
}
