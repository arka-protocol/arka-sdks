//! Plugin Registry
//!
//! Central registry for managing domain plugins.

use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use chrono::{DateTime, Utc};
use thiserror::Error;

use crate::plugin::DomainPlugin;
use crate::types::*;

/// Registry errors.
#[derive(Error, Debug)]
pub enum RegistryError {
    #[error("Plugin '{0}' is already registered")]
    AlreadyRegistered(String),

    #[error("Plugin '{0}' is not registered")]
    NotRegistered(String),

    #[error("Plugin '{0}' requires '{1}' which is not registered")]
    MissingDependency(String, String),

    #[error("Entity type '{0}' is already registered by plugin '{1}'")]
    EntityTypeConflict(String, String),

    #[error("Event type '{0}' is already registered by plugin '{1}'")]
    EventTypeConflict(String, String),

    #[error("Cannot unregister '{0}': plugin '{1}' depends on it")]
    HasDependents(String, String),

    #[error("Hook error: {0}")]
    HookError(String),
}

/// Plugin registration entry.
pub struct PluginRegistration {
    pub plugin: Arc<dyn DomainPlugin>,
    pub registered_at: DateTime<Utc>,
    pub status: String,
}

/// Registry event.
#[derive(Debug, Clone)]
pub struct RegistryEvent {
    pub event_type: String,
    pub plugin_id: String,
    pub data: HashMap<String, String>,
}

/// Event handler type.
pub type RegistryEventHandler = Box<dyn Fn(RegistryEvent) + Send + Sync>;

/// Central registry for PACT domain plugins.
pub struct Registry {
    plugins: RwLock<HashMap<String, PluginRegistration>>,
    entity_type_to_plugin: RwLock<HashMap<String, String>>,
    event_type_to_plugin: RwLock<HashMap<String, String>>,
    event_handlers: RwLock<Vec<RegistryEventHandler>>,
}

impl Registry {
    /// Creates a new registry.
    pub fn new() -> Self {
        Self {
            plugins: RwLock::new(HashMap::new()),
            entity_type_to_plugin: RwLock::new(HashMap::new()),
            event_type_to_plugin: RwLock::new(HashMap::new()),
            event_handlers: RwLock::new(Vec::new()),
        }
    }

    /// Registers a domain plugin.
    pub fn register(&self, plugin: Arc<dyn DomainPlugin>) -> Result<(), RegistryError> {
        let manifest = plugin.manifest();
        let plugin_id = manifest.id.clone();

        // Check for duplicate registration
        {
            let plugins = self.plugins.read().unwrap();
            if plugins.contains_key(&plugin_id) {
                return Err(RegistryError::AlreadyRegistered(plugin_id));
            }
        }

        // Check dependencies
        {
            let plugins = self.plugins.read().unwrap();
            for dep in &manifest.dependencies {
                if !plugins.contains_key(dep) {
                    return Err(RegistryError::MissingDependency(plugin_id.clone(), dep.clone()));
                }
            }
        }

        // Check for entity type conflicts
        {
            let entity_map = self.entity_type_to_plugin.read().unwrap();
            for entity_type in &manifest.entity_types {
                if let Some(existing) = entity_map.get(entity_type) {
                    return Err(RegistryError::EntityTypeConflict(
                        entity_type.clone(),
                        existing.clone(),
                    ));
                }
            }
        }

        // Check for event type conflicts
        {
            let event_map = self.event_type_to_plugin.read().unwrap();
            for event_type in &manifest.event_types {
                if let Some(existing) = event_map.get(event_type) {
                    return Err(RegistryError::EventTypeConflict(
                        event_type.clone(),
                        existing.clone(),
                    ));
                }
            }
        }

        // Call on_load hook
        if let Some(hooks) = plugin.hooks() {
            if let Some(on_load) = &hooks.on_load {
                on_load().map_err(|e| RegistryError::HookError(e.to_string()))?;
            }
        }

        // Register plugin
        {
            let mut plugins = self.plugins.write().unwrap();
            plugins.insert(
                plugin_id.clone(),
                PluginRegistration {
                    plugin: plugin.clone(),
                    registered_at: Utc::now(),
                    status: "active".to_string(),
                },
            );
        }

        // Map entity types
        {
            let mut entity_map = self.entity_type_to_plugin.write().unwrap();
            for entity_type in &manifest.entity_types {
                entity_map.insert(entity_type.clone(), plugin_id.clone());
                self.emit(RegistryEvent {
                    event_type: "entity_type:registered".to_string(),
                    plugin_id: plugin_id.clone(),
                    data: [("entity_type".to_string(), entity_type.clone())]
                        .into_iter()
                        .collect(),
                });
            }
        }

        // Map event types
        {
            let mut event_map = self.event_type_to_plugin.write().unwrap();
            for event_type in &manifest.event_types {
                event_map.insert(event_type.clone(), plugin_id.clone());
            }
        }

        self.emit(RegistryEvent {
            event_type: "plugin:registered".to_string(),
            plugin_id,
            data: HashMap::new(),
        });

        Ok(())
    }

