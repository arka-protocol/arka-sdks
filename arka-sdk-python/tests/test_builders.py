"""
Tests for Rule and Condition Builders.
"""

import pytest
from datetime import datetime

from arka_sdk import (
    RuleBuilder,
    ConditionBuilder,
    rule,
    condition,
    PactRule,
    PactCondition,
    PactConsequence,
    Severity,
    Decision,
)
from arka_sdk.types import (
    CompareCondition,
    AndCondition,
    OrCondition,
    NotCondition,
    ExistsCondition,
    InCondition,
    RangeCondition,
    ExpressionCondition,
)


class TestConditionBuilder:
    """Tests for ConditionBuilder."""

    def test_eq_condition(self):
        """Test building equality condition."""
        cond = condition().field("status").eq("active").build()

        assert isinstance(cond, CompareCondition)
        assert cond.field == "status"
        assert cond.operator == "eq"
        assert cond.value == "active"

    def test_ne_condition(self):
        """Test building not equal condition."""
        cond = condition().field("status").ne("deleted").build()

        assert cond.operator == "ne"
        assert cond.value == "deleted"

    def test_gt_condition(self):
        """Test building greater than condition."""
        cond = condition().field("amount").gt(1000).build()

        assert cond.operator == "gt"
        assert cond.value == 1000

    def test_gte_condition(self):
        """Test building greater than or equal condition."""
        cond = condition().field("amount").gte(1000).build()

        assert cond.operator == "gte"

    def test_lt_condition(self):
        """Test building less than condition."""
        cond = condition().field("amount").lt(100).build()

        assert cond.operator == "lt"
        assert cond.value == 100

    def test_lte_condition(self):
        """Test building less than or equal condition."""
        cond = condition().field("amount").lte(100).build()

        assert cond.operator == "lte"

    def test_contains_condition(self):
        """Test building string contains condition."""
        cond = condition().field("name").contains("test").build()

        assert cond.operator == "contains"
        assert cond.value == "test"

    def test_starts_with_condition(self):
        """Test building starts with condition."""
        cond = condition().field("code").starts_with("US-").build()

        assert cond.operator == "startsWith"
        assert cond.value == "US-"

    def test_ends_with_condition(self):
        """Test building ends with condition."""
        cond = condition().field("email").ends_with("@example.com").build()

        assert cond.operator == "endsWith"

    def test_matches_condition(self):
        """Test building regex match condition."""
        cond = condition().field("code").matches(r"^[A-Z]{2}-\d+$").build()

        assert cond.operator == "regex"

    def test_exists_condition(self):
        """Test building field exists condition."""
        cond = condition().field("optional_field").exists().build()

        assert isinstance(cond, ExistsCondition)
        assert cond.field == "optional_field"

    def test_in_condition(self):
        """Test building IN condition."""
        cond = condition().field("country").in_(["US", "CA", "MX"]).build()

        assert isinstance(cond, InCondition)
        assert cond.field == "country"
        assert cond.values == ["US", "CA", "MX"]

    def test_between_condition(self):
        """Test building range condition."""
        cond = condition().field("amount").between(100, 10000).build()

        assert isinstance(cond, RangeCondition)
        assert cond.min == 100
        assert cond.max == 10000
        assert cond.min_inclusive is True
        assert cond.max_inclusive is True

    def test_between_exclusive(self):
        """Test building exclusive range condition."""
        cond = (
            condition()
            .field("amount")
            .between(100, 10000, min_inclusive=False, max_inclusive=False)
            .build()
        )

        assert cond.min_inclusive is False
        assert cond.max_inclusive is False

    def test_and_condition(self):
        """Test building AND condition."""
        cond1 = condition().field("amount").gt(1000).build()
        cond2 = condition().field("status").eq("active").build()

        and_cond = condition().and_([cond1, cond2]).build()

        assert isinstance(and_cond, AndCondition)
        assert len(and_cond.conditions) == 2

    def test_or_condition(self):
        """Test building OR condition."""
        cond1 = condition().field("country").eq("US").build()
        cond2 = condition().field("country").eq("CA").build()

        or_cond = condition().or_([cond1, cond2]).build()

        assert isinstance(or_cond, OrCondition)
        assert len(or_cond.conditions) == 2

    def test_not_condition(self):
        """Test building NOT condition."""
        inner = condition().field("blocked").eq(True).build()
        not_cond = condition().not_(inner).build()

        assert isinstance(not_cond, NotCondition)
        assert not_cond.condition == inner

    def test_expression_condition(self):
        """Test building expression condition."""
        cond = condition().expression("event.amount > entity.limit").build()

        assert isinstance(cond, ExpressionCondition)
        assert cond.expression == "event.amount > entity.limit"
        assert cond.language == "cel"

    def test_expression_with_language(self):
        """Test expression with custom language."""
        cond = condition().expression("amount > 1000", language="python").build()

        assert cond.language == "python"

    def test_field_required_for_comparison(self):
        """Test that field must be set before comparison."""
        with pytest.raises(ValueError, match="Field must be set"):
            condition().eq("value").build()

    def test_field_required_for_exists(self):
        """Test that field must be set for exists."""
        with pytest.raises(ValueError, match="Field must be set"):
            condition().exists().build()

    def test_no_condition_error(self):
        """Test that build fails without condition."""
        with pytest.raises(ValueError, match="No condition set"):
            condition().build()

    def test_condition_builder_chaining(self):
        """Test that builder methods return self."""
        builder = ConditionBuilder()
        result = builder.field("test")
        assert result is builder

    def test_complex_nested_condition(self):
        """Test building complex nested conditions."""
        # (amount > 1000 AND amount < 100000) OR (status IN ["vip", "premium"])
        amount_min = condition().field("amount").gt(1000).build()
        amount_max = condition().field("amount").lt(100000).build()
        amount_range = condition().and_([amount_min, amount_max]).build()

        status_check = condition().field("status").in_(["vip", "premium"]).build()

        complex_cond = condition().or_([amount_range, status_check]).build()

        assert isinstance(complex_cond, OrCondition)
        assert isinstance(complex_cond.conditions[0], AndCondition)


