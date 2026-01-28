package pact

import (
	"context"
	"encoding/json"
	"testing"
	"time"
)

func TestNewBasePlugin(t *testing.T) {
	manifest := PluginManifest{
		ID:          "test-plugin",
		Name:        "Test Plugin",
		Version:     "1.0.0",
		Author:      "Test Author",
		Description: "A test plugin",
		EntityTypes: []string{"User"},
		EventTypes:  []string{"user_created"},
	}

	entityTypes := []PactEntityType{
		{Name: "User", Description: "A user entity", RequiredFields: []string{"name", "email"}},
	}

	defaultRules := []PactRule{
		{ID: "rule_1", Name: "Default Rule"},
	}

	plugin := NewBasePlugin(manifest, entityTypes, defaultRules)

	if plugin == nil {
		t.Fatal("expected non-nil plugin")
	}
	if plugin.manifest.ID != "test-plugin" {
		t.Errorf("expected ID %q, got %q", "test-plugin", plugin.manifest.ID)
	}
	if len(plugin.entityTypes) != 1 {
		t.Errorf("expected 1 entity type, got %d", len(plugin.entityTypes))
	}
	if len(plugin.defaultRules) != 1 {
		t.Errorf("expected 1 default rule, got %d", len(plugin.defaultRules))
	}
}

func TestBasePluginManifest(t *testing.T) {
	manifest := PluginManifest{
		ID:          "test-plugin",
		Name:        "Test Plugin",
		Version:     "1.0.0",
		EntityTypes: []string{"User"},
	}

	plugin := NewBasePlugin(manifest, nil, nil)
	retrieved := plugin.Manifest()

	if retrieved.ID != manifest.ID {
		t.Errorf("expected ID %q, got %q", manifest.ID, retrieved.ID)
	}
	if retrieved.Name != manifest.Name {
		t.Errorf("expected Name %q, got %q", manifest.Name, retrieved.Name)
	}
}

func TestBasePluginSetHooks(t *testing.T) {
	plugin := NewBasePlugin(PluginManifest{ID: "test"}, nil, nil)

	if plugin.Hooks() != nil {
		t.Error("expected nil hooks initially")
	}

	hooks := &PluginHooks{
		OnLoad: func(ctx context.Context) error {
			return nil
		},
	}
	plugin.SetHooks(hooks)

	if plugin.Hooks() == nil {
		t.Error("expected non-nil hooks after SetHooks")
	}
	if plugin.Hooks().OnLoad == nil {
		t.Error("expected OnLoad to be set")
	}
}

func TestBasePluginGetEntityTypes(t *testing.T) {
	entityTypes := []PactEntityType{
		{Name: "User", Description: "User entity"},
		{Name: "Account", Description: "Account entity"},
	}

	plugin := NewBasePlugin(PluginManifest{ID: "test"}, entityTypes, nil)
	retrieved := plugin.GetEntityTypes()

	if len(retrieved) != 2 {
		t.Errorf("expected 2 entity types, got %d", len(retrieved))
	}
}

func TestBasePluginGetDefaultRules(t *testing.T) {
	rules := []PactRule{
		{ID: "rule_1", Name: "Rule 1"},
		{ID: "rule_2", Name: "Rule 2"},
	}

	plugin := NewBasePlugin(PluginManifest{ID: "test"}, nil, rules)
	retrieved := plugin.GetDefaultRules()

	if len(retrieved) != 2 {
		t.Errorf("expected 2 rules, got %d", len(retrieved))
	}
}

