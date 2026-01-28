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
