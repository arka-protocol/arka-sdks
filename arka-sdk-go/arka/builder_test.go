package pact

import (
	"encoding/json"
	"testing"
	"time"
)

func TestConditionBuilder(t *testing.T) {
	t.Run("creates equality condition", func(t *testing.T) {
		condition := NewCondition().Field("status").Eq("active").Build()

		compareCondition, ok := condition.(CompareCondition)
		if !ok {
			t.Fatal("expected CompareCondition")
		}

		if compareCondition.Field != "status" {
			t.Errorf("expected field %q, got %q", "status", compareCondition.Field)
		}
		if compareCondition.Operator != "eq" {
			t.Errorf("expected operator %q, got %q", "eq", compareCondition.Operator)
		}
		if compareCondition.Value != "active" {
			t.Errorf("expected value %q, got %v", "active", compareCondition.Value)
		}
	})

	t.Run("creates not-equal condition", func(t *testing.T) {
		condition := NewCondition().Field("status").Ne("blocked").Build()

		compareCondition, ok := condition.(CompareCondition)
		if !ok {
			t.Fatal("expected CompareCondition")
		}

		if compareCondition.Operator != "ne" {
			t.Errorf("expected operator %q, got %q", "ne", compareCondition.Operator)
		}
	})

	t.Run("creates greater-than condition", func(t *testing.T) {
		condition := NewCondition().Field("amount").Gt(1000).Build()

		compareCondition, ok := condition.(CompareCondition)
		if !ok {
			t.Fatal("expected CompareCondition")
		}

		if compareCondition.Operator != "gt" {
			t.Errorf("expected operator %q, got %q", "gt", compareCondition.Operator)
		}
		if compareCondition.Value != 1000 {
			t.Errorf("expected value %d, got %v", 1000, compareCondition.Value)
		}
	})

	t.Run("creates greater-than-or-equal condition", func(t *testing.T) {
		condition := NewCondition().Field("amount").Gte(1000).Build()

		compareCondition, ok := condition.(CompareCondition)
		if !ok {
			t.Fatal("expected CompareCondition")
		}

		if compareCondition.Operator != "gte" {
			t.Errorf("expected operator %q, got %q", "gte", compareCondition.Operator)
		}
	})

	t.Run("creates less-than condition", func(t *testing.T) {
		condition := NewCondition().Field("amount").Lt(100).Build()

		compareCondition, ok := condition.(CompareCondition)
		if !ok {
			t.Fatal("expected CompareCondition")
		}

		if compareCondition.Operator != "lt" {
			t.Errorf("expected operator %q, got %q", "lt", compareCondition.Operator)
		}
	})

	t.Run("creates less-than-or-equal condition", func(t *testing.T) {
		condition := NewCondition().Field("amount").Lte(100).Build()

		compareCondition, ok := condition.(CompareCondition)
		if !ok {
			t.Fatal("expected CompareCondition")
		}

		if compareCondition.Operator != "lte" {
			t.Errorf("expected operator %q, got %q", "lte", compareCondition.Operator)
		}
	})

	t.Run("creates contains condition", func(t *testing.T) {
		condition := NewCondition().Field("description").Contains("fraud").Build()

		compareCondition, ok := condition.(CompareCondition)
		if !ok {
			t.Fatal("expected CompareCondition")
		}

		if compareCondition.Operator != "contains" {
			t.Errorf("expected operator %q, got %q", "contains", compareCondition.Operator)
		}
	})

	t.Run("creates startsWith condition", func(t *testing.T) {
		condition := NewCondition().Field("name").StartsWith("John").Build()

		compareCondition, ok := condition.(CompareCondition)
		if !ok {
			t.Fatal("expected CompareCondition")
		}

		if compareCondition.Operator != "startsWith" {
			t.Errorf("expected operator %q, got %q", "startsWith", compareCondition.Operator)
		}
	})

	t.Run("creates endsWith condition", func(t *testing.T) {
		condition := NewCondition().Field("email").EndsWith("@example.com").Build()

		compareCondition, ok := condition.(CompareCondition)
		if !ok {
			t.Fatal("expected CompareCondition")
		}

		if compareCondition.Operator != "endsWith" {
			t.Errorf("expected operator %q, got %q", "endsWith", compareCondition.Operator)
		}
	})

	t.Run("creates regex match condition", func(t *testing.T) {
		condition := NewCondition().Field("phone").Matches(`^\+1\d{10}$`).Build()

		compareCondition, ok := condition.(CompareCondition)
		if !ok {
			t.Fatal("expected CompareCondition")
		}

		if compareCondition.Operator != "regex" {
			t.Errorf("expected operator %q, got %q", "regex", compareCondition.Operator)
		}
	})

	t.Run("creates exists condition", func(t *testing.T) {
		condition := NewCondition().Field("verified_at").Exists().Build()

		existsCondition, ok := condition.(ExistsCondition)
		if !ok {
			t.Fatal("expected ExistsCondition")
		}

		if existsCondition.Field != "verified_at" {
			t.Errorf("expected field %q, got %q", "verified_at", existsCondition.Field)
		}
	})

	t.Run("creates in condition", func(t *testing.T) {
		condition := NewCondition().Field("status").In("active", "pending", "approved").Build()

		inCondition, ok := condition.(InCondition)
		if !ok {
			t.Fatal("expected InCondition")
		}

		if inCondition.Field != "status" {
			t.Errorf("expected field %q, got %q", "status", inCondition.Field)
		}
		if len(inCondition.Values) != 3 {
			t.Errorf("expected 3 values, got %d", len(inCondition.Values))
		}
	})

	t.Run("creates between condition", func(t *testing.T) {
		condition := NewCondition().Field("amount").Between(100, 1000).Build()

		rangeCondition, ok := condition.(RangeCondition)
		if !ok {
			t.Fatal("expected RangeCondition")
		}

		if rangeCondition.Field != "amount" {
			t.Errorf("expected field %q, got %q", "amount", rangeCondition.Field)
		}
		if *rangeCondition.Min != 100 {
			t.Errorf("expected min %v, got %v", 100.0, *rangeCondition.Min)
		}
		if *rangeCondition.Max != 1000 {
			t.Errorf("expected max %v, got %v", 1000.0, *rangeCondition.Max)
		}
		if !rangeCondition.MinInclusive {
			t.Error("expected MinInclusive to be true")
		}
		if !rangeCondition.MaxInclusive {
			t.Error("expected MaxInclusive to be true")
		}
	})

	t.Run("creates expression condition", func(t *testing.T) {
		condition := NewCondition().Expression("event.amount > 1000", "cel").Build()

		exprCondition, ok := condition.(ExpressionCondition)
		if !ok {
			t.Fatal("expected ExpressionCondition")
		}

		if exprCondition.Expression != "event.amount > 1000" {
			t.Errorf("expected expression %q, got %q", "event.amount > 1000", exprCondition.Expression)
		}
		if exprCondition.Language != "cel" {
			t.Errorf("expected language %q, got %q", "cel", exprCondition.Language)
		}
	})

	t.Run("defaults expression language to cel", func(t *testing.T) {
		condition := NewCondition().Expression("event.amount > 1000", "").Build()

		exprCondition, ok := condition.(ExpressionCondition)
		if !ok {
			t.Fatal("expected ExpressionCondition")
		}

		if exprCondition.Language != "cel" {
			t.Errorf("expected language %q, got %q", "cel", exprCondition.Language)
		}
	})
}