    /// Unregisters a plugin.
    pub fn unregister(&self, plugin_id: &str) -> Result<(), RegistryError> {
        let manifest;
        {
            let plugins = self.plugins.read().unwrap();
            let registration = plugins
                .get(plugin_id)
                .ok_or_else(|| RegistryError::NotRegistered(plugin_id.to_string()))?;
            manifest = registration.plugin.manifest().clone();
        }

        // Check if other plugins depend on this one
        {
            let plugins = self.plugins.read().unwrap();
            for (other_id, other_reg) in plugins.iter() {
                if other_id != plugin_id {
                    for dep in &other_reg.plugin.manifest().dependencies {
                        if dep == plugin_id {
                            return Err(RegistryError::HasDependents(
                                plugin_id.to_string(),
                                other_id.clone(),
                            ));
                        }
                    }
                }
            }
        }

        // Get plugin for hook
        let plugin;
        {
            let plugins = self.plugins.read().unwrap();
            plugin = plugins.get(plugin_id).map(|r| r.plugin.clone());
        }

        // Call on_unload hook
        if let Some(p) = &plugin {
            if let Some(hooks) = p.hooks() {
                if let Some(on_unload) = &hooks.on_unload {
                    on_unload().map_err(|e| RegistryError::HookError(e.to_string()))?;
                }
            }
        }

        // Remove entity type mappings
        {
            let mut entity_map = self.entity_type_to_plugin.write().unwrap();
            for entity_type in &manifest.entity_types {
                entity_map.remove(entity_type);
            }
        }

        // Remove event type mappings
        {
            let mut event_map = self.event_type_to_plugin.write().unwrap();
            for event_type in &manifest.event_types {
                event_map.remove(event_type);
            }
        }

        // Remove plugin
        {
            let mut plugins = self.plugins.write().unwrap();
            plugins.remove(plugin_id);
        }

        self.emit(RegistryEvent {
            event_type: "plugin:unregistered".to_string(),
            plugin_id: plugin_id.to_string(),
            data: HashMap::new(),
        });

        Ok(())
    }

    /// Gets a plugin by ID.
    pub fn get_plugin(&self, plugin_id: &str) -> Option<Arc<dyn DomainPlugin>> {
        let plugins = self.plugins.read().unwrap();
        plugins.get(plugin_id).map(|r| r.plugin.clone())
    }

    /// Gets the plugin responsible for an entity type.
    pub fn get_plugin_for_entity_type(&self, entity_type: &str) -> Option<Arc<dyn DomainPlugin>> {
        let entity_map = self.entity_type_to_plugin.read().unwrap();
        entity_map
            .get(entity_type)
            .and_then(|plugin_id| self.get_plugin(plugin_id))
    }

