// Package pact provides the PACT Plugin SDK for Go.
//
// This SDK enables building PACT Protocol domain plugins in Go.
//
// Features:
//   - Plugin interfaces and base implementations
//   - Type-safe rule and condition builders
//   - Event mapping utilities
//   - Validation helpers
//   - gRPC service integration
//
// Example:
//
//	type MyPlugin struct {
//	    arka.BasePlugin
//	}
//
//	func (p *MyPlugin) Manifest() arka.PluginManifest {
//	    return arka.PluginManifest{
//	        ID:          "my-plugin",
//	        Name:        "My Plugin",
//	        Version:     "1.0.0",
//	        Author:      "My Company",
//	        Description: "My custom PACT plugin",
//	    }
//	}
package pact

import (
	"encoding/json"
	"time"
)

// Severity represents rule severity levels.
type Severity string

const (
	SeverityLow      Severity = "LOW"
	SeverityMedium   Severity = "MEDIUM"
	SeverityHigh     Severity = "HIGH"
	SeverityCritical Severity = "CRITICAL"
)

// Decision represents rule consequence decisions.
type Decision string

const (
	DecisionAllow Decision = "ALLOW"
	DecisionDeny  Decision = "DENY"
	DecisionFlag  Decision = "FLAG"
)

// DecisionStatus represents overall decision status.
type DecisionStatus string

const (
	DecisionStatusAllow         DecisionStatus = "ALLOW"
	DecisionStatusAllowWithFlags DecisionStatus = "ALLOW_WITH_FLAGS"
	DecisionStatusDeny          DecisionStatus = "DENY"
)

// EvaluationResult represents the result of evaluating a rule.
type EvaluationResult string

const (
	EvaluationResultPass  EvaluationResult = "PASS"
	EvaluationResultFail  EvaluationResult = "FAIL"
	EvaluationResultSkip  EvaluationResult = "SKIP"
	EvaluationResultError EvaluationResult = "ERROR"
)

// PluginManifest describes plugin capabilities.
type PluginManifest struct {
	ID              string            `json:"id"`
	Name            string            `json:"name"`
	Version         string            `json:"version"`
	Author          string            `json:"author"`
	Description     string            `json:"description"`
	EntityTypes     []string          `json:"entity_types"`
	EventTypes      []string          `json:"event_types"`
	Dependencies    []string          `json:"dependencies,omitempty"`
	PactCoreVersion string            `json:"pact_core_version"`
	ConfigSchema    map[string]interface{}    `json:"config_schema,omitempty"`
}

// DomainEvent represents a domain-specific event before conversion.
type DomainEvent struct {
	Type         string         `json:"type"`
	Payload      map[string]interface{} `json:"payload"`
	EntityID     string         `json:"entity_id,omitempty"`
	Jurisdiction string         `json:"jurisdiction,omitempty"`
	OccurredAt   *time.Time     `json:"occurred_at,omitempty"`
	Metadata     map[string]interface{} `json:"metadata,omitempty"`
}

// PactEvent represents a canonical PACT event.
type PactEvent struct {
	ID           string         `json:"id"`
	Source       string         `json:"source"`
	Type         string         `json:"type"`
	EntityID     string         `json:"entity_id,omitempty"`
	EntityType   string         `json:"entity_type,omitempty"`
	Jurisdiction string         `json:"jurisdiction,omitempty"`
	Payload      map[string]interface{} `json:"payload"`
	OccurredAt   time.Time      `json:"occurred_at"`
	ReceivedAt   time.Time      `json:"received_at"`
	Metadata     map[string]interface{} `json:"metadata,omitempty"`
}

// PactEntity represents a PACT entity.
type PactEntity struct {
	ID           string         `json:"id"`
	Type         string         `json:"type"`
	Data         map[string]interface{} `json:"data"`
	CreatedAt    time.Time      `json:"created_at"`
	UpdatedAt    time.Time      `json:"updated_at"`
	Jurisdiction string         `json:"jurisdiction,omitempty"`
	Metadata     map[string]interface{} `json:"metadata,omitempty"`
}

// PactEntityType defines an entity type with schema.
type PactEntityType struct {
	Name           string         `json:"name"`
	Description    string         `json:"description"`
	Schema         map[string]interface{} `json:"schema"`
	RequiredFields []string       `json:"required_fields,omitempty"`
}

// Condition represents a rule condition.
type Condition interface {
	conditionMarker()
	json.Marshaler
}

// CompareCondition represents a comparison condition.
type CompareCondition struct {
	Type     string `json:"type"`
	Field    string `json:"field"`
	Operator string `json:"operator"`
	Value    interface{}    `json:"value"`
}

func (c CompareCondition) conditionMarker() {}

func (c CompareCondition) MarshalJSON() ([]byte, error) {
	type Alias CompareCondition
	return json.Marshal(&struct {
		Type string `json:"type"`
		*Alias
	}{
		Type:  "compare",
		Alias: (*Alias)(&c),
	})
}

// AndCondition represents a logical AND condition.
type AndCondition struct {
	Conditions []Condition `json:"conditions"`
}

