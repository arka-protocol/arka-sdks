package pact

import (
	"context"
	"fmt"
	"sync"
	"time"
)

// PluginRegistration represents a registered plugin.
type PluginRegistration struct {
	Plugin       DomainPlugin
	RegisteredAt time.Time
	Status       string
	Error        string
}

// RegistryEvent represents a registry event.
type RegistryEvent struct {
	Type     string
	PluginID string
	Data     map[string]any
}

// RegistryEventHandler handles registry events.
type RegistryEventHandler func(event RegistryEvent)

// Registry manages domain plugins.
type Registry struct {
	mu                  sync.RWMutex
	plugins             map[string]*PluginRegistration
	entityTypeToPlugin  map[string]string
	eventTypeToPlugin   map[string]string
	eventHandlers       []RegistryEventHandler
}

// NewRegistry creates a new plugin registry.
func NewRegistry() *Registry {
	return &Registry{
		plugins:            make(map[string]*PluginRegistration),
		entityTypeToPlugin: make(map[string]string),
		eventTypeToPlugin:  make(map[string]string),
		eventHandlers:      []RegistryEventHandler{},
	}
}

// Register registers a domain plugin.
func (r *Registry) Register(ctx context.Context, plugin DomainPlugin) error {
	r.mu.Lock()
	defer r.mu.Unlock()

	manifest := plugin.Manifest()

	// Check for duplicate registration
	if _, exists := r.plugins[manifest.ID]; exists {
		return fmt.Errorf("plugin '%s' is already registered", manifest.ID)
	}

	// Check dependencies
	for _, dep := range manifest.Dependencies {
		if _, exists := r.plugins[dep]; !exists {
			return fmt.Errorf("plugin '%s' requires '%s' which is not registered", manifest.ID, dep)
		}
	}

	// Check for entity type conflicts
	for _, entityType := range manifest.EntityTypes {
		if existing, exists := r.entityTypeToPlugin[entityType]; exists {
			return fmt.Errorf("entity type '%s' is already registered by plugin '%s'", entityType, existing)
		}
	}

	// Check for event type conflicts
	for _, eventType := range manifest.EventTypes {
		if existing, exists := r.eventTypeToPlugin[eventType]; exists {
			return fmt.Errorf("event type '%s' is already registered by plugin '%s'", eventType, existing)
		}
	}

	// Call on_load hook
	if hooks := plugin.Hooks(); hooks != nil && hooks.OnLoad != nil {
		if err := hooks.OnLoad(ctx); err != nil {
			return fmt.Errorf("plugin on_load failed: %w", err)
		}
	}

	// Register plugin
	registration := &PluginRegistration{
		Plugin:       plugin,
		RegisteredAt: time.Now().UTC(),
		Status:       "active",
	}
	r.plugins[manifest.ID] = registration

	// Map entity types to plugin
	for _, entityType := range manifest.EntityTypes {
		r.entityTypeToPlugin[entityType] = manifest.ID
		r.emit(RegistryEvent{
			Type:     "entity_type:registered",
			PluginID: manifest.ID,
			Data:     map[string]any{"entity_type": entityType},
		})
	}

	// Map event types to plugin
	for _, eventType := range manifest.EventTypes {
		r.eventTypeToPlugin[eventType] = manifest.ID
	}

	r.emit(RegistryEvent{
		Type:     "plugin:registered",
		PluginID: manifest.ID,
		Data:     map[string]any{"manifest": manifest},
	})

	return nil
}

// Unregister unregisters a plugin.
func (r *Registry) Unregister(ctx context.Context, pluginID string) error {
	r.mu.Lock()
	defer r.mu.Unlock()

	registration, exists := r.plugins[pluginID]
	if !exists {
		return fmt.Errorf("plugin '%s' is not registered", pluginID)
	}

	// Check if other plugins depend on this one
	for otherID, otherReg := range r.plugins {
		if otherID != pluginID {
			deps := otherReg.Plugin.Manifest().Dependencies
			for _, dep := range deps {
				if dep == pluginID {
					return fmt.Errorf("cannot unregister '%s': plugin '%s' depends on it", pluginID, otherID)
				}
			}
		}
	}

	// Call on_unload hook
	if hooks := registration.Plugin.Hooks(); hooks != nil && hooks.OnUnload != nil {
		if err := hooks.OnUnload(ctx); err != nil {
			return fmt.Errorf("plugin on_unload failed: %w", err)
		}
	}

	// Remove entity type mappings
	for _, entityType := range registration.Plugin.Manifest().EntityTypes {
		delete(r.entityTypeToPlugin, entityType)
	}

	// Remove event type mappings
	for _, eventType := range registration.Plugin.Manifest().EventTypes {
		delete(r.eventTypeToPlugin, eventType)
	}

	// Remove plugin
	delete(r.plugins, pluginID)

	r.emit(RegistryEvent{
		Type:     "plugin:unregistered",
		PluginID: pluginID,
	})

	return nil
}

