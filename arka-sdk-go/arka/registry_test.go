package pact

import (
	"context"
	"sync"
	"testing"
)

// mockPlugin implements DomainPlugin for testing
type mockPlugin struct {
	manifest     PluginManifest
	hooks        *PluginHooks
	entityTypes  []PactEntityType
	defaultRules []PactRule
}

func newMockPlugin(id string, entityTypes, eventTypes []string) *mockPlugin {
	return &mockPlugin{
		manifest: PluginManifest{
			ID:          id,
			Name:        id + " Plugin",
			Version:     "1.0.0",
			EntityTypes: entityTypes,
			EventTypes:  eventTypes,
		},
	}
}

func (p *mockPlugin) Manifest() PluginManifest                        { return p.manifest }
func (p *mockPlugin) Hooks() *PluginHooks                             { return p.hooks }
func (p *mockPlugin) GetEntityTypes() []PactEntityType                { return p.entityTypes }
func (p *mockPlugin) GetDefaultRules() []PactRule                     { return p.defaultRules }
func (p *mockPlugin) MapToCanonicalEvent(DomainEvent) (*PactEvent, error) { return nil, nil }
func (p *mockPlugin) ValidateDomainData(string, map[string]interface{}) ValidationResult {
	return ValidationResult{Valid: true}
}
func (p *mockPlugin) GetEvaluationContext(*PactEvent, *PactEntity) map[string]interface{} {
	return nil
}
func (p *mockPlugin) SerializeForChain(interface{}) ([]byte, error) { return nil, nil }
func (p *mockPlugin) DeserializeFromChain([]byte) (interface{}, error) { return nil, nil }

func TestNewRegistry(t *testing.T) {
	registry := NewRegistry()

	if registry == nil {
		t.Fatal("expected non-nil registry")
	}
	if registry.plugins == nil {
		t.Error("expected non-nil plugins map")
	}
	if registry.entityTypeToPlugin == nil {
		t.Error("expected non-nil entityTypeToPlugin map")
	}
	if registry.eventTypeToPlugin == nil {
		t.Error("expected non-nil eventTypeToPlugin map")
	}
}

func TestRegistryRegister(t *testing.T) {
	t.Run("registers plugin successfully", func(t *testing.T) {
		registry := NewRegistry()
		plugin := newMockPlugin("test-plugin", []string{"User"}, []string{"user_created"})

		err := registry.Register(context.Background(), plugin)
		if err != nil {
			t.Fatalf("Register failed: %v", err)
		}

		if !registry.IsRegistered("test-plugin") {
			t.Error("expected plugin to be registered")
		}
	})

	t.Run("prevents duplicate registration", func(t *testing.T) {
		registry := NewRegistry()
		plugin := newMockPlugin("test-plugin", []string{"User"}, []string{"user_created"})

		registry.Register(context.Background(), plugin)
		err := registry.Register(context.Background(), plugin)

		if err == nil {
			t.Error("expected error for duplicate registration")
		}
	})

	t.Run("prevents entity type conflicts", func(t *testing.T) {
		registry := NewRegistry()
		plugin1 := newMockPlugin("plugin1", []string{"User"}, []string{})
		plugin2 := newMockPlugin("plugin2", []string{"User"}, []string{})

		registry.Register(context.Background(), plugin1)
		err := registry.Register(context.Background(), plugin2)

		if err == nil {
			t.Error("expected error for entity type conflict")
		}
	})

	t.Run("prevents event type conflicts", func(t *testing.T) {
		registry := NewRegistry()
		plugin1 := newMockPlugin("plugin1", []string{}, []string{"user_created"})
		plugin2 := newMockPlugin("plugin2", []string{}, []string{"user_created"})

		registry.Register(context.Background(), plugin1)
		err := registry.Register(context.Background(), plugin2)

		if err == nil {
			t.Error("expected error for event type conflict")
		}
	})

	t.Run("checks dependencies", func(t *testing.T) {
		registry := NewRegistry()
		plugin := newMockPlugin("dependent-plugin", []string{}, []string{})
		plugin.manifest.Dependencies = []string{"missing-plugin"}

		err := registry.Register(context.Background(), plugin)

		if err == nil {
			t.Error("expected error for missing dependency")
		}
	})

	t.Run("registers with satisfied dependencies", func(t *testing.T) {
		registry := NewRegistry()
		basePlugin := newMockPlugin("base-plugin", []string{"Base"}, []string{})
		dependentPlugin := newMockPlugin("dependent-plugin", []string{"Dependent"}, []string{})
		dependentPlugin.manifest.Dependencies = []string{"base-plugin"}

		registry.Register(context.Background(), basePlugin)
		err := registry.Register(context.Background(), dependentPlugin)

		if err != nil {
			t.Fatalf("Register failed: %v", err)
		}
	})

	t.Run("calls OnLoad hook", func(t *testing.T) {
		registry := NewRegistry()
		plugin := newMockPlugin("test-plugin", []string{}, []string{})

		onLoadCalled := false
		plugin.hooks = &PluginHooks{
			OnLoad: func(ctx context.Context) error {
				onLoadCalled = true
				return nil
			},
		}

		registry.Register(context.Background(), plugin)

		if !onLoadCalled {
			t.Error("expected OnLoad to be called")
		}
	})

	t.Run("handles OnLoad error", func(t *testing.T) {
		registry := NewRegistry()
		plugin := newMockPlugin("test-plugin", []string{}, []string{})

		plugin.hooks = &PluginHooks{
			OnLoad: func(ctx context.Context) error {
				return context.DeadlineExceeded
			},
		}

		err := registry.Register(context.Background(), plugin)

		if err == nil {
			t.Error("expected error from OnLoad failure")
		}
	})
}