func TestBasePluginMapToCanonicalEvent(t *testing.T) {
	plugin := NewBasePlugin(PluginManifest{ID: "test-plugin"}, nil, nil)

	t.Run("maps basic event", func(t *testing.T) {
		domainEvent := DomainEvent{
			Type:         "user_created",
			EntityID:     "user_123",
			Jurisdiction: "US",
			Payload: map[string]interface{}{
				"name":  "John",
				"email": "john@example.com",
			},
			Metadata: map[string]interface{}{
				"source": "api",
			},
		}

		pactEvent, err := plugin.MapToCanonicalEvent(domainEvent)
		if err != nil {
			t.Fatalf("MapToCanonicalEvent failed: %v", err)
		}

		if pactEvent.Source != "test-plugin" {
			t.Errorf("expected source %q, got %q", "test-plugin", pactEvent.Source)
		}
		if pactEvent.Type != "user_created" {
			t.Errorf("expected type %q, got %q", "user_created", pactEvent.Type)
		}
		if pactEvent.EntityID != "user_123" {
			t.Errorf("expected entityID %q, got %q", "user_123", pactEvent.EntityID)
		}
		if pactEvent.Jurisdiction != "US" {
			t.Errorf("expected jurisdiction %q, got %q", "US", pactEvent.Jurisdiction)
		}
		if pactEvent.ID == "" {
			t.Error("expected generated ID")
		}
		if pactEvent.OccurredAt.IsZero() {
			t.Error("expected non-zero OccurredAt")
		}
		if pactEvent.ReceivedAt.IsZero() {
			t.Error("expected non-zero ReceivedAt")
		}
	})

	t.Run("uses provided occurred_at time", func(t *testing.T) {
		occurredAt := time.Date(2024, 1, 15, 10, 30, 0, 0, time.UTC)
		domainEvent := DomainEvent{
			Type:       "test_event",
			OccurredAt: &occurredAt,
		}

		pactEvent, err := plugin.MapToCanonicalEvent(domainEvent)
		if err != nil {
			t.Fatalf("MapToCanonicalEvent failed: %v", err)
		}

		if !pactEvent.OccurredAt.Equal(occurredAt) {
			t.Errorf("expected OccurredAt %v, got %v", occurredAt, pactEvent.OccurredAt)
		}
	})

	t.Run("infers entity type from event type", func(t *testing.T) {
		domainEvent := DomainEvent{
			Type: "payment_initiated",
		}

		pactEvent, err := plugin.MapToCanonicalEvent(domainEvent)
		if err != nil {
			t.Fatalf("MapToCanonicalEvent failed: %v", err)
		}

		if pactEvent.EntityType != "Payment" {
			t.Errorf("expected entity type %q, got %q", "Payment", pactEvent.EntityType)
		}
	})
}

func TestBasePluginValidateDomainData(t *testing.T) {
	entityTypes := []PactEntityType{
		{
			Name:           "User",
			Description:    "User entity",
			RequiredFields: []string{"name", "email"},
		},
	}

	plugin := NewBasePlugin(PluginManifest{ID: "test"}, entityTypes, nil)

	t.Run("validates successfully with all required fields", func(t *testing.T) {
		data := map[string]interface{}{
			"name":  "John",
			"email": "john@example.com",
		}

		result := plugin.ValidateDomainData("User", data)

		if !result.Valid {
			t.Error("expected validation to pass")
		}
		if len(result.Errors) != 0 {
			t.Errorf("expected no errors, got %d", len(result.Errors))
		}
	})

	t.Run("fails validation with missing required fields", func(t *testing.T) {
		data := map[string]interface{}{
			"name": "John",
		}

		result := plugin.ValidateDomainData("User", data)

		if result.Valid {
			t.Error("expected validation to fail")
		}
		if len(result.Errors) != 1 {
			t.Errorf("expected 1 error, got %d", len(result.Errors))
		}
		if result.Errors[0].Field != "email" {
			t.Errorf("expected error on field %q, got %q", "email", result.Errors[0].Field)
		}
		if result.Errors[0].Code != "REQUIRED_FIELD_MISSING" {
			t.Errorf("expected code %q, got %q", "REQUIRED_FIELD_MISSING", result.Errors[0].Code)
		}
	})

	t.Run("fails for unknown entity type", func(t *testing.T) {
		data := map[string]interface{}{
			"name": "John",
		}

		result := plugin.ValidateDomainData("Unknown", data)

		if result.Valid {
			t.Error("expected validation to fail for unknown entity type")
		}
		if len(result.Errors) != 1 {
			t.Errorf("expected 1 error, got %d", len(result.Errors))
		}
		if result.Errors[0].Code != "UNKNOWN_ENTITY_TYPE" {
			t.Errorf("expected code %q, got %q", "UNKNOWN_ENTITY_TYPE", result.Errors[0].Code)
		}
	})
}

