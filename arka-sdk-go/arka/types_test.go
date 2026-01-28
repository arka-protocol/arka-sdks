package pact

import (
	"encoding/json"
	"testing"
	"time"
)

func TestSeverityConstants(t *testing.T) {
	tests := []struct {
		severity Severity
		expected string
	}{
		{SeverityLow, "LOW"},
		{SeverityMedium, "MEDIUM"},
		{SeverityHigh, "HIGH"},
		{SeverityCritical, "CRITICAL"},
	}

	for _, tt := range tests {
		if string(tt.severity) != tt.expected {
			t.Errorf("expected severity %q, got %q", tt.expected, string(tt.severity))
		}
	}
}

func TestDecisionConstants(t *testing.T) {
	tests := []struct {
		decision Decision
		expected string
	}{
		{DecisionAllow, "ALLOW"},
		{DecisionDeny, "DENY"},
		{DecisionFlag, "FLAG"},
	}

	for _, tt := range tests {
		if string(tt.decision) != tt.expected {
			t.Errorf("expected decision %q, got %q", tt.expected, string(tt.decision))
		}
	}
}

func TestDecisionStatusConstants(t *testing.T) {
	tests := []struct {
		status   DecisionStatus
		expected string
	}{
		{DecisionStatusAllow, "ALLOW"},
		{DecisionStatusAllowWithFlags, "ALLOW_WITH_FLAGS"},
		{DecisionStatusDeny, "DENY"},
	}

	for _, tt := range tests {
		if string(tt.status) != tt.expected {
			t.Errorf("expected status %q, got %q", tt.expected, string(tt.status))
		}
	}
}

func TestEvaluationResultConstants(t *testing.T) {
	tests := []struct {
		result   EvaluationResult
		expected string
	}{
		{EvaluationResultPass, "PASS"},
		{EvaluationResultFail, "FAIL"},
		{EvaluationResultSkip, "SKIP"},
		{EvaluationResultError, "ERROR"},
	}

	for _, tt := range tests {
		if string(tt.result) != tt.expected {
			t.Errorf("expected result %q, got %q", tt.expected, string(tt.result))
		}
	}
}

func TestPluginManifestSerialization(t *testing.T) {
	manifest := PluginManifest{
		ID:              "test-plugin",
		Name:            "Test Plugin",
		Version:         "1.0.0",
		Author:          "Test Author",
		Description:     "A test plugin",
		EntityTypes:     []string{"User", "Account"},
		EventTypes:      []string{"user_created", "account_opened"},
		Dependencies:    []string{"base-plugin"},
		PactCoreVersion: "1.0.0",
		ConfigSchema: map[string]interface{}{
			"type": "object",
		},
	}

	data, err := json.Marshal(manifest)
	if err != nil {
		t.Fatalf("failed to marshal manifest: %v", err)
	}

	var decoded PluginManifest
	if err := json.Unmarshal(data, &decoded); err != nil {
		t.Fatalf("failed to unmarshal manifest: %v", err)
	}

	if decoded.ID != manifest.ID {
		t.Errorf("expected ID %q, got %q", manifest.ID, decoded.ID)
	}
	if decoded.Name != manifest.Name {
		t.Errorf("expected Name %q, got %q", manifest.Name, decoded.Name)
	}
	if len(decoded.EntityTypes) != len(manifest.EntityTypes) {
		t.Errorf("expected %d entity types, got %d", len(manifest.EntityTypes), len(decoded.EntityTypes))
	}
}

