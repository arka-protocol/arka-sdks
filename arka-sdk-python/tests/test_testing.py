"""
Tests for Plugin Testing Utilities.
"""

import pytest
from datetime import datetime
from typing import Any
from unittest.mock import AsyncMock

from arka_sdk import (
    PluginTestHarness,
    create_mock_event,
    create_mock_entity,
    create_mock_decision,
    PactEvent,
    PactEntity,
    PactDecision,
    PactRule,
    DomainEvent,
    DecisionStatus,
)
from arka_sdk.testing import run_simulation, TestResult, TestSuiteResult
from arka_sdk.types import RuleEvaluation


class TestCreateMockEvent:
    """Tests for create_mock_event utility."""

    def test_creates_valid_event(self):
        """Test that mock event is valid."""
        event = create_mock_event()

        assert isinstance(event, PactEvent)
        assert event.id.startswith("evt_test_")
        assert event.source == "test"
        assert event.type == "TEST_EVENT"

    def test_applies_overrides(self):
        """Test that overrides are applied."""
        event = create_mock_event({
            "source": "custom-source",
            "type": "CUSTOM_EVENT",
            "entity_id": "custom_entity",
            "payload": {"key": "value"},
        })

        assert event.source == "custom-source"
        assert event.type == "CUSTOM_EVENT"
        assert event.entity_id == "custom_entity"
        assert event.payload["key"] == "value"

    def test_generates_unique_ids(self):
        """Test that each mock event has unique ID."""
        event1 = create_mock_event()
        event2 = create_mock_event()

        assert event1.id != event2.id


class TestCreateMockEntity:
    """Tests for create_mock_entity utility."""

    def test_creates_valid_entity(self):
        """Test that mock entity is valid."""
        entity = create_mock_entity()

        assert isinstance(entity, PactEntity)
        assert entity.id.startswith("ent_test_")
        assert entity.type == "TestEntity"

    def test_applies_overrides(self):
        """Test that overrides are applied."""
        entity = create_mock_entity({
            "type": "CustomEntity",
            "data": {"name": "Custom"},
            "jurisdiction": "EU",
        })

        assert entity.type == "CustomEntity"
        assert entity.data["name"] == "Custom"
        assert entity.jurisdiction == "EU"

    def test_generates_unique_ids(self):
        """Test that each mock entity has unique ID."""
        entity1 = create_mock_entity()
        entity2 = create_mock_entity()

        assert entity1.id != entity2.id


class TestCreateMockDecision:
    """Tests for create_mock_decision utility."""

    def test_creates_valid_decision(self):
        """Test that mock decision is valid."""
        decision = create_mock_decision()

        assert isinstance(decision, PactDecision)
        assert decision.id.startswith("dec_test_")
        assert decision.status == DecisionStatus.ALLOW

    def test_applies_overrides(self):
        """Test that overrides are applied."""
        decision = create_mock_decision({
            "status": DecisionStatus.DENY,
            "event_id": "custom_event",
        })

        assert decision.status == DecisionStatus.DENY
        assert decision.event_id == "custom_event"

    def test_with_rule_evaluations(self):
        """Test mock decision with rule evaluations."""
        decision = create_mock_decision({
            "status": DecisionStatus.ALLOW_WITH_FLAGS,
            "rule_evaluations": [
                RuleEvaluation(
                    rule_id="rule_001",
                    rule_name="Test Rule",
                    result="FAIL",
                    code="TEST",
                    message="Test message",
                )
            ],
        })

        assert decision.status == DecisionStatus.ALLOW_WITH_FLAGS
        assert len(decision.rule_evaluations) == 1