class TestRuleBuilder:
    """Tests for RuleBuilder."""

    def test_minimal_rule(self):
        """Test building minimal rule."""
        r = (
            rule()
            .name("Test Rule")
            .when(condition().field("x").eq(1).build())
            .then_flag("TEST", "Test flag")
            .build()
        )

        assert isinstance(r, PactRule)
        assert r.name == "Test Rule"
        assert r.consequence.decision == Decision.FLAG

    def test_full_rule(self):
        """Test building full rule with all options."""
        r = (
            rule()
            .id("rule_custom_001")
            .name("High Value Check")
            .description("Flag transactions over $10,000")
            .jurisdiction("US")
            .severity("HIGH")
            .when(condition().field("amount").gt(10000).build())
            .then_flag("HIGH_VALUE", "Transaction exceeds threshold")
            .tags("aml", "threshold", "high-value")
            .metadata(category="financial", version=1)
            .build()
        )

        assert r.id == "rule_custom_001"
        assert r.name == "High Value Check"
        assert r.description == "Flag transactions over $10,000"
        assert r.jurisdiction == "US"
        assert r.severity == Severity.HIGH
        assert r.consequence.code == "HIGH_VALUE"
        assert "aml" in r.tags
        assert r.metadata["category"] == "financial"

    def test_auto_generated_id(self):
        """Test that rule ID is auto-generated if not provided."""
        r = (
            rule()
            .name("Auto ID Rule")
            .when(condition().field("x").eq(1).build())
            .then_allow()
            .build()
        )

        assert r.id.startswith("rule_")

    def test_then_deny(self):
        """Test DENY consequence."""
        r = (
            rule()
            .name("Deny Rule")
            .when(condition().field("blocked").eq(True).build())
            .then_deny("BLOCKED", "Entity is blocked")
            .build()
        )

        assert r.consequence.decision == Decision.DENY
        assert r.consequence.code == "BLOCKED"

    def test_then_allow(self):
        """Test ALLOW consequence."""
        r = (
            rule()
            .name("Allow Rule")
            .when(condition().field("verified").eq(True).build())
            .then_allow()
            .build()
        )

        assert r.consequence.decision == Decision.ALLOW

    def test_custom_consequence(self):
        """Test custom consequence."""
        consequence = PactConsequence(
            decision=Decision.FLAG,
            code="CUSTOM",
            message="Custom message",
            metadata={"extra": "data"},
        )

        r = (
            rule()
            .name("Custom Consequence Rule")
            .when(condition().field("x").eq(1).build())
            .consequence(consequence)
            .build()
        )

        assert r.consequence.metadata["extra"] == "data"

    def test_when_field_shorthand(self):
        """Test when_field shorthand method."""
        r = (
            rule()
            .name("Shorthand Rule")
            .when_field("amount", "gt", 1000)
            .then_flag("HIGH", "High amount")
            .build()
        )

        assert r.condition.field == "amount"
        assert r.condition.operator == "gt"

    def test_and_conditions(self):
        """Test AND conditions shorthand."""
        cond1 = condition().field("amount").gt(1000).build()
        cond2 = condition().field("verified").eq(True).build()

        r = (
            rule()
            .name("AND Rule")
            .and_conditions([cond1, cond2])
            .then_flag("COMBINED", "Both conditions met")
            .build()
        )

        assert isinstance(r.condition, AndCondition)

    def test_or_conditions(self):
        """Test OR conditions shorthand."""
        cond1 = condition().field("type").eq("premium").build()
        cond2 = condition().field("type").eq("vip").build()

        r = (
            rule()
            .name("OR Rule")
            .or_conditions([cond1, cond2])
            .then_allow()
            .build()
        )

        assert isinstance(r.condition, OrCondition)

    def test_effective_dates(self):
        """Test effective date range."""
        from_date = datetime(2024, 1, 1)
        to_date = datetime(2024, 12, 31)

        r = (
            rule()
            .name("Temporary Rule")
            .when(condition().field("x").eq(1).build())
            .then_flag("TEMP", "Temporary")
            .effective_from(from_date)
            .effective_to(to_date)
            .build()
        )

        assert r.effective_from == from_date
        assert r.effective_to == to_date

    def test_severity_as_string(self):
        """Test setting severity as string."""
        r = (
            rule()
            .name("Critical Rule")
            .severity("CRITICAL")
            .when(condition().field("x").eq(1).build())
            .then_deny("CRITICAL", "Critical")
            .build()
        )

        assert r.severity == Severity.CRITICAL

    def test_severity_as_enum(self):
        """Test setting severity as enum."""
        r = (
            rule()
            .name("Low Severity Rule")
            .severity(Severity.LOW)
            .when(condition().field("x").eq(1).build())
            .then_flag("LOW", "Low")
            .build()
        )

        assert r.severity == Severity.LOW

    def test_missing_name_error(self):
        """Test that name is required."""
        with pytest.raises(ValueError, match="Rule name is required"):
            (
                rule()
                .when(condition().field("x").eq(1).build())
                .then_flag("X", "X")
                .build()
            )

    def test_missing_condition_error(self):
        """Test that condition is required."""
        with pytest.raises(ValueError, match="Rule condition is required"):
            rule().name("No Condition").then_flag("X", "X").build()

    def test_missing_consequence_error(self):
        """Test that consequence is required."""
        with pytest.raises(ValueError, match="Rule consequence is required"):
            (
                rule()
                .name("No Consequence")
                .when(condition().field("x").eq(1).build())
                .build()
            )

    def test_rule_builder_chaining(self):
        """Test that builder methods return self."""
        builder = RuleBuilder()
        result = builder.name("Test")
        assert result is builder

    def test_default_severity(self):
        """Test default severity is MEDIUM."""
        r = (
            rule()
            .name("Default Severity")
            .when(condition().field("x").eq(1).build())
            .then_flag("X", "X")
            .build()
        )

        assert r.severity == Severity.MEDIUM


