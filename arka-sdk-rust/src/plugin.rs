//! Plugin Implementation
//!
//! Base plugin implementation and traits for PACT domain plugins.

use async_trait::async_trait;
use chrono::Utc;
use std::collections::HashMap;
use uuid::Uuid;

use crate::types::*;

/// Plugin lifecycle hooks.
#[derive(Default)]
pub struct PluginHooks {
    pub on_load: Option<Box<dyn Fn() -> Result<(), Box<dyn std::error::Error>> + Send + Sync>>,
    pub on_unload: Option<Box<dyn Fn() -> Result<(), Box<dyn std::error::Error>> + Send + Sync>>,
}

/// Trait that all PACT domain plugins must implement.
#[async_trait]
pub trait DomainPlugin: Send + Sync {
    /// Returns the plugin manifest.
    fn manifest(&self) -> &PluginManifest;

    /// Returns optional lifecycle hooks.
    fn hooks(&self) -> Option<&PluginHooks> {
        None
    }

    /// Returns entity types defined by this plugin.
    fn get_entity_types(&self) -> &[PactEntityType];

    /// Returns default rules for this domain.
    fn get_default_rules(&self) -> &[PactRule];

    /// Converts a domain event to canonical format.
    fn map_to_canonical_event(&self, event: &DomainEvent) -> Result<PactEvent, Box<dyn std::error::Error>>;

    /// Validates domain-specific data.
    fn validate_domain_data(&self, entity_type: &str, data: &HashMap<String, serde_json::Value>) -> ValidationResult;

    /// Returns context for rule evaluation.
    fn get_evaluation_context(&self, _event: &PactEvent, _entity: Option<&PactEntity>) -> HashMap<String, serde_json::Value> {
        HashMap::new()
    }

    /// Serializes data for blockchain.
    fn serialize_for_chain(&self, data: &serde_json::Value) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
        let canonical = canonical_json(data)?;
        Ok(canonical.into_bytes())
    }

    /// Deserializes data from blockchain.
    fn deserialize_from_chain(&self, data: &[u8]) -> Result<serde_json::Value, Box<dyn std::error::Error>> {
        let value = serde_json::from_slice(data)?;
        Ok(value)
    }
}

/// Base plugin implementation.
pub struct BasePlugin {
    manifest: PluginManifest,
    entity_types: Vec<PactEntityType>,
    default_rules: Vec<PactRule>,
    hooks: Option<PluginHooks>,
}

impl BasePlugin {
    /// Creates a new BasePlugin.
    pub fn new(
        manifest: PluginManifest,
        entity_types: Vec<PactEntityType>,
        default_rules: Vec<PactRule>,
    ) -> Self {
        Self {
            manifest,
            entity_types,
            default_rules,
            hooks: None,
        }
    }

    /// Sets the plugin hooks.
    pub fn with_hooks(mut self, hooks: PluginHooks) -> Self {
        self.hooks = Some(hooks);
        self
    }

    /// Returns the plugin manifest.
    pub fn manifest(&self) -> &PluginManifest {
        &self.manifest
    }

    /// Returns the plugin hooks.
    pub fn hooks(&self) -> Option<&PluginHooks> {
        self.hooks.as_ref()
    }

    /// Returns entity types defined by this plugin.
    pub fn get_entity_types(&self) -> &[PactEntityType] {
        &self.entity_types
    }

    /// Returns default rules for this domain.
    pub fn get_default_rules(&self) -> &[PactRule] {
        &self.default_rules
    }

    /// Converts a domain event to canonical format.
    pub fn map_to_canonical_event(&self, event: &DomainEvent) -> Result<PactEvent, Box<dyn std::error::Error>> {
        let now = Utc::now();
        let occurred_at = event.occurred_at.unwrap_or(now);

        Ok(PactEvent {
            id: format!("evt_{}", &Uuid::new_v4().to_string()[..12]),
            source: self.manifest.id.clone(),
            event_type: event.event_type.clone(),
            entity_id: event.entity_id.clone(),
            entity_type: self.infer_entity_type(event),
            jurisdiction: event.jurisdiction.clone(),
            payload: event.payload.clone(),
            occurred_at,
            received_at: now,
            metadata: event.metadata.clone(),
        })
    }