func TestPactEventSerialization(t *testing.T) {
	now := time.Now().UTC().Truncate(time.Second)
	event := PactEvent{
		ID:           "evt_123",
		Source:       "test-plugin",
		Type:         "payment_initiated",
		EntityID:     "user_456",
		EntityType:   "User",
		Jurisdiction: "US",
		Payload: map[string]interface{}{
			"amount":   100.50,
			"currency": "USD",
		},
		OccurredAt: now,
		ReceivedAt: now,
		Metadata: map[string]interface{}{
			"trace_id": "abc123",
		},
	}

	data, err := json.Marshal(event)
	if err != nil {
		t.Fatalf("failed to marshal event: %v", err)
	}

	var decoded PactEvent
	if err := json.Unmarshal(data, &decoded); err != nil {
		t.Fatalf("failed to unmarshal event: %v", err)
	}

	if decoded.ID != event.ID {
		t.Errorf("expected ID %q, got %q", event.ID, decoded.ID)
	}
	if decoded.Type != event.Type {
		t.Errorf("expected Type %q, got %q", event.Type, decoded.Type)
	}
	if decoded.Payload["amount"] != event.Payload["amount"] {
		t.Errorf("expected amount %v, got %v", event.Payload["amount"], decoded.Payload["amount"])
	}
}

func TestPactEntitySerialization(t *testing.T) {
	now := time.Now().UTC().Truncate(time.Second)
	entity := PactEntity{
		ID:   "ent_123",
		Type: "User",
		Data: map[string]interface{}{
			"name":  "John Doe",
			"email": "john@example.com",
		},
		CreatedAt:    now,
		UpdatedAt:    now,
		Jurisdiction: "US",
		Metadata: map[string]interface{}{
			"verified": true,
		},
	}

	data, err := json.Marshal(entity)
	if err != nil {
		t.Fatalf("failed to marshal entity: %v", err)
	}

	var decoded PactEntity
	if err := json.Unmarshal(data, &decoded); err != nil {
		t.Fatalf("failed to unmarshal entity: %v", err)
	}

	if decoded.ID != entity.ID {
		t.Errorf("expected ID %q, got %q", entity.ID, decoded.ID)
	}
	if decoded.Type != entity.Type {
		t.Errorf("expected Type %q, got %q", entity.Type, decoded.Type)
	}
	if decoded.Data["name"] != entity.Data["name"] {
		t.Errorf("expected name %v, got %v", entity.Data["name"], decoded.Data["name"])
	}
}

func TestCompareConditionSerialization(t *testing.T) {
	condition := CompareCondition{
		Field:    "payload.amount",
		Operator: "gt",
		Value:    1000,
	}

	data, err := json.Marshal(condition)
	if err != nil {
		t.Fatalf("failed to marshal condition: %v", err)
	}

	var result map[string]interface{}
	if err := json.Unmarshal(data, &result); err != nil {
		t.Fatalf("failed to unmarshal condition: %v", err)
	}

	if result["type"] != "compare" {
		t.Errorf("expected type %q, got %v", "compare", result["type"])
	}
	if result["field"] != "payload.amount" {
		t.Errorf("expected field %q, got %v", "payload.amount", result["field"])
	}
	if result["operator"] != "gt" {
		t.Errorf("expected operator %q, got %v", "gt", result["operator"])
	}
}

func TestAndConditionSerialization(t *testing.T) {
	condition := AndCondition{
		Conditions: []Condition{
			CompareCondition{Field: "a", Operator: "eq", Value: 1},
			CompareCondition{Field: "b", Operator: "eq", Value: 2},
		},
	}

	data, err := json.Marshal(condition)
	if err != nil {
		t.Fatalf("failed to marshal condition: %v", err)
	}

	var result map[string]interface{}
	if err := json.Unmarshal(data, &result); err != nil {
		t.Fatalf("failed to unmarshal condition: %v", err)
	}

	if result["type"] != "and" {
		t.Errorf("expected type %q, got %v", "and", result["type"])
	}

	conditions, ok := result["conditions"].([]interface{})
	if !ok {
		t.Fatal("expected conditions to be an array")
	}
	if len(conditions) != 2 {
		t.Errorf("expected 2 conditions, got %d", len(conditions))
	}
}

