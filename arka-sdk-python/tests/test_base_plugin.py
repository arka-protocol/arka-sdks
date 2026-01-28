"""
Tests for Base Plugin implementation.
"""

import pytest
from datetime import datetime
from typing import Any

from arka_sdk import (
    BasePactPlugin,
    PactDomainPlugin,
    PluginHooks,
    PluginManifest,
    PactEntityType,
    PactRule,
    DomainEvent,
    PactEvent,
    PactEntity,
    ValidationResult,
    ValidationError,
    PactConsequence,
    Decision,
    Severity,
)
from arka_sdk.types import CompareCondition


class TestPluginHooks:
    """Tests for PluginHooks."""

    @pytest.mark.asyncio
    async def test_on_load_hook(self):
        """Test on_load hook is called."""
        called = []

        async def on_load():
            called.append("loaded")

        hooks = PluginHooks(on_load=on_load)
        await hooks.on_load()

        assert "loaded" in called

    @pytest.mark.asyncio
    async def test_on_unload_hook(self):
        """Test on_unload hook is called."""
        called = []

        async def on_unload():
            called.append("unloaded")

        hooks = PluginHooks(on_unload=on_unload)
        await hooks.on_unload()

        assert "unloaded" in called

    @pytest.mark.asyncio
    async def test_before_event_process_hook(self, sample_pact_event):
        """Test before_event_process hook transforms event."""
        async def transform(event: PactEvent) -> PactEvent:
            event.metadata["processed"] = True
            return event

        hooks = PluginHooks(before_event_process=transform)
        result = await hooks.before_event_process(sample_pact_event)

        assert result.metadata["processed"] is True

    @pytest.mark.asyncio
    async def test_after_decision_hook(self, sample_pact_event, sample_decision):
        """Test after_decision hook is called."""
        decisions_logged = []

        async def log_decision(event: PactEvent, decision: Any):
            decisions_logged.append((event.id, decision.id))

        hooks = PluginHooks(after_decision=log_decision)
        await hooks.after_decision(sample_pact_event, sample_decision)

        assert len(decisions_logged) == 1

    @pytest.mark.asyncio
    async def test_on_rules_updated_hook(self, sample_rule):
        """Test on_rules_updated hook is called."""
        rules_received = []

        async def on_rules_update(rules: list[PactRule]):
            rules_received.extend(rules)

        hooks = PluginHooks(on_rules_updated=on_rules_update)
        await hooks.on_rules_updated([sample_rule])

        assert len(rules_received) == 1

    @pytest.mark.asyncio
    async def test_hooks_without_handlers(self, sample_pact_event):
        """Test that hooks work without handlers."""
        hooks = PluginHooks()

        # Should not raise
        await hooks.on_load()
        await hooks.on_unload()
        result = await hooks.before_event_process(sample_pact_event)
        await hooks.after_decision(sample_pact_event, None)
        await hooks.on_rules_updated([])

        # Event should be unchanged
        assert result == sample_pact_event