func TestBasePluginGetEvaluationContext(t *testing.T) {
	plugin := NewBasePlugin(PluginManifest{ID: "test"}, nil, nil)

	event := &PactEvent{ID: "evt_123"}
	entity := &PactEntity{ID: "ent_123"}

	ctx := plugin.GetEvaluationContext(event, entity)

	if ctx == nil {
		t.Error("expected non-nil context")
	}
	if len(ctx) != 0 {
		t.Errorf("expected empty context from base plugin, got %d entries", len(ctx))
	}
}

func TestBasePluginSerializeForChain(t *testing.T) {
	plugin := NewBasePlugin(PluginManifest{ID: "test"}, nil, nil)

	t.Run("serializes simple data", func(t *testing.T) {
		data := map[string]interface{}{
			"name": "John",
			"age":  30,
		}

		serialized, err := plugin.SerializeForChain(data)
		if err != nil {
			t.Fatalf("SerializeForChain failed: %v", err)
		}

		if len(serialized) == 0 {
			t.Error("expected non-empty serialized data")
		}
	})

	t.Run("produces canonical JSON with sorted keys", func(t *testing.T) {
		data := map[string]interface{}{
			"z_field": "last",
			"a_field": "first",
			"m_field": "middle",
		}

		serialized, err := plugin.SerializeForChain(data)
		if err != nil {
			t.Fatalf("SerializeForChain failed: %v", err)
		}

		expected := `{"a_field":"first","m_field":"middle","z_field":"last"}`
		if string(serialized) != expected {
			t.Errorf("expected %q, got %q", expected, string(serialized))
		}
	})

	t.Run("handles nested objects", func(t *testing.T) {
		data := map[string]interface{}{
			"b": map[string]interface{}{
				"d": 4,
				"c": 3,
			},
			"a": 1,
		}

		serialized, err := plugin.SerializeForChain(data)
		if err != nil {
			t.Fatalf("SerializeForChain failed: %v", err)
		}

		expected := `{"a":1,"b":{"c":3,"d":4}}`
		if string(serialized) != expected {
			t.Errorf("expected %q, got %q", expected, string(serialized))
		}
	})

	t.Run("handles arrays", func(t *testing.T) {
		data := []interface{}{
			map[string]interface{}{"b": 2, "a": 1},
			map[string]interface{}{"d": 4, "c": 3},
		}

		serialized, err := plugin.SerializeForChain(data)
		if err != nil {
			t.Fatalf("SerializeForChain failed: %v", err)
		}

		expected := `[{"a":1,"b":2},{"c":3,"d":4}]`
		if string(serialized) != expected {
			t.Errorf("expected %q, got %q", expected, string(serialized))
		}
	})
}

func TestBasePluginDeserializeFromChain(t *testing.T) {
	plugin := NewBasePlugin(PluginManifest{ID: "test"}, nil, nil)

	t.Run("deserializes JSON data", func(t *testing.T) {
		data := []byte(`{"name":"John","age":30}`)

		result, err := plugin.DeserializeFromChain(data)
		if err != nil {
			t.Fatalf("DeserializeFromChain failed: %v", err)
		}

		m, ok := result.(map[string]interface{})
		if !ok {
			t.Fatal("expected map result")
		}
		if m["name"] != "John" {
			t.Errorf("expected name %q, got %v", "John", m["name"])
		}
	})

	t.Run("handles invalid JSON", func(t *testing.T) {
		data := []byte(`invalid json`)

		_, err := plugin.DeserializeFromChain(data)
		if err == nil {
			t.Error("expected error for invalid JSON")
		}
	})
}

func TestBasePluginCreateRuleID(t *testing.T) {
	plugin := NewBasePlugin(PluginManifest{ID: "test"}, nil, nil)

	id1 := plugin.CreateRuleID()
	id2 := plugin.CreateRuleID()

	if id1 == "" {
		t.Error("expected non-empty ID")
	}
	if !hasPrefix(id1, "rule_") {
		t.Errorf("expected ID to start with %q, got %q", "rule_", id1)
	}
	if id1 == id2 {
		t.Error("expected unique IDs")
	}
}

func hasPrefix(s, prefix string) bool {
	return len(s) >= len(prefix) && s[:len(prefix)] == prefix
}