func TestOrConditionSerialization(t *testing.T) {
	condition := OrCondition{
		Conditions: []Condition{
			CompareCondition{Field: "a", Operator: "eq", Value: 1},
			CompareCondition{Field: "b", Operator: "eq", Value: 2},
		},
	}

	data, err := json.Marshal(condition)
	if err != nil {
		t.Fatalf("failed to marshal condition: %v", err)
	}

	var result map[string]interface{}
	if err := json.Unmarshal(data, &result); err != nil {
		t.Fatalf("failed to unmarshal condition: %v", err)
	}

	if result["type"] != "or" {
		t.Errorf("expected type %q, got %v", "or", result["type"])
	}
}

func TestNotConditionSerialization(t *testing.T) {
	condition := NotCondition{
		Condition: CompareCondition{Field: "a", Operator: "eq", Value: 1},
	}

	data, err := json.Marshal(condition)
	if err != nil {
		t.Fatalf("failed to marshal condition: %v", err)
	}

	var result map[string]interface{}
	if err := json.Unmarshal(data, &result); err != nil {
		t.Fatalf("failed to unmarshal condition: %v", err)
	}

	if result["type"] != "not" {
		t.Errorf("expected type %q, got %v", "not", result["type"])
	}
}

func TestExistsConditionSerialization(t *testing.T) {
	condition := ExistsCondition{
		Field: "payload.optional_field",
	}

	data, err := json.Marshal(condition)
	if err != nil {
		t.Fatalf("failed to marshal condition: %v", err)
	}

	var result map[string]interface{}
	if err := json.Unmarshal(data, &result); err != nil {
		t.Fatalf("failed to unmarshal condition: %v", err)
	}

	if result["type"] != "exists" {
		t.Errorf("expected type %q, got %v", "exists", result["type"])
	}
	if result["field"] != "payload.optional_field" {
		t.Errorf("expected field %q, got %v", "payload.optional_field", result["field"])
	}
}

func TestInConditionSerialization(t *testing.T) {
	condition := InCondition{
		Field:  "status",
		Values: []interface{}{"active", "pending", "approved"},
	}

	data, err := json.Marshal(condition)
	if err != nil {
		t.Fatalf("failed to marshal condition: %v", err)
	}

	var result map[string]interface{}
	if err := json.Unmarshal(data, &result); err != nil {
		t.Fatalf("failed to unmarshal condition: %v", err)
	}

	if result["type"] != "in" {
		t.Errorf("expected type %q, got %v", "in", result["type"])
	}
	if result["field"] != "status" {
		t.Errorf("expected field %q, got %v", "status", result["field"])
	}
}

func TestRangeConditionSerialization(t *testing.T) {
	min := 100.0
	max := 1000.0
	condition := RangeCondition{
		Field:        "amount",
		Min:          &min,
		Max:          &max,
		MinInclusive: true,
		MaxInclusive: false,
	}

	data, err := json.Marshal(condition)
	if err != nil {
		t.Fatalf("failed to marshal condition: %v", err)
	}

	var result map[string]interface{}
	if err := json.Unmarshal(data, &result); err != nil {
		t.Fatalf("failed to unmarshal condition: %v", err)
	}

	if result["type"] != "range" {
		t.Errorf("expected type %q, got %v", "range", result["type"])
	}
	if result["min"] != 100.0 {
		t.Errorf("expected min %v, got %v", 100.0, result["min"])
	}
	if result["max"] != 1000.0 {
		t.Errorf("expected max %v, got %v", 1000.0, result["max"])
	}
}

func TestExpressionConditionSerialization(t *testing.T) {
	condition := ExpressionCondition{
		Expression: "event.amount > 1000 && event.risk_score > 0.8",
		Language:   "cel",
	}

	data, err := json.Marshal(condition)
	if err != nil {
		t.Fatalf("failed to marshal condition: %v", err)
	}

	var result map[string]interface{}
	if err := json.Unmarshal(data, &result); err != nil {
		t.Fatalf("failed to unmarshal condition: %v", err)
	}

	if result["type"] != "expression" {
		t.Errorf("expected type %q, got %v", "expression", result["type"])
	}
	if result["language"] != "cel" {
		t.Errorf("expected language %q, got %v", "cel", result["language"])
	}
}