class TestBasePactPlugin:
    """Tests for BasePactPlugin."""

    def test_plugin_manifest(self, test_plugin):
        """Test plugin manifest is correctly defined."""
        manifest = test_plugin.manifest

        assert manifest.id == "test-plugin"
        assert manifest.name == "Test Plugin"
        assert manifest.version == "1.0.0"
        assert "TestEntity" in manifest.entity_types
        assert "TEST_CREATED" in manifest.event_types

    def test_get_entity_types(self, test_plugin):
        """Test getting entity types."""
        types = test_plugin.get_entity_types()

        assert len(types) == 2
        assert types[0].name == "TestEntity"
        assert "name" in types[0].required_fields

    def test_get_default_rules(self, test_plugin):
        """Test getting default rules."""
        rules = test_plugin.get_default_rules()

        assert len(rules) == 1
        assert rules[0].name == "High Amount Check"
        assert rules[0].jurisdiction == "US"

    def test_map_to_canonical_event(self, test_plugin, sample_domain_event):
        """Test mapping domain event to canonical format."""
        canonical = test_plugin.map_to_canonical_event(sample_domain_event)

        assert canonical.source == "test-plugin"
        assert canonical.type == sample_domain_event.type
        assert canonical.entity_id == sample_domain_event.entity_id
        assert canonical.jurisdiction == sample_domain_event.jurisdiction
        assert canonical.payload == sample_domain_event.payload
        assert canonical.id.startswith("evt_")

    def test_event_mapping_infers_entity_type(self, test_plugin):
        """Test that event mapping infers entity type from event type."""
        event = DomainEvent(type="LOAN_CREATED", payload={})
        canonical = test_plugin.map_to_canonical_event(event)

        assert canonical.entity_type == "Loan"

    def test_event_mapping_uses_provided_occurred_at(self, test_plugin):
        """Test that provided occurred_at is used."""
        occurred = datetime(2024, 1, 15, 12, 0, 0)
        event = DomainEvent(type="TEST_EVENT", payload={}, occurred_at=occurred)
        canonical = test_plugin.map_to_canonical_event(event)

        assert canonical.occurred_at == occurred

    def test_validate_domain_data_valid(self, test_plugin):
        """Test validation with valid data."""
        valid_data = {"name": "Test Entity", "amount": 5000}
        result = test_plugin.validate_domain_data("TestEntity", valid_data)

        assert result.valid is True
        assert len(result.errors) == 0

    def test_validate_domain_data_missing_required(self, test_plugin):
        """Test validation with missing required fields."""
        invalid_data = {"name": "Test Entity"}  # Missing 'amount'
        result = test_plugin.validate_domain_data("TestEntity", invalid_data)

        assert result.valid is False
        assert len(result.errors) == 1
        assert result.errors[0].field == "amount"

    def test_validate_domain_data_unknown_type(self, test_plugin):
        """Test validation with unknown entity type."""
        result = test_plugin.validate_domain_data("UnknownType", {})

        assert result.valid is False
        assert result.errors[0].code == "UNKNOWN_ENTITY_TYPE"

    def test_validate_domain_data_null_required(self, test_plugin):
        """Test validation with null required field."""
        invalid_data = {"name": None, "amount": 5000}
        result = test_plugin.validate_domain_data("TestEntity", invalid_data)

        assert result.valid is False

    def test_get_evaluation_context_default(self, test_plugin, sample_pact_event):
        """Test default evaluation context is empty."""
        context = test_plugin.get_evaluation_context(sample_pact_event)

        assert context == {}

    def test_serialize_for_chain(self, test_plugin):
        """Test serialization for blockchain."""
        data = {"name": "Test", "amount": 1000, "active": True}
        serialized = test_plugin.serialize_for_chain(data)

        # Should be deterministic JSON
        assert isinstance(serialized, bytes)
        assert b'"active":true' in serialized
        assert b'"amount":1000' in serialized

    def test_serialize_for_chain_sorting(self, test_plugin):
        """Test that serialization sorts keys."""
        data1 = {"z": 1, "a": 2, "m": 3}
        data2 = {"a": 2, "m": 3, "z": 1}

        assert test_plugin.serialize_for_chain(data1) == test_plugin.serialize_for_chain(data2)

    def test_deserialize_from_chain(self, test_plugin):
        """Test deserialization from blockchain."""
        original = {"name": "Test", "amount": 1000}
        serialized = test_plugin.serialize_for_chain(original)
        restored = test_plugin.deserialize_from_chain(serialized)

        assert restored == original

    def test_create_rule_id(self, test_plugin):
        """Test rule ID generation."""
        rule_id = test_plugin.create_rule_id()

        assert rule_id.startswith("rule_")
        assert len(rule_id) == 17  # "rule_" + 12 hex chars

    def test_create_rule_helper(self, test_plugin):
        """Test create_rule helper method."""
        condition = CompareCondition(field="amount", operator="gt", value=1000)
        consequence = PactConsequence(
            decision=Decision.FLAG,
            code="HIGH",
            message="High amount",
        )

        created_rule = test_plugin.create_rule(
            name="Helper Rule",
            condition=condition,
            consequence=consequence,
            description="Created with helper",
            jurisdiction="US",
            severity="HIGH",
            tags=["custom"],
        )

        assert created_rule.name == "Helper Rule"
        assert created_rule.description == "Created with helper"
        assert created_rule.severity == Severity.HIGH
        assert "custom" in created_rule.tags
        assert "test-plugin" in created_rule.tags  # Plugin ID added
        assert created_rule.metadata["plugin_id"] == "test-plugin"
        assert created_rule.metadata["plugin_version"] == "1.0.0"

    def test_hooks_property_default_none(self, test_plugin):
        """Test that default hooks property returns None."""
        assert test_plugin.hooks is None