// GetPlugin gets a plugin by ID.
func (r *Registry) GetPlugin(pluginID string) DomainPlugin {
	r.mu.RLock()
	defer r.mu.RUnlock()

	if reg, exists := r.plugins[pluginID]; exists {
		return reg.Plugin
	}
	return nil
}

// GetPluginForEntityType gets the plugin responsible for an entity type.
func (r *Registry) GetPluginForEntityType(entityType string) DomainPlugin {
	r.mu.RLock()
	defer r.mu.RUnlock()

	if pluginID, exists := r.entityTypeToPlugin[entityType]; exists {
		if reg, exists := r.plugins[pluginID]; exists {
			return reg.Plugin
		}
	}
	return nil
}

// GetPluginForEventType gets the plugin responsible for an event type.
func (r *Registry) GetPluginForEventType(eventType string) DomainPlugin {
	r.mu.RLock()
	defer r.mu.RUnlock()

	if pluginID, exists := r.eventTypeToPlugin[eventType]; exists {
		if reg, exists := r.plugins[pluginID]; exists {
			return reg.Plugin
		}
	}
	return nil
}

// GetAllPlugins gets all registered plugins.
func (r *Registry) GetAllPlugins() []DomainPlugin {
	r.mu.RLock()
	defer r.mu.RUnlock()

	plugins := make([]DomainPlugin, 0, len(r.plugins))
	for _, reg := range r.plugins {
		if reg.Status == "active" {
			plugins = append(plugins, reg.Plugin)
		}
	}
	return plugins
}

// GetAllEntityTypes gets all entity types from all plugins.
func (r *Registry) GetAllEntityTypes() []PactEntityType {
	r.mu.RLock()
	defer r.mu.RUnlock()

	var types []PactEntityType
	for _, reg := range r.plugins {
		if reg.Status == "active" {
			types = append(types, reg.Plugin.GetEntityTypes()...)
		}
	}
	return types
}

// GetAllDefaultRules gets all default rules from all plugins.
func (r *Registry) GetAllDefaultRules() []PactRule {
	r.mu.RLock()
	defer r.mu.RUnlock()

	var rules []PactRule
	for _, reg := range r.plugins {
		if reg.Status == "active" {
			rules = append(rules, reg.Plugin.GetDefaultRules()...)
		}
	}
	return rules
}

// IsRegistered checks if a plugin is registered.
func (r *Registry) IsRegistered(pluginID string) bool {
	r.mu.RLock()
	defer r.mu.RUnlock()

	_, exists := r.plugins[pluginID]
	return exists
}

// GetStatus gets plugin status.
func (r *Registry) GetStatus(pluginID string) string {
	r.mu.RLock()
	defer r.mu.RUnlock()

	if reg, exists := r.plugins[pluginID]; exists {
		return reg.Status
	}
	return ""
}

// On subscribes to registry events.
func (r *Registry) On(handler RegistryEventHandler) {
	r.mu.Lock()
	defer r.mu.Unlock()

	r.eventHandlers = append(r.eventHandlers, handler)
}

// emit emits an event to all handlers.
func (r *Registry) emit(event RegistryEvent) {
	for _, handler := range r.eventHandlers {
		handler(event)
	}
}

// Stats returns registry statistics.
func (r *Registry) Stats() map[string]any {
	r.mu.RLock()
	defer r.mu.RUnlock()

	activePlugins := []string{}
	for id, reg := range r.plugins {
		if reg.Status == "active" {
			activePlugins = append(activePlugins, id)
		}
	}

	return map[string]any{
		"plugin_count":      len(r.plugins),
		"entity_type_count": len(r.entityTypeToPlugin),
		"event_type_count":  len(r.eventTypeToPlugin),
		"active_plugins":    activePlugins,
	}
}

// Clear clears all plugins.
func (r *Registry) Clear() {
	r.mu.Lock()
	defer r.mu.Unlock()

	r.plugins = make(map[string]*PluginRegistration)
	r.entityTypeToPlugin = make(map[string]string)
	r.eventTypeToPlugin = make(map[string]string)
}

// Global registry instance
var globalRegistry *Registry
var registryOnce sync.Once

// GetGlobalRegistry returns the global registry instance.
func GetGlobalRegistry() *Registry {
	registryOnce.Do(func() {
		globalRegistry = NewRegistry()
	})
	return globalRegistry
}

// ResetGlobalRegistry resets the global registry.
func ResetGlobalRegistry() {
	if globalRegistry != nil {
		globalRegistry.Clear()
	}
	globalRegistry = nil
	registryOnce = sync.Once{}
}