    /// Gets the plugin responsible for an event type.
    pub fn get_plugin_for_event_type(&self, event_type: &str) -> Option<Arc<dyn DomainPlugin>> {
        let event_map = self.event_type_to_plugin.read().unwrap();
        event_map
            .get(event_type)
            .and_then(|plugin_id| self.get_plugin(plugin_id))
    }

    /// Gets all registered plugins.
    pub fn get_all_plugins(&self) -> Vec<Arc<dyn DomainPlugin>> {
        let plugins = self.plugins.read().unwrap();
        plugins
            .values()
            .filter(|r| r.status == "active")
            .map(|r| r.plugin.clone())
            .collect()
    }

    /// Gets all entity types from all plugins.
    pub fn get_all_entity_types(&self) -> Vec<PactEntityType> {
        let plugins = self.plugins.read().unwrap();
        plugins
            .values()
            .filter(|r| r.status == "active")
            .flat_map(|r| r.plugin.get_entity_types().to_vec())
            .collect()
    }

    /// Gets all default rules from all plugins.
    pub fn get_all_default_rules(&self) -> Vec<PactRule> {
        let plugins = self.plugins.read().unwrap();
        plugins
            .values()
            .filter(|r| r.status == "active")
            .flat_map(|r| r.plugin.get_default_rules().to_vec())
            .collect()
    }

    /// Checks if a plugin is registered.
    pub fn is_registered(&self, plugin_id: &str) -> bool {
        let plugins = self.plugins.read().unwrap();
        plugins.contains_key(plugin_id)
    }

    /// Gets plugin status.
    pub fn get_status(&self, plugin_id: &str) -> Option<String> {
        let plugins = self.plugins.read().unwrap();
        plugins.get(plugin_id).map(|r| r.status.clone())
    }

    /// Subscribes to registry events.
    pub fn on(&self, handler: RegistryEventHandler) {
        let mut handlers = self.event_handlers.write().unwrap();
        handlers.push(handler);
    }

    /// Emits an event.
    fn emit(&self, event: RegistryEvent) {
        let handlers = self.event_handlers.read().unwrap();
        for handler in handlers.iter() {
            handler(event.clone());
        }
    }

    /// Gets registry statistics.
    pub fn stats(&self) -> HashMap<String, serde_json::Value> {
        let plugins = self.plugins.read().unwrap();
        let entity_map = self.entity_type_to_plugin.read().unwrap();
        let event_map = self.event_type_to_plugin.read().unwrap();

        let active_plugins: Vec<String> = plugins
            .iter()
            .filter(|(_, r)| r.status == "active")
            .map(|(id, _)| id.clone())
            .collect();

        let mut stats = HashMap::new();
        stats.insert(
            "plugin_count".to_string(),
            serde_json::json!(plugins.len()),
        );
        stats.insert(
            "entity_type_count".to_string(),
            serde_json::json!(entity_map.len()),
        );
        stats.insert(
            "event_type_count".to_string(),
            serde_json::json!(event_map.len()),
        );
        stats.insert(
            "active_plugins".to_string(),
            serde_json::json!(active_plugins),
        );

        stats
    }

    /// Clears all plugins.
    pub fn clear(&self) {
        let mut plugins = self.plugins.write().unwrap();
        let mut entity_map = self.entity_type_to_plugin.write().unwrap();
        let mut event_map = self.event_type_to_plugin.write().unwrap();

        plugins.clear();
        entity_map.clear();
        event_map.clear();
    }
}

impl Default for Registry {
    fn default() -> Self {
        Self::new()
    }
}

// Global registry
lazy_static::lazy_static! {
    static ref GLOBAL_REGISTRY: Registry = Registry::new();
}