class TestPluginWithCustomHooks:
    """Tests for plugin with custom hooks."""

    def test_plugin_with_hooks(self):
        """Test plugin implementation with hooks."""
        events_received = []

        async def track_event(event: PactEvent) -> PactEvent:
            events_received.append(event.id)
            return event

        class HookedPlugin(BasePactPlugin):
            def __init__(self):
                self._hooks = PluginHooks(before_event_process=track_event)

            @property
            def manifest(self) -> PluginManifest:
                return PluginManifest(
                    id="hooked-plugin",
                    name="Hooked Plugin",
                    version="1.0.0",
                    author="Test",
                    description="Plugin with hooks",
                    pact_core_version="0.1.0",
                )

            @property
            def hooks(self) -> PluginHooks:
                return self._hooks

            @property
            def _entity_types(self) -> list[PactEntityType]:
                return []

            @property
            def _default_rules(self) -> list[PactRule]:
                return []

        plugin = HookedPlugin()
        assert plugin.hooks is not None


class TestCustomPluginImplementation:
    """Tests for custom plugin implementation."""

    def test_custom_entity_type_inference(self):
        """Test custom entity type inference logic."""

        class CustomPlugin(BasePactPlugin):
            @property
            def manifest(self) -> PluginManifest:
                return PluginManifest(
                    id="custom-plugin",
                    name="Custom Plugin",
                    version="1.0.0",
                    author="Test",
                    description="Custom inference",
                    pact_core_version="0.1.0",
                )

            @property
            def _entity_types(self) -> list[PactEntityType]:
                return []

            @property
            def _default_rules(self) -> list[PactRule]:
                return []

            def _infer_entity_type(self, domain_event: DomainEvent) -> str | None:
                # Custom logic: always return "CustomEntity"
                return "CustomEntity"

        plugin = CustomPlugin()
        event = DomainEvent(type="ANY_EVENT", payload={})
        canonical = plugin.map_to_canonical_event(event)

        assert canonical.entity_type == "CustomEntity"

    def test_custom_evaluation_context(self):
        """Test custom evaluation context."""

        class ContextPlugin(BasePactPlugin):
            @property
            def manifest(self) -> PluginManifest:
                return PluginManifest(
                    id="context-plugin",
                    name="Context Plugin",
                    version="1.0.0",
                    author="Test",
                    description="Custom context",
                    pact_core_version="0.1.0",
                )

            @property
            def _entity_types(self) -> list[PactEntityType]:
                return []

            @property
            def _default_rules(self) -> list[PactRule]:
                return []

            def get_evaluation_context(
                self, event: PactEvent, entity: PactEntity | None = None
            ) -> dict[str, Any]:
                return {
                    "custom_field": "custom_value",
                    "event_type": event.type,
                }

        plugin = ContextPlugin()
        now = datetime.utcnow()
        event = PactEvent(
            id="evt_123",
            source="test",
            type="TEST",
            occurred_at=now,
            received_at=now,
        )

        context = plugin.get_evaluation_context(event)

        assert context["custom_field"] == "custom_value"
        assert context["event_type"] == "TEST"
