package pact

import (
	"fmt"
	"time"

	"github.com/google/uuid"
)

// ConditionBuilder provides a fluent API for building conditions.
type ConditionBuilder struct {
	field     string
	condition Condition
}

// NewCondition creates a new ConditionBuilder.
func NewCondition() *ConditionBuilder {
	return &ConditionBuilder{}
}

// Field sets the field path for comparison.
func (b *ConditionBuilder) Field(path string) *ConditionBuilder {
	b.field = path
	return b
}

// Eq creates an equality comparison.
func (b *ConditionBuilder) Eq(value any) *ConditionBuilder {
	b.condition = CompareCondition{Field: b.field, Operator: "eq", Value: value}
	return b
}

// Ne creates a not-equal comparison.
func (b *ConditionBuilder) Ne(value any) *ConditionBuilder {
	b.condition = CompareCondition{Field: b.field, Operator: "ne", Value: value}
	return b
}

// Gt creates a greater-than comparison.
func (b *ConditionBuilder) Gt(value any) *ConditionBuilder {
	b.condition = CompareCondition{Field: b.field, Operator: "gt", Value: value}
	return b
}

// Gte creates a greater-than-or-equal comparison.
func (b *ConditionBuilder) Gte(value any) *ConditionBuilder {
	b.condition = CompareCondition{Field: b.field, Operator: "gte", Value: value}
	return b
}

// Lt creates a less-than comparison.
func (b *ConditionBuilder) Lt(value any) *ConditionBuilder {
	b.condition = CompareCondition{Field: b.field, Operator: "lt", Value: value}
	return b
}

// Lte creates a less-than-or-equal comparison.
func (b *ConditionBuilder) Lte(value any) *ConditionBuilder {
	b.condition = CompareCondition{Field: b.field, Operator: "lte", Value: value}
	return b
}

// Contains creates a string contains comparison.
func (b *ConditionBuilder) Contains(value string) *ConditionBuilder {
	b.condition = CompareCondition{Field: b.field, Operator: "contains", Value: value}
	return b
}

// StartsWith creates a string starts-with comparison.
func (b *ConditionBuilder) StartsWith(value string) *ConditionBuilder {
	b.condition = CompareCondition{Field: b.field, Operator: "startsWith", Value: value}
	return b
}

// EndsWith creates a string ends-with comparison.
func (b *ConditionBuilder) EndsWith(value string) *ConditionBuilder {
	b.condition = CompareCondition{Field: b.field, Operator: "endsWith", Value: value}
	return b
}

// Matches creates a regex match comparison.
func (b *ConditionBuilder) Matches(pattern string) *ConditionBuilder {
	b.condition = CompareCondition{Field: b.field, Operator: "regex", Value: pattern}
	return b
}

// Exists creates a field existence condition.
func (b *ConditionBuilder) Exists() *ConditionBuilder {
	b.condition = ExistsCondition{Field: b.field}
	return b
}

// In creates a value-in-set condition.
func (b *ConditionBuilder) In(values ...any) *ConditionBuilder {
	b.condition = InCondition{Field: b.field, Values: values}
	return b
}

// Between creates a range condition.
func (b *ConditionBuilder) Between(min, max float64) *ConditionBuilder {
	b.condition = RangeCondition{
		Field:        b.field,
		Min:          &min,
		Max:          &max,
		MinInclusive: true,
		MaxInclusive: true,
	}
	return b
}

// Expression creates an expression condition.
func (b *ConditionBuilder) Expression(expr string, language string) *ConditionBuilder {
	if language == "" {
		language = "cel"
	}
	b.condition = ExpressionCondition{Expression: expr, Language: language}
	return b
}

// Build returns the built condition.
func (b *ConditionBuilder) Build() Condition {
	return b.condition
}

// And creates an AND condition from multiple conditions.
func And(conditions ...Condition) Condition {
	return AndCondition{Conditions: conditions}
}

// Or creates an OR condition from multiple conditions.
func Or(conditions ...Condition) Condition {
	return OrCondition{Conditions: conditions}
}

// Not creates a NOT condition.
func Not(condition Condition) Condition {
	return NotCondition{Condition: condition}
}