/// Gets the global registry.
pub fn get_global_registry() -> &'static Registry {
    &GLOBAL_REGISTRY
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::plugin::PluginHooks;
    use serde_json::json;
    use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};

    fn create_test_manifest(id: &str) -> PluginManifest {
        PluginManifest {
            id: id.to_string(),
            name: format!("{} Plugin", id),
            version: "1.0.0".to_string(),
            author: "Test".to_string(),
            description: "Test plugin".to_string(),
            entity_types: vec![format!("{}Entity", id)],
            event_types: vec![format!("{}_EVENT", id.to_uppercase())],
            dependencies: vec![],
            pact_core_version: "0.1.0".to_string(),
            config_schema: None,
        }
    }

    fn create_test_manifest_with_deps(id: &str, deps: Vec<&str>) -> PluginManifest {
        PluginManifest {
            id: id.to_string(),
            name: format!("{} Plugin", id),
            version: "1.0.0".to_string(),
            author: "Test".to_string(),
            description: "Test plugin".to_string(),
            entity_types: vec![format!("{}Entity", id)],
            event_types: vec![format!("{}_EVENT", id.to_uppercase())],
            dependencies: deps.into_iter().map(String::from).collect(),
            pact_core_version: "0.1.0".to_string(),
            config_schema: None,
        }
    }

    struct TestPlugin {
        manifest: PluginManifest,
        entity_types: Vec<PactEntityType>,
        rules: Vec<PactRule>,
        hooks: Option<PluginHooks>,
    }

    impl TestPlugin {
        fn new(manifest: PluginManifest) -> Self {
            let entity_type = PactEntityType {
                name: manifest.entity_types.first().cloned().unwrap_or_default(),
                description: "Test entity".to_string(),
                schema: HashMap::new(),
                required_fields: vec![],
            };

            Self {
                manifest,
                entity_types: vec![entity_type],
                rules: vec![],
                hooks: None,
            }
        }

        fn with_hooks(mut self, hooks: PluginHooks) -> Self {
            self.hooks = Some(hooks);
            self
        }
    }

    #[async_trait::async_trait]
    impl DomainPlugin for TestPlugin {
        fn manifest(&self) -> &PluginManifest {
            &self.manifest
        }

        fn hooks(&self) -> Option<&PluginHooks> {
            self.hooks.as_ref()
        }

        fn get_entity_types(&self) -> &[PactEntityType] {
            &self.entity_types
        }

        fn get_default_rules(&self) -> &[PactRule] {
            &self.rules
        }

        fn map_to_canonical_event(&self, event: &DomainEvent) -> Result<PactEvent, Box<dyn std::error::Error>> {
            let now = Utc::now();
            Ok(PactEvent {
                id: format!("evt_{}", uuid::Uuid::new_v4()),
                source: self.manifest.id.clone(),
                event_type: event.event_type.clone(),
                entity_id: event.entity_id.clone(),
                entity_type: None,
                jurisdiction: event.jurisdiction.clone(),
                payload: event.payload.clone(),
                occurred_at: event.occurred_at.unwrap_or(now),
                received_at: now,
                metadata: event.metadata.clone(),
            })
        }

        fn validate_domain_data(&self, _entity_type: &str, _data: &HashMap<String, serde_json::Value>) -> ValidationResult {
            ValidationResult::success()
        }
    }

    // =========================================================================
    // Registry Tests
    // =========================================================================

    #[test]
    fn test_registry_new() {
        let registry = Registry::new();
        assert_eq!(registry.get_all_plugins().len(), 0);
    }

    #[test]
    fn test_registry_default() {
        let registry = Registry::default();
        assert_eq!(registry.get_all_plugins().len(), 0);
    }

    #[test]
    fn test_registry_register_plugin() {
        let registry = Registry::new();
        let plugin = Arc::new(TestPlugin::new(create_test_manifest("test")));

        let result = registry.register(plugin);
        assert!(result.is_ok());
        assert!(registry.is_registered("test"));
    }

    #[test]
    fn test_registry_register_duplicate() {
        let registry = Registry::new();
        let plugin1 = Arc::new(TestPlugin::new(create_test_manifest("test")));
        let plugin2 = Arc::new(TestPlugin::new(create_test_manifest("test")));

        registry.register(plugin1).unwrap();
        let result = registry.register(plugin2);

        assert!(matches!(result, Err(RegistryError::AlreadyRegistered(_))));
    }

    #[test]
    fn test_registry_register_multiple_plugins() {
        let registry = Registry::new();
        let plugin1 = Arc::new(TestPlugin::new(create_test_manifest("plugin1")));
        let plugin2 = Arc::new(TestPlugin::new(create_test_manifest("plugin2")));

        registry.register(plugin1).unwrap();
        registry.register(plugin2).unwrap();

        assert_eq!(registry.get_all_plugins().len(), 2);
        assert!(registry.is_registered("plugin1"));
        assert!(registry.is_registered("plugin2"));
    }

    #[test]
    fn test_registry_unregister_plugin() {
        let registry = Registry::new();
        let plugin = Arc::new(TestPlugin::new(create_test_manifest("test")));

        registry.register(plugin).unwrap();
        assert!(registry.is_registered("test"));

        let result = registry.unregister("test");
        assert!(result.is_ok());
        assert!(!registry.is_registered("test"));
    }

    #[test]
    fn test_registry_unregister_not_registered() {
        let registry = Registry::new();

        let result = registry.unregister("nonexistent");
        assert!(matches!(result, Err(RegistryError::NotRegistered(_))));
    }

    #[test]
    fn test_registry_get_plugin() {
        let registry = Registry::new();
        let plugin = Arc::new(TestPlugin::new(create_test_manifest("test")));

        registry.register(plugin).unwrap();

        let retrieved = registry.get_plugin("test");
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().manifest().id, "test");
    }

    #[test]
    fn test_registry_get_plugin_not_found() {
        let registry = Registry::new();

        let retrieved = registry.get_plugin("nonexistent");
        assert!(retrieved.is_none());
    }

    // =========================================================================
    // Entity Type Mapping Tests
    // =========================================================================

    #[test]
    fn test_registry_get_plugin_for_entity_type() {
        let registry = Registry::new();
        let plugin = Arc::new(TestPlugin::new(create_test_manifest("test")));

        registry.register(plugin).unwrap();

        let retrieved = registry.get_plugin_for_entity_type("testEntity");
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().manifest().id, "test");
    }

    #[test]
    fn test_registry_get_plugin_for_entity_type_not_found() {
        let registry = Registry::new();

        let retrieved = registry.get_plugin_for_entity_type("UnknownEntity");
        assert!(retrieved.is_none());
    }

    #[test]
    fn test_registry_entity_type_conflict() {
        let registry = Registry::new();

        // First plugin with entity type "SharedEntity"
        let mut manifest1 = create_test_manifest("plugin1");
        manifest1.entity_types = vec!["SharedEntity".to_string()];
        let plugin1 = Arc::new(TestPlugin::new(manifest1));

        // Second plugin with same entity type
        let mut manifest2 = create_test_manifest("plugin2");
        manifest2.entity_types = vec!["SharedEntity".to_string()];
        let plugin2 = Arc::new(TestPlugin::new(manifest2));

        registry.register(plugin1).unwrap();
        let result = registry.register(plugin2);

        assert!(matches!(result, Err(RegistryError::EntityTypeConflict(_, _))));
    }

    // =========================================================================
    // Event Type Mapping Tests
    // =========================================================================

    #[test]
    fn test_registry_get_plugin_for_event_type() {
        let registry = Registry::new();
        let plugin = Arc::new(TestPlugin::new(create_test_manifest("test")));

        registry.register(plugin).unwrap();

        let retrieved = registry.get_plugin_for_event_type("TEST_EVENT");
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().manifest().id, "test");
    }

    #[test]
    fn test_registry_get_plugin_for_event_type_not_found() {
        let registry = Registry::new();

        let retrieved = registry.get_plugin_for_event_type("UNKNOWN_EVENT");
        assert!(retrieved.is_none());
    }

    #[test]
    fn test_registry_event_type_conflict() {
        let registry = Registry::new();

        let mut manifest1 = create_test_manifest("plugin1");
        manifest1.event_types = vec!["SHARED_EVENT".to_string()];
        let plugin1 = Arc::new(TestPlugin::new(manifest1));

        let mut manifest2 = create_test_manifest("plugin2");
        manifest2.event_types = vec!["SHARED_EVENT".to_string()];
        manifest2.entity_types = vec!["Plugin2Entity".to_string()]; // Different entity type
        let plugin2 = Arc::new(TestPlugin::new(manifest2));

        registry.register(plugin1).unwrap();
        let result = registry.register(plugin2);

        assert!(matches!(result, Err(RegistryError::EventTypeConflict(_, _))));
    }

    // =========================================================================
    // Dependency Tests
    // =========================================================================

    #[test]
    fn test_registry_dependency_satisfied() {
        let registry = Registry::new();

        let base_plugin = Arc::new(TestPlugin::new(create_test_manifest("base")));
        let dependent_plugin = Arc::new(TestPlugin::new(
            create_test_manifest_with_deps("dependent", vec!["base"])
        ));

        registry.register(base_plugin).unwrap();
        let result = registry.register(dependent_plugin);

        assert!(result.is_ok());
    }

    #[test]
    fn test_registry_dependency_missing() {
        let registry = Registry::new();

        let plugin = Arc::new(TestPlugin::new(
            create_test_manifest_with_deps("plugin", vec!["nonexistent"])
        ));

        let result = registry.register(plugin);

        assert!(matches!(result, Err(RegistryError::MissingDependency(_, _))));
    }

    #[test]
    fn test_registry_cannot_unregister_with_dependents() {
        let registry = Registry::new();

        let base_plugin = Arc::new(TestPlugin::new(create_test_manifest("base")));
        let dependent_plugin = Arc::new(TestPlugin::new(
            create_test_manifest_with_deps("dependent", vec!["base"])
        ));

        registry.register(base_plugin).unwrap();
        registry.register(dependent_plugin).unwrap();

        let result = registry.unregister("base");

        assert!(matches!(result, Err(RegistryError::HasDependents(_, _))));
    }

    #[test]
    fn test_registry_can_unregister_after_dependent_removed() {
        let registry = Registry::new();

        let base_plugin = Arc::new(TestPlugin::new(create_test_manifest("base")));
        let dependent_plugin = Arc::new(TestPlugin::new(
            create_test_manifest_with_deps("dependent", vec!["base"])
        ));

        registry.register(base_plugin).unwrap();
        registry.register(dependent_plugin).unwrap();

        // Remove dependent first
        registry.unregister("dependent").unwrap();

        // Now we can remove base
        let result = registry.unregister("base");
        assert!(result.is_ok());
    }

    // =========================================================================
    // Hooks Tests
    // =========================================================================

    #[test]
    fn test_registry_calls_on_load_hook() {
        let registry = Registry::new();
        let called = Arc::new(AtomicBool::new(false));
        let called_clone = called.clone();

        let hooks = PluginHooks {
            on_load: Some(Box::new(move || {
                called_clone.store(true, Ordering::SeqCst);
                Ok(())
            })),
            on_unload: None,
        };

        let plugin = Arc::new(TestPlugin::new(create_test_manifest("test")).with_hooks(hooks));

        registry.register(plugin).unwrap();

        assert!(called.load(Ordering::SeqCst));
    }

    #[test]
    fn test_registry_on_load_hook_error() {
        let registry = Registry::new();

        let hooks = PluginHooks {
            on_load: Some(Box::new(|| {
                Err("Load failed".into())
            })),
            on_unload: None,
        };

        let plugin = Arc::new(TestPlugin::new(create_test_manifest("test")).with_hooks(hooks));

        let result = registry.register(plugin);

        assert!(matches!(result, Err(RegistryError::HookError(_))));
        assert!(!registry.is_registered("test"));
    }

    #[test]
    fn test_registry_calls_on_unload_hook() {
        let registry = Registry::new();
        let called = Arc::new(AtomicBool::new(false));
        let called_clone = called.clone();

        let hooks = PluginHooks {
            on_load: None,
            on_unload: Some(Box::new(move || {
                called_clone.store(true, Ordering::SeqCst);
                Ok(())
            })),
        };

        let plugin = Arc::new(TestPlugin::new(create_test_manifest("test")).with_hooks(hooks));

        registry.register(plugin).unwrap();
        registry.unregister("test").unwrap();

        assert!(called.load(Ordering::SeqCst));
    }

    // =========================================================================
    // Status Tests
    // =========================================================================

    #[test]
    fn test_registry_get_status() {
        let registry = Registry::new();
        let plugin = Arc::new(TestPlugin::new(create_test_manifest("test")));

        registry.register(plugin).unwrap();

        let status = registry.get_status("test");
        assert_eq!(status, Some("active".to_string()));
    }

    #[test]
    fn test_registry_get_status_not_found() {
        let registry = Registry::new();

        let status = registry.get_status("nonexistent");
        assert!(status.is_none());
    }

    // =========================================================================
    // Stats Tests
    // =========================================================================

    #[test]
    fn test_registry_stats_empty() {
        let registry = Registry::new();

        let stats = registry.stats();

        assert_eq!(stats.get("plugin_count"), Some(&json!(0)));
        assert_eq!(stats.get("entity_type_count"), Some(&json!(0)));
        assert_eq!(stats.get("event_type_count"), Some(&json!(0)));
    }

    #[test]
    fn test_registry_stats_with_plugins() {
        let registry = Registry::new();
        let plugin1 = Arc::new(TestPlugin::new(create_test_manifest("plugin1")));
        let plugin2 = Arc::new(TestPlugin::new(create_test_manifest("plugin2")));

        registry.register(plugin1).unwrap();
        registry.register(plugin2).unwrap();

        let stats = registry.stats();

        assert_eq!(stats.get("plugin_count"), Some(&json!(2)));
        assert_eq!(stats.get("entity_type_count"), Some(&json!(2)));
        assert_eq!(stats.get("event_type_count"), Some(&json!(2)));

        let active_plugins = stats.get("active_plugins").unwrap().as_array().unwrap();
        assert_eq!(active_plugins.len(), 2);
    }

    // =========================================================================
    // Clear Tests
    // =========================================================================

    #[test]
    fn test_registry_clear() {
        let registry = Registry::new();
        let plugin1 = Arc::new(TestPlugin::new(create_test_manifest("plugin1")));
        let plugin2 = Arc::new(TestPlugin::new(create_test_manifest("plugin2")));

        registry.register(plugin1).unwrap();
        registry.register(plugin2).unwrap();

        assert_eq!(registry.get_all_plugins().len(), 2);

        registry.clear();

        assert_eq!(registry.get_all_plugins().len(), 0);
        assert!(!registry.is_registered("plugin1"));
        assert!(!registry.is_registered("plugin2"));
    }

    // =========================================================================
    // Get All Tests
    // =========================================================================

    #[test]
    fn test_registry_get_all_entity_types() {
        let registry = Registry::new();
        let plugin1 = Arc::new(TestPlugin::new(create_test_manifest("plugin1")));
        let plugin2 = Arc::new(TestPlugin::new(create_test_manifest("plugin2")));

        registry.register(plugin1).unwrap();
        registry.register(plugin2).unwrap();

        let entity_types = registry.get_all_entity_types();
        assert_eq!(entity_types.len(), 2);
    }

    #[test]
    fn test_registry_get_all_default_rules() {
        let registry = Registry::new();

        // Create a plugin with rules
        let manifest = create_test_manifest("test");
        let mut plugin = TestPlugin::new(manifest);
        plugin.rules = vec![
            PactRule {
                id: "rule1".to_string(),
                name: "Rule 1".to_string(),
                description: "Test rule".to_string(),
                jurisdiction: None,
                severity: Severity::Medium,
                condition: Condition::Compare {
                    field: "x".to_string(),
                    operator: "eq".to_string(),
                    value: json!(1),
                },
                consequence: Consequence {
                    decision: Decision::Allow,
                    code: "OK".to_string(),
                    message: "Ok".to_string(),
                    metadata: HashMap::new(),
                },
                tags: vec![],
                effective_from: None,
                effective_to: None,
                metadata: HashMap::new(),
            },
        ];

        registry.register(Arc::new(plugin)).unwrap();

        let rules = registry.get_all_default_rules();
        assert_eq!(rules.len(), 1);
        assert_eq!(rules[0].id, "rule1");
    }

    // =========================================================================
    // Event Handler Tests
    // =========================================================================

    #[test]
    fn test_registry_event_handler() {
        let registry = Registry::new();
        let event_count = Arc::new(AtomicUsize::new(0));
        let event_count_clone = event_count.clone();

        registry.on(Box::new(move |_event| {
            event_count_clone.fetch_add(1, Ordering::SeqCst);
        }));

        let plugin = Arc::new(TestPlugin::new(create_test_manifest("test")));
        registry.register(plugin).unwrap();

        // Should have received events for:
        // - entity_type:registered
        // - plugin:registered
        assert!(event_count.load(Ordering::SeqCst) >= 2);
    }

    #[test]
    fn test_registry_event_handler_on_unregister() {
        let registry = Registry::new();
        let unregister_received = Arc::new(AtomicBool::new(false));
        let unregister_clone = unregister_received.clone();

        registry.on(Box::new(move |event| {
            if event.event_type == "plugin:unregistered" {
                unregister_clone.store(true, Ordering::SeqCst);
            }
        }));

        let plugin = Arc::new(TestPlugin::new(create_test_manifest("test")));
        registry.register(plugin).unwrap();
        registry.unregister("test").unwrap();

        assert!(unregister_received.load(Ordering::SeqCst));
    }

    // =========================================================================
    // RegistryError Tests
    // =========================================================================

    #[test]
    fn test_registry_error_display() {
        let err = RegistryError::AlreadyRegistered("test".to_string());
        assert_eq!(err.to_string(), "Plugin 'test' is already registered");

        let err = RegistryError::NotRegistered("test".to_string());
        assert_eq!(err.to_string(), "Plugin 'test' is not registered");

        let err = RegistryError::MissingDependency("plugin".to_string(), "dep".to_string());
        assert_eq!(err.to_string(), "Plugin 'plugin' requires 'dep' which is not registered");

        let err = RegistryError::EntityTypeConflict("Entity".to_string(), "plugin".to_string());
        assert_eq!(err.to_string(), "Entity type 'Entity' is already registered by plugin 'plugin'");

        let err = RegistryError::EventTypeConflict("EVENT".to_string(), "plugin".to_string());
        assert_eq!(err.to_string(), "Event type 'EVENT' is already registered by plugin 'plugin'");

        let err = RegistryError::HasDependents("base".to_string(), "dependent".to_string());
        assert_eq!(err.to_string(), "Cannot unregister 'base': plugin 'dependent' depends on it");

        let err = RegistryError::HookError("error".to_string());
        assert_eq!(err.to_string(), "Hook error: error");
    }

    // =========================================================================
    // RegistryEvent Tests
    // =========================================================================

    #[test]
    fn test_registry_event_clone() {
        let event = RegistryEvent {
            event_type: "plugin:registered".to_string(),
            plugin_id: "test".to_string(),
            data: [("key".to_string(), "value".to_string())].into_iter().collect(),
        };

        let cloned = event.clone();

        assert_eq!(cloned.event_type, event.event_type);
        assert_eq!(cloned.plugin_id, event.plugin_id);
        assert_eq!(cloned.data.get("key"), Some(&"value".to_string()));
    }

    // =========================================================================
    // Global Registry Tests
    // =========================================================================

    #[test]
    fn test_get_global_registry() {
        let registry = get_global_registry();
        // Just verify we can access it without panic
        let _ = registry.stats();
    }
}