func TestConsequenceSerialization(t *testing.T) {
	consequence := Consequence{
		Decision: DecisionDeny,
		Code:     "AMOUNT_TOO_HIGH",
		Message:  "Transaction amount exceeds limit",
		Metadata: map[string]interface{}{
			"limit": 10000,
		},
	}

	data, err := json.Marshal(consequence)
	if err != nil {
		t.Fatalf("failed to marshal consequence: %v", err)
	}

	var decoded Consequence
	if err := json.Unmarshal(data, &decoded); err != nil {
		t.Fatalf("failed to unmarshal consequence: %v", err)
	}

	if decoded.Decision != DecisionDeny {
		t.Errorf("expected decision %q, got %q", DecisionDeny, decoded.Decision)
	}
	if decoded.Code != "AMOUNT_TOO_HIGH" {
		t.Errorf("expected code %q, got %q", "AMOUNT_TOO_HIGH", decoded.Code)
	}
}

func TestPactRuleSerialization(t *testing.T) {
	now := time.Now().UTC().Truncate(time.Second)
	rule := PactRule{
		ID:           "rule_123",
		Name:         "High Amount Check",
		Description:  "Flags transactions over 10000",
		Jurisdiction: "US",
		Severity:     SeverityHigh,
		Condition:    CompareCondition{Field: "amount", Operator: "gt", Value: 10000},
		Consequence: Consequence{
			Decision: DecisionFlag,
			Code:     "HIGH_AMOUNT",
			Message:  "High amount transaction",
		},
		Tags:          []string{"compliance", "aml"},
		EffectiveFrom: &now,
		Metadata: map[string]interface{}{
			"regulation": "BSA",
		},
	}

	data, err := json.Marshal(rule)
	if err != nil {
		t.Fatalf("failed to marshal rule: %v", err)
	}

	var decoded map[string]interface{}
	if err := json.Unmarshal(data, &decoded); err != nil {
		t.Fatalf("failed to unmarshal rule: %v", err)
	}

	if decoded["id"] != "rule_123" {
		t.Errorf("expected id %q, got %v", "rule_123", decoded["id"])
	}
	if decoded["severity"] != "HIGH" {
		t.Errorf("expected severity %q, got %v", "HIGH", decoded["severity"])
	}
}

func TestRuleEvaluationSerialization(t *testing.T) {
	evaluation := RuleEvaluation{
		RuleID:     "rule_123",
		RuleName:   "High Amount Check",
		Result:     EvaluationResultFail,
		Code:       "HIGH_AMOUNT",
		Message:    "Amount exceeds limit",
		DurationMs: 5,
	}

	data, err := json.Marshal(evaluation)
	if err != nil {
		t.Fatalf("failed to marshal evaluation: %v", err)
	}

	var decoded RuleEvaluation
	if err := json.Unmarshal(data, &decoded); err != nil {
		t.Fatalf("failed to unmarshal evaluation: %v", err)
	}

	if decoded.RuleID != evaluation.RuleID {
		t.Errorf("expected RuleID %q, got %q", evaluation.RuleID, decoded.RuleID)
	}
	if decoded.Result != EvaluationResultFail {
		t.Errorf("expected Result %q, got %q", EvaluationResultFail, decoded.Result)
	}
}