class TestPluginTestHarness:
    """Tests for PluginTestHarness."""

    def test_test_entity_types_success(self, test_plugin):
        """Test entity types validation success."""
        harness = PluginTestHarness(test_plugin)
        result = harness.test_entity_types()

        assert result.passed is True
        assert result.name == "Entity Types"
        assert result.details["count"] == 2

    def test_test_entity_types_no_types(self, test_plugin):
        """Test entity types validation with no types defined."""
        # Create an inline plugin class with no entity types
        from arka_sdk import BasePactPlugin, PluginManifest, PactEntityType, PactRule

        class EmptyPlugin(BasePactPlugin):
            @property
            def manifest(self) -> PluginManifest:
                return PluginManifest(
                    id="empty-plugin",
                    name="Empty Plugin",
                    version="1.0.0",
                    author="Test",
                    description="Plugin with no entity types",
                    pact_core_version="0.1.0",
                )

            @property
            def _entity_types(self) -> list[PactEntityType]:
                return []

            @property
            def _default_rules(self) -> list[PactRule]:
                return []

        plugin = EmptyPlugin()
        harness = PluginTestHarness(plugin)
        result = harness.test_entity_types()

        assert result.passed is False
        assert "No entity types defined" in result.error

    def test_test_default_rules_success(self, test_plugin):
        """Test default rules validation success."""
        harness = PluginTestHarness(test_plugin)
        result = harness.test_default_rules()

        assert result.passed is True
        assert result.name == "Default Rules"
        assert result.details["count"] == 1

    def test_test_event_mapping_success(self, test_plugin):
        """Test event mapping validation success."""
        harness = PluginTestHarness(test_plugin)
        domain_event = DomainEvent(
            type="TEST_CREATED",
            payload={"name": "Test"},
        )
        result = harness.test_event_mapping(domain_event)

        assert result.passed is True
        assert result.details["input"] == "TEST_CREATED"

    def test_test_validation_success(self, test_plugin):
        """Test domain validation success."""
        harness = PluginTestHarness(test_plugin)
        result = harness.test_validation(
            entity_type="TestEntity",
            valid_data={"name": "Test", "amount": 1000},
            invalid_data={"name": "Test"},  # Missing amount
        )

        assert result.passed is True
        assert result.details["valid_data_passed"] is True
        assert result.details["invalid_data_rejected"] is True

    def test_test_validation_valid_rejected(self, test_plugin):
        """Test when valid data is incorrectly rejected."""
        harness = PluginTestHarness(test_plugin)
        # Passing data missing required fields as "valid" should fail
        result = harness.test_validation(
            entity_type="TestEntity",
            valid_data={},  # Missing required fields
            invalid_data={},
        )

        assert result.passed is False
        assert "Valid data was rejected" in result.error

    def test_test_validation_invalid_accepted(self, test_plugin):
        """Test when invalid data is incorrectly accepted."""
        harness = PluginTestHarness(test_plugin)
        # Both valid and invalid have all required fields
        result = harness.test_validation(
            entity_type="TestEntity",
            valid_data={"name": "Test", "amount": 1000},
            invalid_data={"name": "Test", "amount": 2000},  # Actually valid
        )

        assert result.passed is False
        assert "Invalid data was accepted" in result.error

    def test_run_all(self, test_plugin):
        """Test running all tests."""
        harness = PluginTestHarness(test_plugin)
        suite_result = harness.run_all()

        assert isinstance(suite_result, TestSuiteResult)
        assert suite_result.plugin == "test-plugin"
        assert suite_result.total_tests == 2  # entity_types + default_rules
        assert suite_result.passed == 2
        assert suite_result.failed == 0

    def test_add_custom_result(self, test_plugin):
        """Test adding custom test result."""
        harness = PluginTestHarness(test_plugin)
        harness.add_result(TestResult(
            name="Custom Test",
            passed=True,
            duration_ms=10,
        ))

        results = harness.get_results()
        assert len(results) == 1
        assert results[0].name == "Custom Test"

    def test_result_duration_tracking(self, test_plugin):
        """Test that duration is tracked."""
        harness = PluginTestHarness(test_plugin)
        result = harness.test_entity_types()

        assert result.duration_ms >= 0


