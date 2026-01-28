package pact

import (
	"context"
	"encoding/json"
	"fmt"
	"sort"
	"strings"
	"time"

	"github.com/google/uuid"
)

// PluginHooks defines plugin lifecycle hooks.
type PluginHooks struct {
	// OnLoad is called when plugin is loaded.
	OnLoad func(ctx context.Context) error

	// OnUnload is called when plugin is unloaded.
	OnUnload func(ctx context.Context) error

	// BeforeEventProcess is called before an event is processed.
	BeforeEventProcess func(ctx context.Context, event *PactEvent) (*PactEvent, error)

	// AfterDecision is called after a decision is made.
	AfterDecision func(ctx context.Context, event *PactEvent, decision *PactDecision) error

	// OnRulesUpdated is called when rules are updated.
	OnRulesUpdated func(ctx context.Context, rules []PactRule) error
}

// DomainPlugin is the interface that all PACT domain plugins must implement.
type DomainPlugin interface {
	// Manifest returns the plugin manifest.
	Manifest() PluginManifest

	// Hooks returns optional lifecycle hooks.
	Hooks() *PluginHooks

	// GetEntityTypes returns entity types defined by this plugin.
	GetEntityTypes() []PactEntityType

	// GetDefaultRules returns default rules for this domain.
	GetDefaultRules() []PactRule

	// MapToCanonicalEvent converts a domain event to canonical format.
	MapToCanonicalEvent(event DomainEvent) (*PactEvent, error)

	// ValidateDomainData validates domain-specific data.
	ValidateDomainData(entityType string, data map[string]interface{}) ValidationResult

	// GetEvaluationContext returns context for rule evaluation.
	GetEvaluationContext(event *PactEvent, entity *PactEntity) map[string]interface{}

	// SerializeForChain serializes data for blockchain.
	SerializeForChain(data interface{}) ([]byte, error)

	// DeserializeFromChain deserializes data from blockchain.
	DeserializeFromChain(data []byte) (interface{}, error)
}

// BasePlugin provides a base implementation of DomainPlugin.
type BasePlugin struct {
	manifest     PluginManifest
	entityTypes  []PactEntityType
	defaultRules []PactRule
	hooks        *PluginHooks
}

// NewBasePlugin creates a new BasePlugin.
func NewBasePlugin(
	manifest PluginManifest,
	entityTypes []PactEntityType,
	defaultRules []PactRule,
) *BasePlugin {
	return &BasePlugin{
		manifest:     manifest,
		entityTypes:  entityTypes,
		defaultRules: defaultRules,
	}
}

// SetHooks sets the plugin hooks.
func (p *BasePlugin) SetHooks(hooks *PluginHooks) {
	p.hooks = hooks
}

// Manifest returns the plugin manifest.
func (p *BasePlugin) Manifest() PluginManifest {
	return p.manifest
}

// Hooks returns the plugin hooks.
func (p *BasePlugin) Hooks() *PluginHooks {
	return p.hooks
}

// GetEntityTypes returns entity types defined by this plugin.
func (p *BasePlugin) GetEntityTypes() []PactEntityType {
	return p.entityTypes
}

// GetDefaultRules returns default rules for this domain.
func (p *BasePlugin) GetDefaultRules() []PactRule {
	return p.defaultRules
}

// MapToCanonicalEvent converts a domain event to canonical format.
func (p *BasePlugin) MapToCanonicalEvent(event DomainEvent) (*PactEvent, error) {
	now := time.Now().UTC()
	occurredAt := now
	if event.OccurredAt != nil {
		occurredAt = *event.OccurredAt
	}

	return &PactEvent{
		ID:           fmt.Sprintf("evt_%s", uuid.New().String()[:12]),
		Source:       p.manifest.ID,
		Type:         event.Type,
		EntityID:     event.EntityID,
		EntityType:   p.inferEntityType(event),
		Jurisdiction: event.Jurisdiction,
		Payload:      event.Payload,
		OccurredAt:   occurredAt,
		ReceivedAt:   now,
		Metadata:     event.Metadata,
	}, nil
}

// inferEntityType infers entity type from event type.
func (p *BasePlugin) inferEntityType(event DomainEvent) string {
	parts := strings.Split(event.Type, "_")
	if len(parts) >= 2 {
		entityName := parts[0]
		return strings.Title(strings.ToLower(entityName))
	}
	return ""
}