func TestRegistryUnregister(t *testing.T) {
	t.Run("unregisters plugin successfully", func(t *testing.T) {
		registry := NewRegistry()
		plugin := newMockPlugin("test-plugin", []string{"User"}, []string{"user_created"})

		registry.Register(context.Background(), plugin)
		err := registry.Unregister(context.Background(), "test-plugin")

		if err != nil {
			t.Fatalf("Unregister failed: %v", err)
		}
		if registry.IsRegistered("test-plugin") {
			t.Error("expected plugin to be unregistered")
		}
	})

	t.Run("fails for non-existent plugin", func(t *testing.T) {
		registry := NewRegistry()

		err := registry.Unregister(context.Background(), "nonexistent")

		if err == nil {
			t.Error("expected error for non-existent plugin")
		}
	})

	t.Run("prevents unregistering with dependents", func(t *testing.T) {
		registry := NewRegistry()
		basePlugin := newMockPlugin("base-plugin", []string{"Base"}, []string{})
		dependentPlugin := newMockPlugin("dependent-plugin", []string{"Dependent"}, []string{})
		dependentPlugin.manifest.Dependencies = []string{"base-plugin"}

		registry.Register(context.Background(), basePlugin)
		registry.Register(context.Background(), dependentPlugin)

		err := registry.Unregister(context.Background(), "base-plugin")

		if err == nil {
			t.Error("expected error for plugin with dependents")
		}
	})

	t.Run("calls OnUnload hook", func(t *testing.T) {
		registry := NewRegistry()
		plugin := newMockPlugin("test-plugin", []string{}, []string{})

		onUnloadCalled := false
		plugin.hooks = &PluginHooks{
			OnUnload: func(ctx context.Context) error {
				onUnloadCalled = true
				return nil
			},
		}

		registry.Register(context.Background(), plugin)
		registry.Unregister(context.Background(), "test-plugin")

		if !onUnloadCalled {
			t.Error("expected OnUnload to be called")
		}
	})

	t.Run("removes entity type mappings", func(t *testing.T) {
		registry := NewRegistry()
		plugin := newMockPlugin("test-plugin", []string{"User"}, []string{})

		registry.Register(context.Background(), plugin)
		registry.Unregister(context.Background(), "test-plugin")

		if registry.GetPluginForEntityType("User") != nil {
			t.Error("expected entity type mapping to be removed")
		}
	})

	t.Run("removes event type mappings", func(t *testing.T) {
		registry := NewRegistry()
		plugin := newMockPlugin("test-plugin", []string{}, []string{"user_created"})

		registry.Register(context.Background(), plugin)
		registry.Unregister(context.Background(), "test-plugin")

		if registry.GetPluginForEventType("user_created") != nil {
			t.Error("expected event type mapping to be removed")
		}
	})
}

func TestRegistryGetPlugin(t *testing.T) {
	t.Run("returns registered plugin", func(t *testing.T) {
		registry := NewRegistry()
		plugin := newMockPlugin("test-plugin", []string{}, []string{})

		registry.Register(context.Background(), plugin)
		retrieved := registry.GetPlugin("test-plugin")

		if retrieved == nil {
			t.Fatal("expected non-nil plugin")
		}
		if retrieved.Manifest().ID != "test-plugin" {
			t.Errorf("expected ID %q, got %q", "test-plugin", retrieved.Manifest().ID)
		}
	})

	t.Run("returns nil for non-existent plugin", func(t *testing.T) {
		registry := NewRegistry()

		retrieved := registry.GetPlugin("nonexistent")

		if retrieved != nil {
			t.Error("expected nil for non-existent plugin")
		}
	})
}