func TestLogicalConditionBuilders(t *testing.T) {
	t.Run("creates AND condition", func(t *testing.T) {
		condition := And(
			NewCondition().Field("amount").Gt(1000).Build(),
			NewCondition().Field("status").Eq("pending").Build(),
		)

		andCondition, ok := condition.(AndCondition)
		if !ok {
			t.Fatal("expected AndCondition")
		}

		if len(andCondition.Conditions) != 2 {
			t.Errorf("expected 2 conditions, got %d", len(andCondition.Conditions))
		}
	})

	t.Run("creates OR condition", func(t *testing.T) {
		condition := Or(
			NewCondition().Field("amount").Gt(10000).Build(),
			NewCondition().Field("risk_score").Gt(0.9).Build(),
		)

		orCondition, ok := condition.(OrCondition)
		if !ok {
			t.Fatal("expected OrCondition")
		}

		if len(orCondition.Conditions) != 2 {
			t.Errorf("expected 2 conditions, got %d", len(orCondition.Conditions))
		}
	})

	t.Run("creates NOT condition", func(t *testing.T) {
		condition := Not(
			NewCondition().Field("status").Eq("blocked").Build(),
		)

		notCondition, ok := condition.(NotCondition)
		if !ok {
			t.Fatal("expected NotCondition")
		}

		_, ok = notCondition.Condition.(CompareCondition)
		if !ok {
			t.Fatal("expected inner condition to be CompareCondition")
		}
	})

	t.Run("creates nested logical conditions", func(t *testing.T) {
		condition := And(
			Or(
				NewCondition().Field("amount").Gt(1000).Build(),
				NewCondition().Field("risk_score").Gt(0.8).Build(),
			),
			Not(
				NewCondition().Field("status").In("blocked", "suspended").Build(),
			),
		)

		andCondition, ok := condition.(AndCondition)
		if !ok {
			t.Fatal("expected AndCondition")
		}

		if len(andCondition.Conditions) != 2 {
			t.Errorf("expected 2 conditions, got %d", len(andCondition.Conditions))
		}

		_, ok = andCondition.Conditions[0].(OrCondition)
		if !ok {
			t.Fatal("expected first condition to be OrCondition")
		}

		_, ok = andCondition.Conditions[1].(NotCondition)
		if !ok {
			t.Fatal("expected second condition to be NotCondition")
		}
	})
}

