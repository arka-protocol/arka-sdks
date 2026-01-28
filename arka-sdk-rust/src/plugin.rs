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