func TestRegistryGetPluginForEntityType(t *testing.T) {
	t.Run("returns plugin for entity type", func(t *testing.T) {
		registry := NewRegistry()
		plugin := newMockPlugin("test-plugin", []string{"User", "Account"}, []string{})

		registry.Register(context.Background(), plugin)

		retrieved := registry.GetPluginForEntityType("User")
		if retrieved == nil {
			t.Fatal("expected non-nil plugin")
		}
		if retrieved.Manifest().ID != "test-plugin" {
			t.Errorf("expected ID %q, got %q", "test-plugin", retrieved.Manifest().ID)
		}

		retrieved = registry.GetPluginForEntityType("Account")
		if retrieved == nil {
			t.Fatal("expected non-nil plugin for Account")
		}
	})

	t.Run("returns nil for unknown entity type", func(t *testing.T) {
		registry := NewRegistry()

		retrieved := registry.GetPluginForEntityType("Unknown")

		if retrieved != nil {
			t.Error("expected nil for unknown entity type")
		}
	})
}

func TestRegistryGetPluginForEventType(t *testing.T) {
	t.Run("returns plugin for event type", func(t *testing.T) {
		registry := NewRegistry()
		plugin := newMockPlugin("test-plugin", []string{}, []string{"user_created", "user_updated"})

		registry.Register(context.Background(), plugin)

		retrieved := registry.GetPluginForEventType("user_created")
		if retrieved == nil {
			t.Fatal("expected non-nil plugin")
		}
		if retrieved.Manifest().ID != "test-plugin" {
			t.Errorf("expected ID %q, got %q", "test-plugin", retrieved.Manifest().ID)
		}
	})

	t.Run("returns nil for unknown event type", func(t *testing.T) {
		registry := NewRegistry()

		retrieved := registry.GetPluginForEventType("unknown_event")

		if retrieved != nil {
			t.Error("expected nil for unknown event type")
		}
	})
}

func TestRegistryGetAllPlugins(t *testing.T) {
	t.Run("returns all active plugins", func(t *testing.T) {
		registry := NewRegistry()
		plugin1 := newMockPlugin("plugin1", []string{"Type1"}, []string{})
		plugin2 := newMockPlugin("plugin2", []string{"Type2"}, []string{})

		registry.Register(context.Background(), plugin1)
		registry.Register(context.Background(), plugin2)

		plugins := registry.GetAllPlugins()

		if len(plugins) != 2 {
			t.Errorf("expected 2 plugins, got %d", len(plugins))
		}
	})

	t.Run("returns empty slice when no plugins", func(t *testing.T) {
		registry := NewRegistry()

		plugins := registry.GetAllPlugins()

		if len(plugins) != 0 {
			t.Errorf("expected 0 plugins, got %d", len(plugins))
		}
	})
}

func TestRegistryGetAllEntityTypes(t *testing.T) {
	t.Run("returns all entity types from all plugins", func(t *testing.T) {
		registry := NewRegistry()
		plugin := newMockPlugin("test-plugin", []string{}, []string{})
		plugin.entityTypes = []PactEntityType{
			{Name: "User", Description: "A user"},
			{Name: "Account", Description: "An account"},
		}

		registry.Register(context.Background(), plugin)

		types := registry.GetAllEntityTypes()

		if len(types) != 2 {
			t.Errorf("expected 2 entity types, got %d", len(types))
		}
	})
}

func TestRegistryGetAllDefaultRules(t *testing.T) {
	t.Run("returns all default rules from all plugins", func(t *testing.T) {
		registry := NewRegistry()
		plugin := newMockPlugin("test-plugin", []string{}, []string{})
		plugin.defaultRules = []PactRule{
			{ID: "rule1", Name: "Rule 1"},
			{ID: "rule2", Name: "Rule 2"},
		}

		registry.Register(context.Background(), plugin)

		rules := registry.GetAllDefaultRules()

		if len(rules) != 2 {
			t.Errorf("expected 2 rules, got %d", len(rules))
		}
	})
}

func TestRegistryGetStatus(t *testing.T) {
	t.Run("returns status for registered plugin", func(t *testing.T) {
		registry := NewRegistry()
		plugin := newMockPlugin("test-plugin", []string{}, []string{})

		registry.Register(context.Background(), plugin)

		status := registry.GetStatus("test-plugin")

		if status != "active" {
			t.Errorf("expected status %q, got %q", "active", status)
		}
	})

	t.Run("returns empty string for non-existent plugin", func(t *testing.T) {
		registry := NewRegistry()

		status := registry.GetStatus("nonexistent")

		if status != "" {
			t.Errorf("expected empty status, got %q", status)
		}
	})
}