func TestRuleBuilder(t *testing.T) {
	t.Run("creates basic rule", func(t *testing.T) {
		rule, err := NewRule().
			Name("High Amount Check").
			When(NewCondition().Field("amount").Gt(10000).Build()).
			ThenDeny("HIGH_AMOUNT", "Amount exceeds limit").
			Build()

		if err != nil {
			t.Fatalf("Build failed: %v", err)
		}

		if rule.Name != "High Amount Check" {
			t.Errorf("expected name %q, got %q", "High Amount Check", rule.Name)
		}
		if rule.Description != "High Amount Check" {
			t.Errorf("expected description %q, got %q", "High Amount Check", rule.Description)
		}
		if rule.Severity != SeverityMedium {
			t.Errorf("expected severity %q, got %q", SeverityMedium, rule.Severity)
		}
		if rule.Consequence.Decision != DecisionDeny {
			t.Errorf("expected decision %q, got %q", DecisionDeny, rule.Consequence.Decision)
		}
		if rule.ID == "" {
			t.Error("expected generated ID")
		}
	})

	t.Run("creates rule with all options", func(t *testing.T) {
		now := time.Now().UTC()
		later := now.Add(24 * time.Hour)

		rule, err := NewRule().
			ID("rule_custom_123").
			Name("Custom Rule").
			Description("A custom rule with all options").
			Jurisdiction("US").
			Severity(SeverityCritical).
			When(NewCondition().Field("amount").Gt(50000).Build()).
			ThenDeny("CRITICAL_AMOUNT", "Critical amount detected").
			Tags("aml", "compliance", "critical").
			EffectiveFrom(now).
			EffectiveTo(later).
			Metadata("regulation", "BSA").
			Metadata("version", "1.0").
			Build()

		if err != nil {
			t.Fatalf("Build failed: %v", err)
		}

		if rule.ID != "rule_custom_123" {
			t.Errorf("expected ID %q, got %q", "rule_custom_123", rule.ID)
		}
		if rule.Description != "A custom rule with all options" {
			t.Errorf("expected description %q, got %q", "A custom rule with all options", rule.Description)
		}
		if rule.Jurisdiction != "US" {
			t.Errorf("expected jurisdiction %q, got %q", "US", rule.Jurisdiction)
		}
		if rule.Severity != SeverityCritical {
			t.Errorf("expected severity %q, got %q", SeverityCritical, rule.Severity)
		}
		if len(rule.Tags) != 3 {
			t.Errorf("expected 3 tags, got %d", len(rule.Tags))
		}
		if rule.EffectiveFrom == nil {
			t.Error("expected EffectiveFrom to be set")
		}
		if rule.EffectiveTo == nil {
			t.Error("expected EffectiveTo to be set")
		}
		if rule.Metadata["regulation"] != "BSA" {
			t.Errorf("expected metadata regulation %q, got %v", "BSA", rule.Metadata["regulation"])
		}
	})

	t.Run("creates rule with WhenField shorthand", func(t *testing.T) {
		rule, err := NewRule().
			Name("Simple Rule").
			WhenField("status", "eq", "blocked").
			ThenDeny("BLOCKED", "User is blocked").
			Build()

		if err != nil {
			t.Fatalf("Build failed: %v", err)
		}

		compareCondition, ok := rule.Condition.(CompareCondition)
		if !ok {
			t.Fatal("expected CompareCondition")
		}

		if compareCondition.Field != "status" {
			t.Errorf("expected field %q, got %q", "status", compareCondition.Field)
		}
	})

	t.Run("creates rule with ThenFlag consequence", func(t *testing.T) {
		rule, err := NewRule().
			Name("Flag Rule").
			When(NewCondition().Field("amount").Gt(5000).Build()).
			ThenFlag("REVIEW_REQUIRED", "Manual review required").
			Build()

		if err != nil {
			t.Fatalf("Build failed: %v", err)
		}

		if rule.Consequence.Decision != DecisionFlag {
			t.Errorf("expected decision %q, got %q", DecisionFlag, rule.Consequence.Decision)
		}
	})

	t.Run("creates rule with ThenAllow consequence", func(t *testing.T) {
		rule, err := NewRule().
			Name("Allow Rule").
			When(NewCondition().Field("verified").Eq(true).Build()).
			ThenAllow("VERIFIED", "User is verified").
			Build()

		if err != nil {
			t.Fatalf("Build failed: %v", err)
		}

		if rule.Consequence.Decision != DecisionAllow {
			t.Errorf("expected decision %q, got %q", DecisionAllow, rule.Consequence.Decision)
		}
	})

	t.Run("fails without name", func(t *testing.T) {
		_, err := NewRule().
			When(NewCondition().Field("a").Eq(1).Build()).
			ThenDeny("CODE", "Message").
			Build()

		if err == nil {
			t.Error("expected error for missing name")
		}
	})

	t.Run("fails without condition", func(t *testing.T) {
		_, err := NewRule().
			Name("Test Rule").
			ThenDeny("CODE", "Message").
			Build()

		if err == nil {
			t.Error("expected error for missing condition")
		}
	})

	t.Run("fails without consequence", func(t *testing.T) {
		_, err := NewRule().
			Name("Test Rule").
			When(NewCondition().Field("a").Eq(1).Build()).
			Build()

		if err == nil {
			t.Error("expected error for missing consequence")
		}
	})

	t.Run("MustBuild returns rule", func(t *testing.T) {
		rule := NewRule().
			Name("Must Build Rule").
			When(NewCondition().Field("a").Eq(1).Build()).
			ThenDeny("CODE", "Message").
			MustBuild()

		if rule.Name != "Must Build Rule" {
			t.Errorf("expected name %q, got %q", "Must Build Rule", rule.Name)
		}
	})

	t.Run("MustBuild panics on error", func(t *testing.T) {
		defer func() {
			if r := recover(); r == nil {
				t.Error("expected panic for invalid rule")
			}
		}()

		NewRule().MustBuild()
	})
}