    /// Infers entity type from event type.
    fn infer_entity_type(&self, event: &DomainEvent) -> Option<String> {
        let parts: Vec<&str> = event.event_type.split('_').collect();
        if parts.len() >= 2 {
            let entity_name = parts[0].to_lowercase();
            let mut chars = entity_name.chars();
            if let Some(first) = chars.next() {
                return Some(first.to_uppercase().chain(chars).collect());
            }
        }
        None
    }

    /// Validates domain-specific data.
    pub fn validate_domain_data(&self, entity_type: &str, data: &HashMap<String, serde_json::Value>) -> ValidationResult {
        // Find the entity type
        let entity_type_def = self.entity_types.iter().find(|et| et.name == entity_type);

        match entity_type_def {
            None => ValidationResult::failure(vec![ValidationError {
                field: "entity_type".to_string(),
                message: format!("Unknown entity type: {}", entity_type),
                code: "UNKNOWN_ENTITY_TYPE".to_string(),
            }]),
            Some(et) => {
                let mut errors = Vec::new();

                // Check required fields
                for field in &et.required_fields {
                    if !data.contains_key(field) {
                        errors.push(ValidationError {
                            field: field.clone(),
                            message: format!("Required field '{}' is missing", field),
                            code: "REQUIRED_FIELD_MISSING".to_string(),
                        });
                    }
                }

                if errors.is_empty() {
                    ValidationResult::success()
                } else {
                    ValidationResult::failure(errors)
                }
            }
        }
    }

    /// Creates a rule ID.
    pub fn create_rule_id(&self) -> String {
        format!("rule_{}", &Uuid::new_v4().to_string()[..12])
    }

    /// Creates a rule with plugin defaults.
    pub fn create_rule(
        &self,
        name: &str,
        condition: Condition,
        consequence: Consequence,
    ) -> PactRule {
        let mut metadata = HashMap::new();
        metadata.insert(
            "plugin_id".to_string(),
            serde_json::Value::String(self.manifest.id.clone()),
        );
        metadata.insert(
            "plugin_version".to_string(),
            serde_json::Value::String(self.manifest.version.clone()),
        );

        PactRule {
            id: self.create_rule_id(),
            name: name.to_string(),
            description: name.to_string(),
            jurisdiction: None,
            severity: Severity::Medium,
            condition,
            consequence,
            tags: vec![self.manifest.id.clone()],
            effective_from: None,
            effective_to: None,
            metadata,
        }
    }
}