class TestBuilderFactoryFunctions:
    """Tests for factory functions."""

    def test_rule_factory(self):
        """Test rule() factory function."""
        builder = rule()
        assert isinstance(builder, RuleBuilder)

    def test_condition_factory(self):
        """Test condition() factory function."""
        builder = condition()
        assert isinstance(builder, ConditionBuilder)


class TestRealWorldExamples:
    """Real-world usage examples as tests."""

    def test_aml_threshold_rule(self):
        """Test AML threshold rule example."""
        aml_rule = (
            rule()
            .name("AML High Value Transaction")
            .description("Flag transactions over $10,000 per BSA requirements")
            .jurisdiction("US")
            .severity("HIGH")
            .when(condition().field("payload.amount").gt(10000).build())
            .then_flag("AML_THRESHOLD", "Transaction exceeds BSA reporting threshold")
            .tags("aml", "bsa", "threshold", "ctr")
            .build()
        )

        assert aml_rule.jurisdiction == "US"
        assert "bsa" in aml_rule.tags

    def test_kyc_verification_rule(self):
        """Test KYC verification rule example."""
        kyc_rule = (
            rule()
            .name("KYC Identity Verification Required")
            .description("Require identity verification for accounts with high limits")
            .when(
                condition()
                .and_(
                    [
                        condition().field("account.limit").gt(50000).build(),
                        condition().not_(
                            condition().field("account.kyc_verified").eq(True).build()
                        ).build(),
                    ]
                )
                .build()
            )
            .then_deny("KYC_REQUIRED", "Identity verification required for this limit")
            .severity("CRITICAL")
            .build()
        )

        assert isinstance(kyc_rule.condition, AndCondition)
        assert kyc_rule.consequence.decision == Decision.DENY

    def test_sanctions_check_rule(self):
        """Test sanctions check rule example."""
        sanctions_rule = (
            rule()
            .name("Sanctions List Check")
            .description("Block transactions involving sanctioned countries")
            .severity("CRITICAL")
            .when(
                condition()
                .field("counterparty.country")
                .in_(["KP", "IR", "SY", "CU"])
                .build()
            )
            .then_deny("OFAC_VIOLATION", "Transaction involves sanctioned country")
            .tags("sanctions", "ofac", "compliance")
            .build()
        )

        assert sanctions_rule.severity == Severity.CRITICAL
        assert sanctions_rule.consequence.decision == Decision.DENY
