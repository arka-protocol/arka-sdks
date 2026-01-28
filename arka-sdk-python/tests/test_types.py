"""
Tests for PACT SDK type definitions.
"""

import pytest
from datetime import datetime
from pydantic import ValidationError as PydanticValidationError

from arka_sdk import (
    PluginManifest,
    DomainEvent,
    PactEvent,
    PactEntity,
    PactEntityType,
    PactRule,
    PactCondition,
    PactConsequence,
    PactDecision,
    ValidationResult,
    ValidationError,
    Severity,
    Decision,
    DecisionStatus,
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


class TestPluginManifest:
    """Tests for PluginManifest model."""

    def test_valid_manifest(self):
        """Test creating a valid plugin manifest."""
        manifest = PluginManifest(
            id="my-plugin",
            name="My Plugin",
            version="1.0.0",
            author="Test Author",
            description="A test plugin",
            entity_types=["Entity1", "Entity2"],
            event_types=["EVENT_1", "EVENT_2"],
            pact_core_version="0.1.0",
        )

        assert manifest.id == "my-plugin"
        assert manifest.name == "My Plugin"
        assert manifest.version == "1.0.0"
        assert len(manifest.entity_types) == 2
        assert len(manifest.event_types) == 2

    def test_manifest_with_dependencies(self):
        """Test manifest with dependencies."""
        manifest = PluginManifest(
            id="dependent-plugin",
            name="Dependent Plugin",
            version="1.0.0",
            author="Test",
            description="Plugin with dependencies",
            dependencies=["base-plugin"],
            pact_core_version="0.1.0",
        )

        assert "base-plugin" in manifest.dependencies

    def test_manifest_with_config_schema(self):
        """Test manifest with config schema."""
        config_schema = {
            "type": "object",
            "properties": {"api_key": {"type": "string"}},
        }
        manifest = PluginManifest(
            id="configurable-plugin",
            name="Configurable Plugin",
            version="1.0.0",
            author="Test",
            description="Plugin with config",
            pact_core_version="0.1.0",
            config_schema=config_schema,
        )

        assert manifest.config_schema == config_schema

    def test_manifest_missing_required_fields(self):
        """Test that missing required fields raises error."""
        with pytest.raises(PydanticValidationError):
            PluginManifest(
                id="incomplete",
                name="Incomplete Plugin",
                # Missing version, author, description, pact_core_version
            )


class TestDomainEvent:
    """Tests for DomainEvent model."""

    def test_minimal_domain_event(self):
        """Test creating minimal domain event."""
        event = DomainEvent(type="MY_EVENT")

        assert event.type == "MY_EVENT"
        assert event.payload == {}
        assert event.entity_id is None

    def test_full_domain_event(self):
        """Test creating full domain event."""
        now = datetime.utcnow()
        event = DomainEvent(
            type="MY_EVENT",
            payload={"key": "value"},
            entity_id="entity_123",
            jurisdiction="US",
            occurred_at=now,
            metadata={"source": "test"},
        )

        assert event.type == "MY_EVENT"
        assert event.payload["key"] == "value"
        assert event.entity_id == "entity_123"
        assert event.jurisdiction == "US"
        assert event.occurred_at == now
        assert event.metadata["source"] == "test"


class TestPactEvent:
    """Tests for PactEvent model."""

    def test_valid_pact_event(self):
        """Test creating valid PACT event."""
        now = datetime.utcnow()
        event = PactEvent(
            id="evt_123",
            source="my-plugin",
            type="MY_EVENT",
            entity_id="entity_123",
            entity_type="MyEntity",
            jurisdiction="US",
            payload={"amount": 1000},
            occurred_at=now,
            received_at=now,
        )

        assert event.id == "evt_123"
        assert event.source == "my-plugin"
        assert event.type == "MY_EVENT"
        assert event.payload["amount"] == 1000

    def test_pact_event_serialization(self):
        """Test PACT event JSON serialization."""
        now = datetime.utcnow()
        event = PactEvent(
            id="evt_123",
            source="my-plugin",
            type="MY_EVENT",
            occurred_at=now,
            received_at=now,
        )

        json_data = event.model_dump(mode="json")
        assert json_data["id"] == "evt_123"
        assert "occurred_at" in json_data


class TestPactEntity:
    """Tests for PactEntity model."""

    def test_valid_entity(self):
        """Test creating valid entity."""
        now = datetime.utcnow()
        entity = PactEntity(
            id="ent_123",
            type="MyEntity",
            data={"name": "Test"},
            created_at=now,
            updated_at=now,
        )

        assert entity.id == "ent_123"
        assert entity.type == "MyEntity"
        assert entity.data["name"] == "Test"

    def test_entity_with_jurisdiction(self):
        """Test entity with jurisdiction."""
        now = datetime.utcnow()
        entity = PactEntity(
            id="ent_123",
            type="MyEntity",
            data={},
            created_at=now,
            updated_at=now,
            jurisdiction="EU",
            metadata={"region": "DE"},
        )

        assert entity.jurisdiction == "EU"
        assert entity.metadata["region"] == "DE"


class TestConditionTypes:
    """Tests for condition types."""

    def test_compare_condition(self):
        """Test compare condition."""
        cond = CompareCondition(
            field="amount",
            operator="gt",
            value=1000,
        )

        assert cond.type == "compare"
        assert cond.field == "amount"
        assert cond.operator == "gt"
        assert cond.value == 1000

    def test_and_condition(self):
        """Test AND condition."""
        cond1 = CompareCondition(field="amount", operator="gt", value=1000)
        cond2 = CompareCondition(field="status", operator="eq", value="active")

        and_cond = AndCondition(conditions=[cond1, cond2])

        assert and_cond.type == "and"
        assert len(and_cond.conditions) == 2

    def test_or_condition(self):
        """Test OR condition."""
        cond1 = CompareCondition(field="country", operator="eq", value="US")
        cond2 = CompareCondition(field="country", operator="eq", value="CA")

        or_cond = OrCondition(conditions=[cond1, cond2])

        assert or_cond.type == "or"
        assert len(or_cond.conditions) == 2

    def test_not_condition(self):
        """Test NOT condition."""
        inner = CompareCondition(field="blocked", operator="eq", value=True)
        not_cond = NotCondition(condition=inner)

        assert not_cond.type == "not"
        assert not_cond.condition == inner

    def test_exists_condition(self):
        """Test exists condition."""
        cond = ExistsCondition(field="optional_field")

        assert cond.type == "exists"
        assert cond.field == "optional_field"

    def test_in_condition(self):
        """Test IN condition."""
        cond = InCondition(
            field="status",
            values=["active", "pending", "approved"],
        )

        assert cond.type == "in"
        assert len(cond.values) == 3

    def test_range_condition(self):
        """Test range condition."""
        cond = RangeCondition(
            field="amount",
            min=100,
            max=10000,
            min_inclusive=True,
            max_inclusive=False,
        )

        assert cond.type == "range"
        assert cond.min == 100
        assert cond.max == 10000
        assert cond.min_inclusive is True
        assert cond.max_inclusive is False

    def test_expression_condition(self):
        """Test expression condition."""
        cond = ExpressionCondition(
            expression="event.amount > entity.limit",
            language="cel",
        )

        assert cond.type == "expression"
        assert cond.language == "cel"

    def test_nested_conditions(self):
        """Test nested conditions."""
        # (amount > 1000 AND status == "active") OR country IN ["US", "CA"]
        cond1 = CompareCondition(field="amount", operator="gt", value=1000)
        cond2 = CompareCondition(field="status", operator="eq", value="active")
        and_cond = AndCondition(conditions=[cond1, cond2])

        cond3 = InCondition(field="country", values=["US", "CA"])
        or_cond = OrCondition(conditions=[and_cond, cond3])

        assert or_cond.type == "or"
        assert or_cond.conditions[0].type == "and"


class TestPactRule:
    """Tests for PactRule model."""

    def test_valid_rule(self):
        """Test creating valid rule."""
        rule = PactRule(
            id="rule_001",
            name="Test Rule",
            description="A test rule",
            jurisdiction="US",
            severity=Severity.HIGH,
            condition=CompareCondition(
                field="amount",
                operator="gt",
                value=10000,
            ),
            consequence=PactConsequence(
                decision=Decision.FLAG,
                code="HIGH_AMOUNT",
                message="Amount exceeds threshold",
            ),
            tags=["aml", "threshold"],
        )

        assert rule.id == "rule_001"
        assert rule.name == "Test Rule"
        assert rule.severity == Severity.HIGH
        assert rule.consequence.decision == Decision.FLAG

    def test_rule_with_effective_dates(self):
        """Test rule with effective dates."""
        from_date = datetime(2024, 1, 1)
        to_date = datetime(2024, 12, 31)

        rule = PactRule(
            id="rule_002",
            name="Temporary Rule",
            condition=CompareCondition(field="x", operator="eq", value=1),
            consequence=PactConsequence(
                decision=Decision.FLAG,
                code="TEMP",
                message="Temporary",
            ),
            effective_from=from_date,
            effective_to=to_date,
        )

        assert rule.effective_from == from_date
        assert rule.effective_to == to_date


class TestPactDecision:
    """Tests for PactDecision model."""

    def test_allow_decision(self):
        """Test ALLOW decision."""
        decision = PactDecision(
            id="dec_001",
            event_id="evt_001",
            status=DecisionStatus.ALLOW,
            rule_evaluations=[],
            created_at=datetime.utcnow(),
        )

        assert decision.status == DecisionStatus.ALLOW

    def test_deny_decision(self):
        """Test DENY decision."""
        decision = PactDecision(
            id="dec_002",
            event_id="evt_002",
            status=DecisionStatus.DENY,
            rule_evaluations=[],
            created_at=datetime.utcnow(),
        )

        assert decision.status == DecisionStatus.DENY

    def test_allow_with_flags_decision(self):
        """Test ALLOW_WITH_FLAGS decision."""
        from arka_sdk.types import RuleEvaluation

        decision = PactDecision(
            id="dec_003",
            event_id="evt_003",
            status=DecisionStatus.ALLOW_WITH_FLAGS,
            rule_evaluations=[
                RuleEvaluation(
                    rule_id="rule_001",
                    rule_name="Test Rule",
                    result="FAIL",
                    code="HIGH_AMOUNT",
                    message="Amount exceeds threshold",
                ),
            ],
            created_at=datetime.utcnow(),
        )

        assert decision.status == DecisionStatus.ALLOW_WITH_FLAGS
        assert len(decision.rule_evaluations) == 1


class TestValidationResult:
    """Tests for ValidationResult model."""

    def test_valid_result(self):
        """Test valid validation result."""
        result = ValidationResult(valid=True, errors=[], warnings=[])

        assert result.valid is True
        assert len(result.errors) == 0

    def test_invalid_result(self):
        """Test invalid validation result."""
        result = ValidationResult(
            valid=False,
            errors=[
                ValidationError(
                    field="amount",
                    message="Amount is required",
                    code="REQUIRED_FIELD",
                ),
            ],
        )

        assert result.valid is False
        assert len(result.errors) == 1
        assert result.errors[0].field == "amount"


class TestEnums:
    """Tests for enum types."""

    def test_severity_enum(self):
        """Test Severity enum."""
        assert Severity.LOW.value == "LOW"
        assert Severity.MEDIUM.value == "MEDIUM"
        assert Severity.HIGH.value == "HIGH"
        assert Severity.CRITICAL.value == "CRITICAL"

    def test_decision_enum(self):
        """Test Decision enum."""
        assert Decision.ALLOW.value == "ALLOW"
        assert Decision.DENY.value == "DENY"
        assert Decision.FLAG.value == "FLAG"

    def test_decision_status_enum(self):
        """Test DecisionStatus enum."""
        assert DecisionStatus.ALLOW.value == "ALLOW"
        assert DecisionStatus.ALLOW_WITH_FLAGS.value == "ALLOW_WITH_FLAGS"
        assert DecisionStatus.DENY.value == "DENY"