class TestRunSimulation:
    """Tests for run_simulation utility."""

    @pytest.mark.asyncio
    async def test_basic_simulation(self, test_plugin):
        """Test basic simulation run."""
        events = [
            DomainEvent(type="TEST_CREATED", payload={"amount": 5000}),
            DomainEvent(type="TEST_CREATED", payload={"amount": 15000}),
        ]

        async def mock_evaluate(event: PactEvent, rules: list[PactRule]) -> PactDecision:
            # Simple evaluation: flag high amounts
            status = DecisionStatus.ALLOW
            evaluations = []

            for rule in rules:
                amount = event.payload.get("amount", 0)
                if amount > 10000:
                    status = DecisionStatus.ALLOW_WITH_FLAGS
                    evaluations.append(RuleEvaluation(
                        rule_id=rule.id,
                        rule_name=rule.name,
                        result="FAIL",
                        code="HIGH_AMOUNT",
                    ))
                else:
                    evaluations.append(RuleEvaluation(
                        rule_id=rule.id,
                        rule_name=rule.name,
                        result="PASS",
                    ))

            return PactDecision(
                id=f"dec_{event.id}",
                event_id=event.id,
                status=status,
                rule_evaluations=evaluations,
                created_at=datetime.utcnow(),
            )

        results = await run_simulation(test_plugin, events, mock_evaluate)

        assert results["total_events"] == 2
        assert results["decisions"]["allow"] == 1
        assert results["decisions"]["allow_with_flags"] == 1
        assert results["duration_ms"] >= 0

    @pytest.mark.asyncio
    async def test_simulation_tracks_triggered_rules(self, test_plugin):
        """Test that simulation tracks triggered rules."""
        events = [
            DomainEvent(type="TEST_CREATED", payload={"amount": 20000}),
            DomainEvent(type="TEST_CREATED", payload={"amount": 30000}),
        ]

        async def mock_evaluate(event: PactEvent, rules: list[PactRule]) -> PactDecision:
            return PactDecision(
                id=f"dec_{event.id}",
                event_id=event.id,
                status=DecisionStatus.ALLOW_WITH_FLAGS,
                rule_evaluations=[
                    RuleEvaluation(
                        rule_id="test-rule-001",
                        rule_name="Test Rule",
                        result="FAIL",
                    )
                ],
                created_at=datetime.utcnow(),
            )

        results = await run_simulation(test_plugin, events, mock_evaluate)

        assert "test-rule-001" in results["triggered_rules"]
        assert results["triggered_rules"]["test-rule-001"] == 2

    @pytest.mark.asyncio
    async def test_simulation_with_deny_decisions(self, test_plugin):
        """Test simulation with DENY decisions."""
        events = [
            DomainEvent(type="TEST_CREATED", payload={"blocked": True}),
        ]

        async def mock_evaluate(event: PactEvent, rules: list[PactRule]) -> PactDecision:
            return PactDecision(
                id=f"dec_{event.id}",
                event_id=event.id,
                status=DecisionStatus.DENY,
                rule_evaluations=[],
                created_at=datetime.utcnow(),
            )

        results = await run_simulation(test_plugin, events, mock_evaluate)

        assert results["decisions"]["deny"] == 1


class TestTestResult:
    """Tests for TestResult model."""

    def test_passed_result(self):
        """Test creating passed result."""
        result = TestResult(
            name="Test Name",
            passed=True,
            duration_ms=50,
            details={"key": "value"},
        )

        assert result.passed is True
        assert result.error is None
        assert result.details["key"] == "value"

    def test_failed_result(self):
        """Test creating failed result."""
        result = TestResult(
            name="Test Name",
            passed=False,
            duration_ms=100,
            error="Something went wrong",
        )

        assert result.passed is False
        assert result.error == "Something went wrong"


class TestTestSuiteResult:
    """Tests for TestSuiteResult model."""

    def test_suite_result(self):
        """Test creating suite result."""
        results = [
            TestResult(name="Test 1", passed=True, duration_ms=10),
            TestResult(name="Test 2", passed=True, duration_ms=20),
            TestResult(name="Test 3", passed=False, duration_ms=30, error="Failed"),
        ]

        suite = TestSuiteResult(
            plugin="test-plugin",
            total_tests=3,
            passed=2,
            failed=1,
            duration_ms=60,
            results=results,
        )

        assert suite.plugin == "test-plugin"
        assert suite.total_tests == 3
        assert suite.passed == 2
        assert suite.failed == 1
        assert len(suite.results) == 3