// ValidateDomainData validates domain-specific data.
func (p *BasePlugin) ValidateDomainData(entityType string, data map[string]interface{}) ValidationResult {
	// Find the entity type
	var entityTypeDef *PactEntityType
	for _, et := range p.entityTypes {
		if et.Name == entityType {
			entityTypeDef = &et
			break
		}
	}

	if entityTypeDef == nil {
		return ValidationResult{
			Valid: false,
			Errors: []ValidationError{{
				Field:   "entity_type",
				Message: fmt.Sprintf("Unknown entity type: %s", entityType),
				Code:    "UNKNOWN_ENTITY_TYPE",
			}},
		}
	}

	// Basic required field validation
	var errors []ValidationError
	for _, field := range entityTypeDef.RequiredFields {
		if _, ok := data[field]; !ok {
			errors = append(errors, ValidationError{
				Field:   field,
				Message: fmt.Sprintf("Required field '%s' is missing", field),
				Code:    "REQUIRED_FIELD_MISSING",
			})
		}
	}

	return ValidationResult{
		Valid:  len(errors) == 0,
		Errors: errors,
	}
}

// GetEvaluationContext returns context for rule evaluation.
func (p *BasePlugin) GetEvaluationContext(event *PactEvent, entity *PactEntity) map[string]interface{} {
	return make(map[string]interface{})
}

// SerializeForChain serializes data for blockchain.
func (p *BasePlugin) SerializeForChain(data interface{}) ([]byte, error) {
	return canonicalJSON(data)
}

// DeserializeFromChain deserializes data from blockchain.
func (p *BasePlugin) DeserializeFromChain(data []byte) (interface{}, error) {
	var result interface{}
	err := json.Unmarshal(data, &result)
	return result, err
}

// CreateRuleID generates a new rule ID.
func (p *BasePlugin) CreateRuleID() string {
	return fmt.Sprintf("rule_%s", uuid.New().String()[:12])
}

// CreateRule creates a rule with plugin defaults.
func (p *BasePlugin) CreateRule(
	name string,
	condition Condition,
	consequence Consequence,
	opts ...RuleOption,
) PactRule {
	rule := PactRule{
		ID:          p.CreateRuleID(),
		Name:        name,
		Description: name,
		Severity:    SeverityMedium,
		Condition:   condition,
		Consequence: consequence,
		Tags:        []string{p.manifest.ID},
		Metadata: map[string]interface{}{
			"plugin_id":      p.manifest.ID,
			"plugin_version": p.manifest.Version,
		},
	}

	for _, opt := range opts {
		opt(&rule)
	}

	return rule
}

// RuleOption is a function that modifies a rule.
type RuleOption func(*PactRule)

// WithDescription sets the rule description.
func WithDescription(desc string) RuleOption {
	return func(r *PactRule) {
		r.Description = desc
	}
}

// WithJurisdiction sets the rule jurisdiction.
func WithJurisdiction(jurisdiction string) RuleOption {
	return func(r *PactRule) {
		r.Jurisdiction = jurisdiction
	}
}

// WithSeverity sets the rule severity.
func WithSeverity(severity Severity) RuleOption {
	return func(r *PactRule) {
		r.Severity = severity
	}
}

// WithTags adds tags to the rule.
func WithTags(tags ...string) RuleOption {
	return func(r *PactRule) {
		r.Tags = append(r.Tags, tags...)
	}
}

// WithMetadata adds metadata to the rule.
func WithMetadata(key string, value interface{}) RuleOption {
	return func(r *PactRule) {
		if r.Metadata == nil {
			r.Metadata = make(map[string]interface{})
		}
		r.Metadata[key] = value
	}
}

// canonicalJSON produces canonical JSON with sorted keys.
func canonicalJSON(data interface{}) ([]byte, error) {
	// First marshal to JSON
	b, err := json.Marshal(data)
	if err != nil {
		return nil, err
	}

	// Unmarshal to get consistent representation
	var v interface{}
	if err := json.Unmarshal(b, &v); err != nil {
		return nil, err
	}

	// Re-marshal with sorted keys
	return marshalSorted(v)
}

func marshalSorted(v interface{}) ([]byte, error) {
	switch val := v.(type) {
	case map[string]interface{}:
		// Get sorted keys
		keys := make([]string, 0, len(val))
		for k := range val {
			keys = append(keys, k)
		}
		sort.Strings(keys)

		// Build sorted JSON manually
		result := "{"
		for i, k := range keys {
			if i > 0 {
				result += ","
			}
			keyJSON, _ := json.Marshal(k)
			valJSON, err := marshalSorted(val[k])
			if err != nil {
				return nil, err
			}
			result += string(keyJSON) + ":" + string(valJSON)
		}
		result += "}"
		return []byte(result), nil

	case []interface{}:
		result := "["
		for i, item := range val {
			if i > 0 {
				result += ","
			}
			itemJSON, err := marshalSorted(item)
			if err != nil {
				return nil, err
			}
			result += string(itemJSON)
		}
		result += "]"
		return []byte(result), nil

	default:
		return json.Marshal(v)
	}
}