/// Produces canonical JSON with sorted keys.
fn canonical_json(value: &serde_json::Value) -> Result<String, Box<dyn std::error::Error>> {
    match value {
        serde_json::Value::Object(map) => {
            let mut keys: Vec<_> = map.keys().collect();
            keys.sort();

            let mut parts = Vec::new();
            for key in keys {
                let key_json = serde_json::to_string(key)?;
                let value_json = canonical_json(&map[key])?;
                parts.push(format!("{}:{}", key_json, value_json));
            }
            Ok(format!("{{{}}}", parts.join(",")))
        }
        serde_json::Value::Array(arr) => {
            let mut parts = Vec::new();
            for item in arr {
                parts.push(canonical_json(item)?);
            }
            Ok(format!("[{}]", parts.join(",")))
        }
        _ => Ok(serde_json::to_string(value)?),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn create_test_manifest() -> PluginManifest {
        PluginManifest {
            id: "test-plugin".to_string(),
            name: "Test Plugin".to_string(),
            version: "1.0.0".to_string(),
            author: "Test Author".to_string(),
            description: "A test plugin".to_string(),
            entity_types: vec!["TestEntity".to_string()],
            event_types: vec!["TEST_EVENT".to_string()],
            dependencies: vec![],
            pact_core_version: "0.1.0".to_string(),
            config_schema: None,
        }
    }

    fn create_test_entity_types() -> Vec<PactEntityType> {
        vec![PactEntityType {
            name: "TestEntity".to_string(),
            description: "A test entity".to_string(),
            schema: HashMap::new(),
            required_fields: vec!["id".to_string(), "name".to_string()],
        }]
    }

    fn create_test_rules() -> Vec<PactRule> {
        vec![PactRule {
            id: "test-rule-1".to_string(),
            name: "Test Rule".to_string(),
            description: "A test rule".to_string(),
            jurisdiction: None,
            severity: Severity::Medium,
            condition: Condition::Compare {
                field: "value".to_string(),
                operator: "gt".to_string(),
                value: json!(100),
            },
            consequence: Consequence {
                decision: Decision::Flag,
                code: "TEST_FLAG".to_string(),
                message: "Test flag triggered".to_string(),
                metadata: HashMap::new(),
            },
            tags: vec!["test".to_string()],
            effective_from: None,
            effective_to: None,
            metadata: HashMap::new(),
        }]
    }

    // =========================================================================
    // BasePlugin Tests
    // =========================================================================

    #[test]
    fn test_base_plugin_new() {
        let manifest = create_test_manifest();
        let entity_types = create_test_entity_types();
        let rules = create_test_rules();

        let plugin = BasePlugin::new(manifest.clone(), entity_types, rules);

        assert_eq!(plugin.manifest().id, "test-plugin");
        assert_eq!(plugin.manifest().name, "Test Plugin");
        assert!(plugin.hooks().is_none());
    }

    #[test]
    fn test_base_plugin_manifest() {
        let plugin = BasePlugin::new(
            create_test_manifest(),
            create_test_entity_types(),
            create_test_rules(),
        );

        let manifest = plugin.manifest();
        assert_eq!(manifest.id, "test-plugin");
        assert_eq!(manifest.version, "1.0.0");
        assert_eq!(manifest.entity_types.len(), 1);
        assert_eq!(manifest.event_types.len(), 1);
    }

    #[test]
    fn test_base_plugin_get_entity_types() {
        let plugin = BasePlugin::new(
            create_test_manifest(),
            create_test_entity_types(),
            vec![],
        );

        let entity_types = plugin.get_entity_types();
        assert_eq!(entity_types.len(), 1);
        assert_eq!(entity_types[0].name, "TestEntity");
        assert_eq!(entity_types[0].required_fields.len(), 2);
    }

    #[test]
    fn test_base_plugin_get_default_rules() {
        let plugin = BasePlugin::new(
            create_test_manifest(),
            vec![],
            create_test_rules(),
        );

        let rules = plugin.get_default_rules();
        assert_eq!(rules.len(), 1);
        assert_eq!(rules[0].name, "Test Rule");
    }

    #[test]
    fn test_base_plugin_with_hooks() {
        let hooks = PluginHooks {
            on_load: Some(Box::new(|| Ok(()))),
            on_unload: None,
        };

        let plugin = BasePlugin::new(
            create_test_manifest(),
            vec![],
            vec![],
        ).with_hooks(hooks);

        assert!(plugin.hooks().is_some());
        assert!(plugin.hooks().unwrap().on_load.is_some());
        assert!(plugin.hooks().unwrap().on_unload.is_none());
    }

    // =========================================================================
    // Event Mapping Tests
    // =========================================================================

    #[test]
    fn test_map_to_canonical_event_minimal() {
        let plugin = BasePlugin::new(
            create_test_manifest(),
            vec![],
            vec![],
        );

        let domain_event = DomainEvent {
            event_type: "TEST_EVENT".to_string(),
            payload: HashMap::new(),
            entity_id: None,
            jurisdiction: None,
            occurred_at: None,
            metadata: HashMap::new(),
        };

        let pact_event = plugin.map_to_canonical_event(&domain_event).unwrap();

        assert!(pact_event.id.starts_with("evt_"));
        assert_eq!(pact_event.source, "test-plugin");
        assert_eq!(pact_event.event_type, "TEST_EVENT");
        assert!(pact_event.entity_id.is_none());
    }

    #[test]
    fn test_map_to_canonical_event_full() {
        let plugin = BasePlugin::new(
            create_test_manifest(),
            vec![],
            vec![],
        );

        let mut payload = HashMap::new();
        payload.insert("amount".to_string(), json!(1000));
        payload.insert("currency".to_string(), json!("USD"));

        let mut metadata = HashMap::new();
        metadata.insert("source_system".to_string(), json!("external-api"));

        let occurred_at = Utc::now() - chrono::Duration::hours(1);

        let domain_event = DomainEvent {
            event_type: "TRANSACTION_CREATED".to_string(),
            payload,
            entity_id: Some("entity-123".to_string()),
            jurisdiction: Some("US".to_string()),
            occurred_at: Some(occurred_at),
            metadata,
        };

        let pact_event = plugin.map_to_canonical_event(&domain_event).unwrap();

        assert_eq!(pact_event.source, "test-plugin");
        assert_eq!(pact_event.event_type, "TRANSACTION_CREATED");
        assert_eq!(pact_event.entity_id, Some("entity-123".to_string()));
        assert_eq!(pact_event.jurisdiction, Some("US".to_string()));
        assert_eq!(pact_event.payload.get("amount"), Some(&json!(1000)));
        assert_eq!(pact_event.occurred_at, occurred_at);
        assert!(pact_event.received_at >= occurred_at);
    }

    #[test]
    fn test_map_to_canonical_event_infers_entity_type() {
        let plugin = BasePlugin::new(
            create_test_manifest(),
            vec![],
            vec![],
        );

        let domain_event = DomainEvent {
            event_type: "ACCOUNT_CREATED".to_string(),
            payload: HashMap::new(),
            entity_id: None,
            jurisdiction: None,
            occurred_at: None,
            metadata: HashMap::new(),
        };

        let pact_event = plugin.map_to_canonical_event(&domain_event).unwrap();
        assert_eq!(pact_event.entity_type, Some("Account".to_string()));
    }

    #[test]
    fn test_map_to_canonical_event_infers_entity_type_user() {
        let plugin = BasePlugin::new(
            create_test_manifest(),
            vec![],
            vec![],
        );

        let domain_event = DomainEvent {
            event_type: "USER_UPDATED".to_string(),
            payload: HashMap::new(),
            entity_id: None,
            jurisdiction: None,
            occurred_at: None,
            metadata: HashMap::new(),
        };

        let pact_event = plugin.map_to_canonical_event(&domain_event).unwrap();
        assert_eq!(pact_event.entity_type, Some("User".to_string()));
    }

    // =========================================================================
    // Validation Tests
    // =========================================================================

    #[test]
    fn test_validate_domain_data_success() {
        let plugin = BasePlugin::new(
            create_test_manifest(),
            create_test_entity_types(),
            vec![],
        );

        let mut data = HashMap::new();
        data.insert("id".to_string(), json!("123"));
        data.insert("name".to_string(), json!("Test"));
        data.insert("extra".to_string(), json!("value"));

        let result = plugin.validate_domain_data("TestEntity", &data);

        assert!(result.valid);
        assert!(result.errors.is_empty());
    }

    #[test]
    fn test_validate_domain_data_missing_required() {
        let plugin = BasePlugin::new(
            create_test_manifest(),
            create_test_entity_types(),
            vec![],
        );

        let mut data = HashMap::new();
        data.insert("id".to_string(), json!("123"));
        // Missing "name" field

        let result = plugin.validate_domain_data("TestEntity", &data);

        assert!(!result.valid);
        assert_eq!(result.errors.len(), 1);
        assert_eq!(result.errors[0].field, "name");
        assert_eq!(result.errors[0].code, "REQUIRED_FIELD_MISSING");
    }

    #[test]
    fn test_validate_domain_data_multiple_missing() {
        let plugin = BasePlugin::new(
            create_test_manifest(),
            create_test_entity_types(),
            vec![],
        );

        let data = HashMap::new(); // Empty data

        let result = plugin.validate_domain_data("TestEntity", &data);

        assert!(!result.valid);
        assert_eq!(result.errors.len(), 2);
    }

    #[test]
    fn test_validate_domain_data_unknown_entity_type() {
        let plugin = BasePlugin::new(
            create_test_manifest(),
            create_test_entity_types(),
            vec![],
        );

        let data = HashMap::new();

        let result = plugin.validate_domain_data("UnknownEntity", &data);

        assert!(!result.valid);
        assert_eq!(result.errors.len(), 1);
        assert_eq!(result.errors[0].field, "entity_type");
        assert_eq!(result.errors[0].code, "UNKNOWN_ENTITY_TYPE");
    }

    // =========================================================================
    // Rule Creation Tests
    // =========================================================================

    #[test]
    fn test_create_rule_id() {
        let plugin = BasePlugin::new(
            create_test_manifest(),
            vec![],
            vec![],
        );

        let id1 = plugin.create_rule_id();
        let id2 = plugin.create_rule_id();

        assert!(id1.starts_with("rule_"));
        assert!(id2.starts_with("rule_"));
        assert_ne!(id1, id2);
    }

    #[test]
    fn test_create_rule() {
        let plugin = BasePlugin::new(
            create_test_manifest(),
            vec![],
            vec![],
        );

        let condition = Condition::Compare {
            field: "amount".to_string(),
            operator: "gt".to_string(),
            value: json!(10000),
        };

        let consequence = Consequence {
            decision: Decision::Flag,
            code: "HIGH_VALUE".to_string(),
            message: "High value transaction".to_string(),
            metadata: HashMap::new(),
        };

        let rule = plugin.create_rule("High Value Check", condition, consequence);

        assert!(rule.id.starts_with("rule_"));
        assert_eq!(rule.name, "High Value Check");
        assert_eq!(rule.description, "High Value Check");
        assert_eq!(rule.severity, Severity::Medium);
        assert!(rule.tags.contains(&"test-plugin".to_string()));
        assert_eq!(rule.metadata.get("plugin_id"), Some(&json!("test-plugin")));
        assert_eq!(rule.metadata.get("plugin_version"), Some(&json!("1.0.0")));
    }

    // =========================================================================
    // Canonical JSON Tests
    // =========================================================================

    #[test]
    fn test_canonical_json_object() {
        let value = json!({
            "c": 3,
            "a": 1,
            "b": 2
        });

        let canonical = canonical_json(&value).unwrap();
        // Keys should be sorted alphabetically
        assert_eq!(canonical, r#"{"a":1,"b":2,"c":3}"#);
    }

    #[test]
    fn test_canonical_json_nested_object() {
        let value = json!({
            "z": {
                "b": 2,
                "a": 1
            },
            "a": 1
        });

        let canonical = canonical_json(&value).unwrap();
        assert_eq!(canonical, r#"{"a":1,"z":{"a":1,"b":2}}"#);
    }

    #[test]
    fn test_canonical_json_array() {
        let value = json!([3, 1, 2]);

        let canonical = canonical_json(&value).unwrap();
        // Arrays preserve order
        assert_eq!(canonical, "[3,1,2]");
    }

    #[test]
    fn test_canonical_json_string() {
        let value = json!("hello");

        let canonical = canonical_json(&value).unwrap();
        assert_eq!(canonical, r#""hello""#);
    }

    #[test]
    fn test_canonical_json_number() {
        let value = json!(42);

        let canonical = canonical_json(&value).unwrap();
        assert_eq!(canonical, "42");
    }

    #[test]
    fn test_canonical_json_boolean() {
        let canonical_true = canonical_json(&json!(true)).unwrap();
        let canonical_false = canonical_json(&json!(false)).unwrap();

        assert_eq!(canonical_true, "true");
        assert_eq!(canonical_false, "false");
    }

    #[test]
    fn test_canonical_json_null() {
        let value = json!(null);

        let canonical = canonical_json(&value).unwrap();
        assert_eq!(canonical, "null");
    }

    #[test]
    fn test_canonical_json_complex() {
        let value = json!({
            "users": [
                {"name": "Bob", "age": 25},
                {"name": "Alice", "age": 30}
            ],
            "count": 2
        });

        let canonical = canonical_json(&value).unwrap();
        // Keys sorted: count before users
        // Within user objects: age before name
        assert_eq!(
            canonical,
            r#"{"count":2,"users":[{"age":25,"name":"Bob"},{"age":30,"name":"Alice"}]}"#
        );
    }

    // =========================================================================
    // PluginHooks Tests
    // =========================================================================

    #[test]
    fn test_plugin_hooks_default() {
        let hooks = PluginHooks::default();
        assert!(hooks.on_load.is_none());
        assert!(hooks.on_unload.is_none());
    }

    #[test]
    fn test_plugin_hooks_on_load_success() {
        let hooks = PluginHooks {
            on_load: Some(Box::new(|| Ok(()))),
            on_unload: None,
        };

        let result = (hooks.on_load.unwrap())();
        assert!(result.is_ok());
    }

    #[test]
    fn test_plugin_hooks_on_load_error() {
        let hooks = PluginHooks {
            on_load: Some(Box::new(|| {
                Err("Load failed".into())
            })),
            on_unload: None,
        };

        let result = (hooks.on_load.unwrap())();
        assert!(result.is_err());
    }

    // =========================================================================
    // DomainPlugin Trait Tests (via BasePlugin)
    // =========================================================================

    struct TestPlugin {
        base: BasePlugin,
    }

    impl TestPlugin {
        fn new() -> Self {
            Self {
                base: BasePlugin::new(
                    create_test_manifest(),
                    create_test_entity_types(),
                    create_test_rules(),
                ),
            }
        }
    }

    #[async_trait]
    impl DomainPlugin for TestPlugin {
        fn manifest(&self) -> &PluginManifest {
            self.base.manifest()
        }

        fn hooks(&self) -> Option<&PluginHooks> {
            self.base.hooks()
        }

        fn get_entity_types(&self) -> &[PactEntityType] {
            self.base.get_entity_types()
        }

        fn get_default_rules(&self) -> &[PactRule] {
            self.base.get_default_rules()
        }

        fn map_to_canonical_event(&self, event: &DomainEvent) -> Result<PactEvent, Box<dyn std::error::Error>> {
            self.base.map_to_canonical_event(event)
        }

        fn validate_domain_data(&self, entity_type: &str, data: &HashMap<String, serde_json::Value>) -> ValidationResult {
            self.base.validate_domain_data(entity_type, data)
        }
    }

    #[test]
    fn test_domain_plugin_trait() {
        let plugin = TestPlugin::new();

        assert_eq!(plugin.manifest().id, "test-plugin");
        assert_eq!(plugin.get_entity_types().len(), 1);
        assert_eq!(plugin.get_default_rules().len(), 1);
    }

    #[test]
    fn test_domain_plugin_default_evaluation_context() {
        let plugin = TestPlugin::new();
        let now = Utc::now();

        let event = PactEvent {
            id: "evt_123".to_string(),
            source: "test".to_string(),
            event_type: "TEST".to_string(),
            entity_id: None,
            entity_type: None,
            jurisdiction: None,
            payload: HashMap::new(),
            occurred_at: now,
            received_at: now,
            metadata: HashMap::new(),
        };

        let context = plugin.get_evaluation_context(&event, None);
        assert!(context.is_empty());
    }

    #[test]
    fn test_domain_plugin_serialize_for_chain() {
        let plugin = TestPlugin::new();
        let data = json!({"key": "value", "number": 42});

        let serialized = plugin.serialize_for_chain(&data).unwrap();

        // Should be canonical JSON as bytes
        let as_string = String::from_utf8(serialized).unwrap();
        assert!(as_string.contains("\"key\""));
        assert!(as_string.contains("\"number\""));
    }

    #[test]
    fn test_domain_plugin_deserialize_from_chain() {
        let plugin = TestPlugin::new();
        let data = r#"{"key":"value","number":42}"#.as_bytes();

        let deserialized = plugin.deserialize_from_chain(data).unwrap();

        assert_eq!(deserialized["key"], "value");
        assert_eq!(deserialized["number"], 42);
    }

    #[test]
    fn test_domain_plugin_roundtrip_serialization() {
        let plugin = TestPlugin::new();
        let original = json!({
            "transaction_id": "tx_123",
            "amount": 1000.50,
            "metadata": {
                "source": "api",
                "version": 1
            }
        });

        let serialized = plugin.serialize_for_chain(&original).unwrap();
        let deserialized = plugin.deserialize_from_chain(&serialized).unwrap();

        assert_eq!(deserialized["transaction_id"], "tx_123");
        assert_eq!(deserialized["amount"], 1000.50);
        assert_eq!(deserialized["metadata"]["source"], "api");
    }
}
