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