func TestPactDecisionSerialization(t *testing.T) {
	now := time.Now().UTC().Truncate(time.Second)
	decision := PactDecision{
		ID:      "dec_123",
		EventID: "evt_456",
		Status:  DecisionStatusAllowWithFlags,
		RuleEvaluations: []RuleEvaluation{
			{RuleID: "rule_1", Result: EvaluationResultPass},
			{RuleID: "rule_2", Result: EvaluationResultFail, Code: "FLAG_1"},
		},
		CreatedAt: now,
		Metadata: map[string]interface{}{
			"processing_time_ms": 25,
		},
	}

	data, err := json.Marshal(decision)
	if err != nil {
		t.Fatalf("failed to marshal decision: %v", err)
	}

	var decoded PactDecision
	if err := json.Unmarshal(data, &decoded); err != nil {
		t.Fatalf("failed to unmarshal decision: %v", err)
	}

	if decoded.ID != decision.ID {
		t.Errorf("expected ID %q, got %q", decision.ID, decoded.ID)
	}
	if decoded.Status != DecisionStatusAllowWithFlags {
		t.Errorf("expected Status %q, got %q", DecisionStatusAllowWithFlags, decoded.Status)
	}
	if len(decoded.RuleEvaluations) != 2 {
		t.Errorf("expected 2 evaluations, got %d", len(decoded.RuleEvaluations))
	}
}

func TestValidationResultSerialization(t *testing.T) {
	result := ValidationResult{
		Valid: false,
		Errors: []ValidationError{
			{Field: "email", Message: "invalid email format", Code: "INVALID_FORMAT"},
			{Field: "age", Message: "must be positive", Code: "INVALID_VALUE"},
		},
		Warnings: []ValidationWarning{
			{Field: "phone", Message: "phone is recommended"},
		},
	}

	data, err := json.Marshal(result)
	if err != nil {
		t.Fatalf("failed to marshal result: %v", err)
	}

	var decoded ValidationResult
	if err := json.Unmarshal(data, &decoded); err != nil {
		t.Fatalf("failed to unmarshal result: %v", err)
	}

	if decoded.Valid {
		t.Error("expected Valid to be false")
	}
	if len(decoded.Errors) != 2 {
		t.Errorf("expected 2 errors, got %d", len(decoded.Errors))
	}
	if len(decoded.Warnings) != 1 {
		t.Errorf("expected 1 warning, got %d", len(decoded.Warnings))
	}
}

func TestDomainEventSerialization(t *testing.T) {
	now := time.Now().UTC().Truncate(time.Second)
	event := DomainEvent{
		Type: "payment_processed",
		Payload: map[string]interface{}{
			"amount":   500.0,
			"currency": "USD",
		},
		EntityID:     "txn_123",
		Jurisdiction: "US",
		OccurredAt:   &now,
		Metadata: map[string]interface{}{
			"source": "api",
		},
	}

	data, err := json.Marshal(event)
	if err != nil {
		t.Fatalf("failed to marshal event: %v", err)
	}

	var decoded DomainEvent
	if err := json.Unmarshal(data, &decoded); err != nil {
		t.Fatalf("failed to unmarshal event: %v", err)
	}

	if decoded.Type != event.Type {
		t.Errorf("expected Type %q, got %q", event.Type, decoded.Type)
	}
	if decoded.EntityID != event.EntityID {
		t.Errorf("expected EntityID %q, got %q", event.EntityID, decoded.EntityID)
	}
}

func TestNestedConditionSerialization(t *testing.T) {
	// Test complex nested condition
	condition := And(
		Or(
			CompareCondition{Field: "amount", Operator: "gt", Value: 1000},
			CompareCondition{Field: "risk_score", Operator: "gt", Value: 0.8},
		),
		Not(
			InCondition{Field: "status", Values: []interface{}{"blocked", "suspended"}},
		),
		ExistsCondition{Field: "verified_at"},
	)

	data, err := json.Marshal(condition)
	if err != nil {
		t.Fatalf("failed to marshal nested condition: %v", err)
	}

	var result map[string]interface{}
	if err := json.Unmarshal(data, &result); err != nil {
		t.Fatalf("failed to unmarshal nested condition: %v", err)
	}

	if result["type"] != "and" {
		t.Errorf("expected type %q, got %v", "and", result["type"])
	}

	conditions, ok := result["conditions"].([]interface{})
	if !ok {
		t.Fatal("expected conditions array")
	}
	if len(conditions) != 3 {
		t.Errorf("expected 3 conditions, got %d", len(conditions))
	}
}
