"""
Plugin Testing Utilities

Provides utilities for testing PACT domain plugins.
"""

from __future__ import annotations

from datetime import datetime
from typing import Any, Callable, Awaitable
from uuid import uuid4

from arka_sdk.types import (
    PactEvent,
    PactEntity,
    PactDecision,
    PactRule,
    DecisionStatus,
    RuleEvaluation,
)
from arka_sdk.base_plugin import PactDomainPlugin, DomainEvent


def create_mock_event(overrides: dict[str, Any] | None = None) -> PactEvent:
    """Creates a mock event for testing."""
    now = datetime.utcnow()
    defaults = {
        "id": f"evt_test_{uuid4().hex[:8]}",
        "source": "test",
        "type": "TEST_EVENT",
        "entity_id": "entity_123",
        "entity_type": "TestEntity",
        "payload": {},
        "occurred_at": now,
        "received_at": now,
    }
    return PactEvent(**(defaults | (overrides or {})))


def create_mock_entity(overrides: dict[str, Any] | None = None) -> PactEntity:
    """Creates a mock entity for testing."""
    now = datetime.utcnow()
    defaults = {
        "id": f"ent_test_{uuid4().hex[:8]}",
        "type": "TestEntity",
        "data": {},
        "created_at": now,
        "updated_at": now,
    }
    return PactEntity(**(defaults | (overrides or {})))


def create_mock_decision(overrides: dict[str, Any] | None = None) -> PactDecision:
    """Creates a mock decision for testing."""
    defaults = {
        "id": f"dec_test_{uuid4().hex[:8]}",
        "event_id": "evt_123",
        "status": DecisionStatus.ALLOW,
        "rule_evaluations": [],
        "created_at": datetime.utcnow(),
        "metadata": {},
    }
    return PactDecision(**(defaults | (overrides or {})))


class TestResult:
    """Test result for a single test case."""

    def __init__(
        self,
        name: str,
        passed: bool,
        duration_ms: int,
        error: str | None = None,
        details: Any = None,
    ):
        self.name = name
        self.passed = passed
        self.duration_ms = duration_ms
        self.error = error
        self.details = details


class TestSuiteResult:
    """Test suite results."""

    def __init__(
        self,
        plugin: str,
        total_tests: int,
        passed: int,
        failed: int,
        duration_ms: int,
        results: list[TestResult],
    ):
        self.plugin = plugin
        self.total_tests = total_tests
        self.passed = passed
        self.failed = failed
        self.duration_ms = duration_ms
        self.results = results