func TestConditionBuilderSerialization(t *testing.T) {
	t.Run("serializes complex condition to JSON", func(t *testing.T) {
		condition := And(
			NewCondition().Field("amount").Gt(1000).Build(),
			Or(
				NewCondition().Field("status").Eq("pending").Build(),
				NewCondition().Field("risk_score").Gte(0.8).Build(),
			),
			Not(
				NewCondition().Field("country").In("blocked_country").Build(),
			),
		)

		data, err := json.Marshal(condition)
		if err != nil {
			t.Fatalf("failed to marshal condition: %v", err)
		}

		var result map[string]interface{}
		if err := json.Unmarshal(data, &result); err != nil {
			t.Fatalf("failed to unmarshal: %v", err)
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
	})
}

func TestRuleBuilderSerialization(t *testing.T) {
	t.Run("serializes complete rule to JSON", func(t *testing.T) {
		rule := NewRule().
			ID("rule_test_123").
			Name("Test Rule").
			Description("A test rule").
			Jurisdiction("US").
			Severity(SeverityHigh).
			When(
				And(
					NewCondition().Field("amount").Gt(10000).Build(),
					NewCondition().Field("status").Eq("pending").Build(),
				),
			).
			ThenDeny("HIGH_AMOUNT_PENDING", "High amount pending transaction").
			Tags("compliance", "aml").
			Metadata("source", "automated").
			MustBuild()

		data, err := json.Marshal(rule)
		if err != nil {
			t.Fatalf("failed to marshal rule: %v", err)
		}

		var result map[string]interface{}
		if err := json.Unmarshal(data, &result); err != nil {
			t.Fatalf("failed to unmarshal: %v", err)
		}

		if result["id"] != "rule_test_123" {
			t.Errorf("expected id %q, got %v", "rule_test_123", result["id"])
		}
		if result["name"] != "Test Rule" {
			t.Errorf("expected name %q, got %v", "Test Rule", result["name"])
		}
		if result["severity"] != "HIGH" {
			t.Errorf("expected severity %q, got %v", "HIGH", result["severity"])
		}
	})
}