func TestRegistryEventHandlers(t *testing.T) {
	t.Run("emits events to handlers", func(t *testing.T) {
		registry := NewRegistry()
		plugin := newMockPlugin("test-plugin", []string{"User"}, []string{})

		var events []RegistryEvent
		registry.On(func(event RegistryEvent) {
			events = append(events, event)
		})

		registry.Register(context.Background(), plugin)

		// Should have entity_type:registered and plugin:registered events
		if len(events) < 2 {
			t.Errorf("expected at least 2 events, got %d", len(events))
		}

		foundPluginRegistered := false
		foundEntityTypeRegistered := false
		for _, e := range events {
			if e.Type == "plugin:registered" {
				foundPluginRegistered = true
			}
			if e.Type == "entity_type:registered" {
				foundEntityTypeRegistered = true
			}
		}

		if !foundPluginRegistered {
			t.Error("expected plugin:registered event")
		}
		if !foundEntityTypeRegistered {
			t.Error("expected entity_type:registered event")
		}
	})
}

func TestRegistryStats(t *testing.T) {
	t.Run("returns registry statistics", func(t *testing.T) {
		registry := NewRegistry()
		plugin := newMockPlugin("test-plugin", []string{"User", "Account"}, []string{"event1", "event2"})

		registry.Register(context.Background(), plugin)

		stats := registry.Stats()

		if stats["plugin_count"] != 1 {
			t.Errorf("expected plugin_count 1, got %v", stats["plugin_count"])
		}
		if stats["entity_type_count"] != 2 {
			t.Errorf("expected entity_type_count 2, got %v", stats["entity_type_count"])
		}
		if stats["event_type_count"] != 2 {
			t.Errorf("expected event_type_count 2, got %v", stats["event_type_count"])
		}
	})
}

func TestRegistryClear(t *testing.T) {
	t.Run("clears all plugins", func(t *testing.T) {
		registry := NewRegistry()
		plugin := newMockPlugin("test-plugin", []string{"User"}, []string{"event1"})

		registry.Register(context.Background(), plugin)
		registry.Clear()

		if registry.IsRegistered("test-plugin") {
			t.Error("expected plugin to be cleared")
		}
		if registry.GetPluginForEntityType("User") != nil {
			t.Error("expected entity type mapping to be cleared")
		}
		if registry.GetPluginForEventType("event1") != nil {
			t.Error("expected event type mapping to be cleared")
		}
	})
}

func TestGlobalRegistry(t *testing.T) {
	// Reset global registry before each test
	ResetGlobalRegistry()

	t.Run("returns singleton instance", func(t *testing.T) {
		registry1 := GetGlobalRegistry()
		registry2 := GetGlobalRegistry()

		if registry1 != registry2 {
			t.Error("expected same registry instance")
		}
	})

	t.Run("reset creates new instance", func(t *testing.T) {
		registry1 := GetGlobalRegistry()
		plugin := newMockPlugin("test-plugin", []string{}, []string{})
		registry1.Register(context.Background(), plugin)

		ResetGlobalRegistry()
		registry2 := GetGlobalRegistry()

		if registry2.IsRegistered("test-plugin") {
			t.Error("expected reset registry to not have plugin")
		}
	})
}

func TestRegistryConcurrency(t *testing.T) {
	t.Run("handles concurrent registration safely", func(t *testing.T) {
		registry := NewRegistry()
		var wg sync.WaitGroup

		// Register multiple plugins concurrently
		for i := 0; i < 100; i++ {
			wg.Add(1)
			go func(idx int) {
				defer wg.Done()
				plugin := newMockPlugin(
					"plugin-"+string(rune('a'+idx%26))+string(rune('0'+idx/26)),
					[]string{"Type" + string(rune('a'+idx%26)) + string(rune('0'+idx/26))},
					[]string{},
				)
				registry.Register(context.Background(), plugin)
			}(i)
		}

		wg.Wait()

		// Should have registered some plugins (some may fail due to conflicts)
		plugins := registry.GetAllPlugins()
		if len(plugins) == 0 {
			t.Error("expected some plugins to be registered")
		}
	})

	t.Run("handles concurrent reads safely", func(t *testing.T) {
		registry := NewRegistry()
		plugin := newMockPlugin("test-plugin", []string{"User"}, []string{"event1"})
		registry.Register(context.Background(), plugin)

		var wg sync.WaitGroup

		// Concurrent reads
		for i := 0; i < 100; i++ {
			wg.Add(4)
			go func() {
				defer wg.Done()
				registry.GetPlugin("test-plugin")
			}()
			go func() {
				defer wg.Done()
				registry.GetPluginForEntityType("User")
			}()
			go func() {
				defer wg.Done()
				registry.GetPluginForEventType("event1")
			}()
			go func() {
				defer wg.Done()
				registry.GetAllPlugins()
			}()
		}

		wg.Wait()
	})
}
