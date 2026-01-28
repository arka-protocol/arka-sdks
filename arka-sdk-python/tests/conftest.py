"""
Pytest configuration and fixtures for PACT SDK tests.
"""

import pytest
from datetime import datetime
from typing import Any
from unittest.mock import AsyncMock, MagicMock

from arka_sdk import (
    BasePactPlugin,
    PluginManifest,
    PactEntityType,
    PactRule,
    PactCondition,
    PactConsequence,
    PactEvent,
    PactEntity,
    PactDecision,
    DomainEvent,
    Decision,
    DecisionStatus,
    Severity,
    ValidationResult,
    ValidationError,
    RuleBuilder,
    ConditionBuilder,
    rule,
    condition,
)
from arka_sdk.types import CompareCondition


class TestPlugin(BasePactPlugin):
    """A test plugin implementation for testing purposes."""

    @property
    def manifest(self) -> PluginManifest:
        return PluginManifest(
            id="test-plugin",
            name="Test Plugin",
            version="1.0.0",
            author="Test Author",
            description="A plugin for testing",
            entity_types=["TestEntity", "AnotherEntity"],
            event_types=["TEST_CREATED", "TEST_UPDATED"],
            pact_core_version="0.1.0",
        )

    @property
    def _entity_types(self) -> list[PactEntityType]:
        return [
            PactEntityType(
                name="TestEntity",
                description="A test entity",
                schema={
                    "type": "object",
                    "properties": {
                        "name": {"type": "string"},
                        "amount": {"type": "number"},
                        "status": {"type": "string"},
                    },
                },
                required_fields=["name", "amount"],
            ),
            PactEntityType(
                name="AnotherEntity",
                description="Another test entity",
                schema={
                    "type": "object",
                    "properties": {
                        "title": {"type": "string"},
                    },
                },
                required_fields=["title"],
            ),
        ]

    @property
    def _default_rules(self) -> list[PactRule]:
        return [
            PactRule(
                id="test-rule-001",
                name="High Amount Check",
                description="Flag transactions over threshold",
                jurisdiction="US",
                severity=Severity.HIGH,
                condition=CompareCondition(
                    field="payload.amount",
                    operator="gt",
                    value=10000,
                ),
                consequence=PactConsequence(
                    decision=Decision.FLAG,
                    code="HIGH_AMOUNT",
                    message="Amount exceeds threshold",
                ),
                tags=["aml", "threshold"],
            ),
        ]


@pytest.fixture
def test_plugin() -> TestPlugin:
    """Provides a test plugin instance."""
    return TestPlugin()


@pytest.fixture
def sample_domain_event() -> DomainEvent:
    """Provides a sample domain event."""
    return DomainEvent(
        type="TEST_CREATED",
        payload={"name": "Test", "amount": 15000},
        entity_id="entity_123",
        jurisdiction="US",
    )


@pytest.fixture
def sample_pact_event() -> PactEvent:
    """Provides a sample PACT event."""
    now = datetime.utcnow()
    return PactEvent(
        id="evt_test123",
        source="test-plugin",
        type="TEST_CREATED",
        entity_id="entity_123",
        entity_type="TestEntity",
        jurisdiction="US",
        payload={"name": "Test", "amount": 15000},
        occurred_at=now,
        received_at=now,
    )


@pytest.fixture
def sample_entity() -> PactEntity:
    """Provides a sample PACT entity."""
    now = datetime.utcnow()
    return PactEntity(
        id="ent_test123",
        type="TestEntity",
        data={"name": "Test Entity", "amount": 5000},
        created_at=now,
        updated_at=now,
        jurisdiction="US",
    )


@pytest.fixture
def sample_decision() -> PactDecision:
    """Provides a sample PACT decision."""
    return PactDecision(
        id="dec_test123",
        event_id="evt_test123",
        status=DecisionStatus.ALLOW,
        rule_evaluations=[],
        created_at=datetime.utcnow(),
    )


@pytest.fixture
def sample_rule() -> PactRule:
    """Provides a sample PACT rule."""
    return PactRule(
        id="rule_sample",
        name="Sample Rule",
        description="A sample rule for testing",
        jurisdiction="US",
        severity=Severity.MEDIUM,
        condition=CompareCondition(
            field="amount",
            operator="gt",
            value=1000,
        ),
        consequence=PactConsequence(
            decision=Decision.FLAG,
            code="SAMPLE",
            message="Sample flag",
        ),
    )