// RuleBuilder provides a fluent API for building rules.
type RuleBuilder struct {
	id            string
	name          string
	description   string
	jurisdiction  string
	severity      Severity
	condition     Condition
	consequence   *Consequence
	tags          []string
	effectiveFrom *time.Time
	effectiveTo   *time.Time
	metadata      map[string]any
}

// NewRule creates a new RuleBuilder.
func NewRule() *RuleBuilder {
	return &RuleBuilder{
		severity: SeverityMedium,
		tags:     []string{},
		metadata: make(map[string]any),
	}
}

// ID sets the rule ID.
func (b *RuleBuilder) ID(id string) *RuleBuilder {
	b.id = id
	return b
}

// Name sets the rule name.
func (b *RuleBuilder) Name(name string) *RuleBuilder {
	b.name = name
	return b
}

// Description sets the rule description.
func (b *RuleBuilder) Description(desc string) *RuleBuilder {
	b.description = desc
	return b
}

// Jurisdiction sets the rule jurisdiction.
func (b *RuleBuilder) Jurisdiction(jurisdiction string) *RuleBuilder {
	b.jurisdiction = jurisdiction
	return b
}

// Severity sets the rule severity.
func (b *RuleBuilder) Severity(severity Severity) *RuleBuilder {
	b.severity = severity
	return b
}

// When sets the rule condition.
func (b *RuleBuilder) When(condition Condition) *RuleBuilder {
	b.condition = condition
	return b
}

// WhenField sets a simple field comparison condition.
func (b *RuleBuilder) WhenField(field, operator string, value any) *RuleBuilder {
	b.condition = CompareCondition{Field: field, Operator: operator, Value: value}
	return b
}

// ThenDeny sets a DENY consequence.
func (b *RuleBuilder) ThenDeny(code, message string) *RuleBuilder {
	b.consequence = &Consequence{Decision: DecisionDeny, Code: code, Message: message}
	return b
}

// ThenFlag sets a FLAG consequence.
func (b *RuleBuilder) ThenFlag(code, message string) *RuleBuilder {
	b.consequence = &Consequence{Decision: DecisionFlag, Code: code, Message: message}
	return b
}

// ThenAllow sets an ALLOW consequence.
func (b *RuleBuilder) ThenAllow(code, message string) *RuleBuilder {
	b.consequence = &Consequence{Decision: DecisionAllow, Code: code, Message: message}
	return b
}

// Tags adds tags to the rule.
func (b *RuleBuilder) Tags(tags ...string) *RuleBuilder {
	b.tags = append(b.tags, tags...)
	return b
}

// EffectiveFrom sets the effective start date.
func (b *RuleBuilder) EffectiveFrom(t time.Time) *RuleBuilder {
	b.effectiveFrom = &t
	return b
}

// EffectiveTo sets the effective end date.
func (b *RuleBuilder) EffectiveTo(t time.Time) *RuleBuilder {
	b.effectiveTo = &t
	return b
}

// Metadata adds metadata to the rule.
func (b *RuleBuilder) Metadata(key string, value any) *RuleBuilder {
	b.metadata[key] = value
	return b
}

// Build builds and returns the rule.
func (b *RuleBuilder) Build() (*PactRule, error) {
	if b.name == "" {
		return nil, fmt.Errorf("rule name is required")
	}
	if b.condition == nil {
		return nil, fmt.Errorf("rule condition is required")
	}
	if b.consequence == nil {
		return nil, fmt.Errorf("rule consequence is required")
	}

	id := b.id
	if id == "" {
		id = fmt.Sprintf("rule_%s", uuid.New().String()[:12])
	}

	desc := b.description
	if desc == "" {
		desc = b.name
	}

	return &PactRule{
		ID:            id,
		Name:          b.name,
		Description:   desc,
		Jurisdiction:  b.jurisdiction,
		Severity:      b.severity,
		Condition:     b.condition,
		Consequence:   *b.consequence,
		Tags:          b.tags,
		EffectiveFrom: b.effectiveFrom,
		EffectiveTo:   b.effectiveTo,
		Metadata:      b.metadata,
	}, nil
}

// MustBuild builds and returns the rule, panicking on error.
func (b *RuleBuilder) MustBuild() PactRule {
	rule, err := b.Build()
	if err != nil {
		panic(err)
	}
	return *rule
}