class PluginTestHarness:
    """
    Plugin test harness for comprehensive testing.

    Example:
        >>> harness = PluginTestHarness(my_plugin)
        >>> result = harness.run_all()
        >>> print(f"Passed: {result.passed}/{result.total_tests}")
    """

    def __init__(self, plugin: PactDomainPlugin):
        self.plugin = plugin
        self.results: list[TestResult] = []

    def test_entity_types(self) -> TestResult:
        """Tests entity type definitions."""
        start = datetime.utcnow()
        try:
            entity_types = self.plugin.get_entity_types()

            if len(entity_types) == 0:
                return TestResult(
                    name="Entity Types",
                    passed=False,
                    duration_ms=self._elapsed_ms(start),
                    error="No entity types defined",
                )

            for etype in entity_types:
                if not etype.name:
                    raise ValueError("Entity type missing name")
                if not etype.schema:
                    raise ValueError(f"Entity type {etype.name} missing schema")

            return TestResult(
                name="Entity Types",
                passed=True,
                duration_ms=self._elapsed_ms(start),
                details={
                    "count": len(entity_types),
                    "types": [t.name for t in entity_types],
                },
            )
        except Exception as e:
            return TestResult(
                name="Entity Types",
                passed=False,
                duration_ms=self._elapsed_ms(start),
                error=str(e),
            )

    def test_default_rules(self) -> TestResult:
        """Tests default rules."""
        start = datetime.utcnow()
        try:
            rules = self.plugin.get_default_rules()

            for rule in rules:
                if not rule.id:
                    raise ValueError("Rule missing ID")
                if not rule.name:
                    raise ValueError(f"Rule {rule.id} missing name")
                if not rule.condition:
                    raise ValueError(f"Rule {rule.id} missing condition")
                if not rule.consequence:
                    raise ValueError(f"Rule {rule.id} missing consequence")

            return TestResult(
                name="Default Rules",
                passed=True,
                duration_ms=self._elapsed_ms(start),
                details={
                    "count": len(rules),
                    "rules": [r.name for r in rules],
                },
            )
        except Exception as e:
            return TestResult(
                name="Default Rules",
                passed=False,
                duration_ms=self._elapsed_ms(start),
                error=str(e),
            )

    def test_event_mapping(self, domain_event: DomainEvent) -> TestResult:
        """Tests event mapping."""
        start = datetime.utcnow()
        try:
            canonical_event = self.plugin.map_to_canonical_event(domain_event)

            if not canonical_event.source:
                raise ValueError("Mapped event missing source")
            if not canonical_event.type:
                raise ValueError("Mapped event missing type")

            return TestResult(
                name="Event Mapping",
                passed=True,
                duration_ms=self._elapsed_ms(start),
                details={
                    "input": domain_event.type,
                    "output": canonical_event.type,
                },
            )
        except Exception as e:
            return TestResult(
                name="Event Mapping",
                passed=False,
                duration_ms=self._elapsed_ms(start),
                error=str(e),
            )

    def test_validation(
        self,
        entity_type: str,
        valid_data: dict[str, Any],
        invalid_data: dict[str, Any],
    ) -> TestResult:
        """Tests domain data validation."""
        start = datetime.utcnow()
        try:
            # Test valid data passes
            valid_result = self.plugin.validate_domain_data(entity_type, valid_data)
            if not valid_result.valid:
                errors = ", ".join(e.message for e in valid_result.errors)
                raise ValueError(f"Valid data was rejected: {errors}")

            # Test invalid data fails
            invalid_result = self.plugin.validate_domain_data(entity_type, invalid_data)
            if invalid_result.valid:
                raise ValueError("Invalid data was accepted")

            return TestResult(
                name="Validation",
                passed=True,
                duration_ms=self._elapsed_ms(start),
                details={
                    "valid_data_passed": True,
                    "invalid_data_rejected": True,
                    "errors_found": len(invalid_result.errors),
                },
            )
        except Exception as e:
            return TestResult(
                name="Validation",
                passed=False,
                duration_ms=self._elapsed_ms(start),
                error=str(e),
            )

    def run_all(self) -> TestSuiteResult:
        """Runs all tests."""
        start = datetime.utcnow()
        self.results = []

        self.results.append(self.test_entity_types())
        self.results.append(self.test_default_rules())

        passed = sum(1 for r in self.results if r.passed)
        failed = sum(1 for r in self.results if not r.passed)

        return TestSuiteResult(
            plugin=self.plugin.manifest.id,
            total_tests=len(self.results),
            passed=passed,
            failed=failed,
            duration_ms=self._elapsed_ms(start),
            results=self.results,
        )

    def add_result(self, result: TestResult) -> None:
        """Adds a custom test result."""
        self.results.append(result)

    def get_results(self) -> list[TestResult]:
        """Gets all results."""
        return self.results

    @staticmethod
    def _elapsed_ms(start: datetime) -> int:
        """Calculates elapsed time in milliseconds."""
        return int((datetime.utcnow() - start).total_seconds() * 1000)


async def run_simulation(
    plugin: PactDomainPlugin,
    events: list[DomainEvent],
    evaluate_rules: Callable[[PactEvent, list[PactRule]], Awaitable[PactDecision]],
) -> dict[str, Any]:
    """
    Runs a simulation with generated events.

    Args:
        plugin: The plugin to test
        events: Domain events to process
        evaluate_rules: Rule evaluation function

    Returns:
        Simulation results with decision counts and triggered rules
    """
    start = datetime.utcnow()
    rules = plugin.get_default_rules()
    triggered_rules: dict[str, int] = {}
    decisions = {"allow": 0, "allow_with_flags": 0, "deny": 0}

    for domain_event in events:
        pact_event = plugin.map_to_canonical_event(domain_event)
        decision = await evaluate_rules(pact_event, rules)

        # Count decisions
        if decision.status == DecisionStatus.ALLOW:
            decisions["allow"] += 1
        elif decision.status == DecisionStatus.ALLOW_WITH_FLAGS:
            decisions["allow_with_flags"] += 1
        elif decision.status == DecisionStatus.DENY:
            decisions["deny"] += 1

        # Count triggered rules
        for evaluation in decision.rule_evaluations:
            if evaluation.result == "FAIL":
                triggered_rules[evaluation.rule_id] = (
                    triggered_rules.get(evaluation.rule_id, 0) + 1
                )

    elapsed = int((datetime.utcnow() - start).total_seconds() * 1000)

    return {
        "total_events": len(events),
        "decisions": decisions,
        "triggered_rules": triggered_rules,
        "duration_ms": elapsed,
    }