func (c AndCondition) conditionMarker() {}

func (c AndCondition) MarshalJSON() ([]byte, error) {
	return json.Marshal(map[string]interface{}{
		"type":       "and",
		"conditions": c.Conditions,
	})
}

// OrCondition represents a logical OR condition.
type OrCondition struct {
	Conditions []Condition `json:"conditions"`
}

func (c OrCondition) conditionMarker() {}

func (c OrCondition) MarshalJSON() ([]byte, error) {
	return json.Marshal(map[string]interface{}{
		"type":       "or",
		"conditions": c.Conditions,
	})
}

// NotCondition represents a logical NOT condition.
type NotCondition struct {
	Condition Condition `json:"condition"`
}

func (c NotCondition) conditionMarker() {}

func (c NotCondition) MarshalJSON() ([]byte, error) {
	return json.Marshal(map[string]interface{}{
		"type":      "not",
		"condition": c.Condition,
	})
}

// ExistsCondition checks if a field exists.
type ExistsCondition struct {
	Field string `json:"field"`
}

func (c ExistsCondition) conditionMarker() {}

func (c ExistsCondition) MarshalJSON() ([]byte, error) {
	return json.Marshal(map[string]interface{}{
		"type":  "exists",
		"field": c.Field,
	})
}

// InCondition checks if a value is in a set.
type InCondition struct {
	Field  string `json:"field"`
	Values []interface{}  `json:"values"`
}

func (c InCondition) conditionMarker() {}

func (c InCondition) MarshalJSON() ([]byte, error) {
	return json.Marshal(map[string]interface{}{
		"type":   "in",
		"field":  c.Field,
		"values": c.Values,
	})
}

// RangeCondition checks if a value is in a range.
type RangeCondition struct {
	Field        string   `json:"field"`
	Min          *float64 `json:"min,omitempty"`
	Max          *float64 `json:"max,omitempty"`
	MinInclusive bool     `json:"min_inclusive"`
	MaxInclusive bool     `json:"max_inclusive"`
}

func (c RangeCondition) conditionMarker() {}

func (c RangeCondition) MarshalJSON() ([]byte, error) {
	return json.Marshal(map[string]interface{}{
		"type":          "range",
		"field":         c.Field,
		"min":           c.Min,
		"max":           c.Max,
		"min_inclusive": c.MinInclusive,
		"max_inclusive": c.MaxInclusive,
	})
}

// ExpressionCondition uses a custom expression.
type ExpressionCondition struct {
	Expression string `json:"expression"`
	Language   string `json:"language"`
}

func (c ExpressionCondition) conditionMarker() {}

func (c ExpressionCondition) MarshalJSON() ([]byte, error) {
	return json.Marshal(map[string]interface{}{
		"type":       "expression",
		"expression": c.Expression,
		"language":   c.Language,
	})
}

// Consequence represents a rule consequence.
type Consequence struct {
	Decision Decision       `json:"decision"`
	Code     string         `json:"code"`
	Message  string         `json:"message"`
	Metadata map[string]interface{} `json:"metadata,omitempty"`
}

// PactRule represents a PACT rule.
type PactRule struct {
	ID            string         `json:"id"`
	Name          string         `json:"name"`
	Description   string         `json:"description"`
	Jurisdiction  string         `json:"jurisdiction,omitempty"`
	Severity      Severity       `json:"severity"`
	Condition     Condition      `json:"condition"`
	Consequence   Consequence    `json:"consequence"`
	Tags          []string       `json:"tags,omitempty"`
	EffectiveFrom *time.Time     `json:"effective_from,omitempty"`
	EffectiveTo   *time.Time     `json:"effective_to,omitempty"`
	Metadata      map[string]interface{} `json:"metadata,omitempty"`
}

// RuleEvaluation represents the result of evaluating a rule.
type RuleEvaluation struct {
	RuleID     string           `json:"rule_id"`
	RuleName   string           `json:"rule_name"`
	Result     EvaluationResult `json:"result"`
	Code       string           `json:"code,omitempty"`
	Message    string           `json:"message,omitempty"`
	DurationMs int64            `json:"duration_ms"`
}

// PactDecision represents a decision from rule evaluation.
type PactDecision struct {
	ID              string           `json:"id"`
	EventID         string           `json:"event_id"`
	Status          DecisionStatus   `json:"status"`
	RuleEvaluations []RuleEvaluation `json:"rule_evaluations"`
	CreatedAt       time.Time        `json:"created_at"`
	Metadata        map[string]interface{}   `json:"metadata,omitempty"`
}

// ValidationError represents a validation error.
type ValidationError struct {
	Field   string `json:"field"`
	Message string `json:"message"`
	Code    string `json:"code"`
}

// ValidationWarning represents a validation warning.
type ValidationWarning struct {
	Field   string `json:"field"`
	Message string `json:"message"`
}

// ValidationResult represents the result of validation.
type ValidationResult struct {
	Valid    bool                `json:"valid"`
	Errors   []ValidationError   `json:"errors,omitempty"`
	Warnings []ValidationWarning `json:"warnings,omitempty"`
}