func TestBasePluginCreateRule(t *testing.T) {
	manifest := PluginManifest{
		ID:      "test-plugin",
		Version: "1.0.0",
	}
	plugin := NewBasePlugin(manifest, nil, nil)

	t.Run("creates rule with defaults", func(t *testing.T) {
		condition := CompareCondition{Field: "amount", Operator: "gt", Value: 1000}
		consequence := Consequence{Decision: DecisionDeny, Code: "HIGH_AMOUNT", Message: "Amount too high"}

		rule := plugin.CreateRule("Test Rule", condition, consequence)

		if rule.Name != "Test Rule" {
			t.Errorf("expected name %q, got %q", "Test Rule", rule.Name)
		}
		if rule.Description != "Test Rule" {
			t.Errorf("expected description %q, got %q", "Test Rule", rule.Description)
		}
		if rule.Severity != SeverityMedium {
			t.Errorf("expected severity %q, got %q", SeverityMedium, rule.Severity)
		}
		if rule.ID == "" {
			t.Error("expected generated ID")
		}
		if len(rule.Tags) == 0 || rule.Tags[0] != "test-plugin" {
			t.Error("expected plugin ID in tags")
		}
		if rule.Metadata["plugin_id"] != "test-plugin" {
			t.Errorf("expected plugin_id metadata")
		}
		if rule.Metadata["plugin_version"] != "1.0.0" {
			t.Errorf("expected plugin_version metadata")
		}
	})

	t.Run("creates rule with options", func(t *testing.T) {
		condition := CompareCondition{Field: "amount", Operator: "gt", Value: 1000}
		consequence := Consequence{Decision: DecisionDeny, Code: "HIGH_AMOUNT", Message: "Amount too high"}

		rule := plugin.CreateRule("Test Rule", condition, consequence,
			WithDescription("Custom description"),
			WithJurisdiction("US"),
			WithSeverity(SeverityCritical),
			WithTags("aml", "compliance"),
			WithMetadata("custom_key", "custom_value"),
		)

		if rule.Description != "Custom description" {
			t.Errorf("expected description %q, got %q", "Custom description", rule.Description)
		}
		if rule.Jurisdiction != "US" {
			t.Errorf("expected jurisdiction %q, got %q", "US", rule.Jurisdiction)
		}
		if rule.Severity != SeverityCritical {
			t.Errorf("expected severity %q, got %q", SeverityCritical, rule.Severity)
		}
		if len(rule.Tags) != 3 { // test-plugin + aml + compliance
			t.Errorf("expected 3 tags, got %d", len(rule.Tags))
		}
		if rule.Metadata["custom_key"] != "custom_value" {
			t.Errorf("expected custom_key metadata")
		}
	})
}

func TestPluginHooksLifecycle(t *testing.T) {
	manifest := PluginManifest{
		ID:          "test-plugin",
		Name:        "Test Plugin",
		EntityTypes: []string{"User"},
		EventTypes:  []string{"user_created"},
	}

	plugin := NewBasePlugin(manifest, nil, nil)

	loadCalled := false
	unloadCalled := false
	beforeEventCalled := false
	afterDecisionCalled := false
	rulesUpdatedCalled := false

	hooks := &PluginHooks{
		OnLoad: func(ctx context.Context) error {
			loadCalled = true
			return nil
		},
		OnUnload: func(ctx context.Context) error {
			unloadCalled = true
			return nil
		},
		BeforeEventProcess: func(ctx context.Context, event *PactEvent) (*PactEvent, error) {
			beforeEventCalled = true
			return event, nil
		},
		AfterDecision: func(ctx context.Context, event *PactEvent, decision *PactDecision) error {
			afterDecisionCalled = true
			return nil
		},
		OnRulesUpdated: func(ctx context.Context, rules []PactRule) error {
			rulesUpdatedCalled = true
			return nil
		},
	}

	plugin.SetHooks(hooks)

	// Test OnLoad
	hooks.OnLoad(context.Background())
	if !loadCalled {
		t.Error("expected OnLoad to be called")
	}

	// Test OnUnload
	hooks.OnUnload(context.Background())
	if !unloadCalled {
		t.Error("expected OnUnload to be called")
	}

	// Test BeforeEventProcess
	event := &PactEvent{ID: "evt_123"}
	hooks.BeforeEventProcess(context.Background(), event)
	if !beforeEventCalled {
		t.Error("expected BeforeEventProcess to be called")
	}

	// Test AfterDecision
	decision := &PactDecision{ID: "dec_123"}
	hooks.AfterDecision(context.Background(), event, decision)
	if !afterDecisionCalled {
		t.Error("expected AfterDecision to be called")
	}

	// Test OnRulesUpdated
	rules := []PactRule{{ID: "rule_1"}}
	hooks.OnRulesUpdated(context.Background(), rules)
	if !rulesUpdatedCalled {
		t.Error("expected OnRulesUpdated to be called")
	}
}

func TestRuleOptions(t *testing.T) {
	t.Run("WithDescription sets description", func(t *testing.T) {
		rule := &PactRule{}
		WithDescription("Test description")(rule)

		if rule.Description != "Test description" {
			t.Errorf("expected description %q, got %q", "Test description", rule.Description)
		}
	})

	t.Run("WithJurisdiction sets jurisdiction", func(t *testing.T) {
		rule := &PactRule{}
		WithJurisdiction("EU")(rule)

		if rule.Jurisdiction != "EU" {
			t.Errorf("expected jurisdiction %q, got %q", "EU", rule.Jurisdiction)
		}
	})

	t.Run("WithSeverity sets severity", func(t *testing.T) {
		rule := &PactRule{}
		WithSeverity(SeverityHigh)(rule)

		if rule.Severity != SeverityHigh {
			t.Errorf("expected severity %q, got %q", SeverityHigh, rule.Severity)
		}
	})

	t.Run("WithTags appends tags", func(t *testing.T) {
		rule := &PactRule{Tags: []string{"existing"}}
		WithTags("new1", "new2")(rule)

		if len(rule.Tags) != 3 {
			t.Errorf("expected 3 tags, got %d", len(rule.Tags))
		}
	})

	t.Run("WithMetadata adds metadata", func(t *testing.T) {
		rule := &PactRule{}
		WithMetadata("key1", "value1")(rule)
		WithMetadata("key2", "value2")(rule)

		if rule.Metadata["key1"] != "value1" {
			t.Errorf("expected key1 %q, got %v", "value1", rule.Metadata["key1"])
		}
		if rule.Metadata["key2"] != "value2" {
			t.Errorf("expected key2 %q, got %v", "value2", rule.Metadata["key2"])
		}
	})

	t.Run("WithMetadata initializes nil map", func(t *testing.T) {
		rule := &PactRule{}
		WithMetadata("key", "value")(rule)

		if rule.Metadata == nil {
			t.Error("expected metadata to be initialized")
		}
	})
}

func TestBasePluginInterfaceCompliance(t *testing.T) {
	// Verify BasePlugin implements DomainPlugin interface
	var _ DomainPlugin = (*BasePlugin)(nil)

	plugin := NewBasePlugin(PluginManifest{ID: "test"}, nil, nil)

	// Test that all interface methods work
	_ = plugin.Manifest()
	_ = plugin.Hooks()
	_ = plugin.GetEntityTypes()
	_ = plugin.GetDefaultRules()
	_, _ = plugin.MapToCanonicalEvent(DomainEvent{Type: "test"})
	_ = plugin.ValidateDomainData("User", map[string]interface{}{})
	_ = plugin.GetEvaluationContext(nil, nil)
	_, _ = plugin.SerializeForChain(map[string]interface{}{})
	_, _ = plugin.DeserializeFromChain([]byte("{}"))
}

func TestCanonicalJSONDeterminism(t *testing.T) {
	plugin := NewBasePlugin(PluginManifest{ID: "test"}, nil, nil)

	data := map[string]interface{}{
		"z": 3,
		"a": 1,
		"m": 2,
		"nested": map[string]interface{}{
			"y": "last",
			"b": "first",
		},
	}

	// Serialize multiple times and ensure consistent output
	var results []string
	for i := 0; i < 10; i++ {
		serialized, err := plugin.SerializeForChain(data)
		if err != nil {
			t.Fatalf("SerializeForChain failed: %v", err)
		}
		results = append(results, string(serialized))
	}

	first := results[0]
	for i, result := range results {
		if result != first {
			t.Errorf("serialization %d differs: %q vs %q", i, result, first)
		}
	}

	// Verify it's valid JSON
	var parsed map[string]interface{}
	if err := json.Unmarshal([]byte(first), &parsed); err != nil {
		t.Errorf("result is not valid JSON: %v", err)
	}
}
