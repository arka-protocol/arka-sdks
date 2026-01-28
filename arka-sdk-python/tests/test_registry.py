"""
Tests for Plugin Registry.
"""

import pytest
from datetime import datetime

from arka_sdk import (
    PluginRegistry,
    get_plugin_registry,
    BasePactPlugin,
    PluginManifest,
    PactEntityType,
    PactRule,
    PluginHooks,
    PactConsequence,
    Decision,
    Severity,
)
from arka_sdk.types import CompareCondition
from arka_sdk.registry import reset_plugin_registry


class MinimalPlugin(BasePactPlugin):
    """A minimal plugin for testing."""

    def __init__(
        self,
        plugin_id: str = "minimal-plugin",
        entity_types: list[str] | None = None,
        event_types: list[str] | None = None,
        dependencies: list[str] | None = None,
    ):
        self._plugin_id = plugin_id
        self._entity_types_names = entity_types or ["MinimalEntity"]
        self._event_types_names = event_types or ["MINIMAL_EVENT"]
        self._dependencies = dependencies or []

    @property
    def manifest(self) -> PluginManifest:
        return PluginManifest(
            id=self._plugin_id,
            name=f"{self._plugin_id} Plugin",
            version="1.0.0",
            author="Test",
            description="Minimal plugin",
            entity_types=self._entity_types_names,
            event_types=self._event_types_names,
            dependencies=self._dependencies,
            pact_core_version="0.1.0",
        )

    @property
    def _entity_types(self) -> list[PactEntityType]:
        return [
            PactEntityType(
                name=name,
                description=f"{name} type",
                schema={"type": "object"},
            )
            for name in self._entity_types_names
        ]

    @property
    def _default_rules(self) -> list[PactRule]:
        return []


class PluginWithHooks(MinimalPlugin):
    """Plugin with lifecycle hooks."""

    def __init__(self, *args, **kwargs):
        super().__init__(*args, **kwargs)
        self.load_called = False
        self.unload_called = False

    @property
    def hooks(self) -> PluginHooks:
        async def on_load():
            self.load_called = True

        async def on_unload():
            self.unload_called = True

        return PluginHooks(on_load=on_load, on_unload=on_unload)


@pytest.fixture
def registry():
    """Provides a fresh registry for each test."""
    reg = PluginRegistry()
    yield reg
    reg.clear()


@pytest.fixture
def global_registry():
    """Provides the global registry, reset after each test."""
    reset_plugin_registry()
    yield get_plugin_registry()
    reset_plugin_registry()


class TestPluginRegistration:
    """Tests for plugin registration."""

    @pytest.mark.asyncio
    async def test_register_plugin(self, registry):
        """Test basic plugin registration."""
        plugin = MinimalPlugin()
        await registry.register(plugin)

        assert registry.is_registered("minimal-plugin")
        assert registry.get_status("minimal-plugin") == "active"

    @pytest.mark.asyncio
    async def test_register_duplicate_fails(self, registry):
        """Test that duplicate registration fails."""
        plugin1 = MinimalPlugin()
        plugin2 = MinimalPlugin()

        await registry.register(plugin1)

        with pytest.raises(ValueError, match="already registered"):
            await registry.register(plugin2)

    @pytest.mark.asyncio
    async def test_register_entity_type_conflict(self, registry):
        """Test that entity type conflicts are detected."""
        plugin1 = MinimalPlugin(plugin_id="plugin1", entity_types=["SharedEntity"])
        plugin2 = MinimalPlugin(plugin_id="plugin2", entity_types=["SharedEntity"])

        await registry.register(plugin1)

        with pytest.raises(ValueError, match="SharedEntity.*already registered"):
            await registry.register(plugin2)

    @pytest.mark.asyncio
    async def test_register_event_type_conflict(self, registry):
        """Test that event type conflicts are detected."""
        plugin1 = MinimalPlugin(
            plugin_id="plugin1",
            entity_types=["Entity1"],
            event_types=["SHARED_EVENT"],
        )
        plugin2 = MinimalPlugin(
            plugin_id="plugin2",
            entity_types=["Entity2"],
            event_types=["SHARED_EVENT"],
        )

        await registry.register(plugin1)

        with pytest.raises(ValueError, match="SHARED_EVENT.*already registered"):
            await registry.register(plugin2)

    @pytest.mark.asyncio
    async def test_register_missing_dependency(self, registry):
        """Test that missing dependencies are detected."""
        plugin = MinimalPlugin(
            plugin_id="dependent",
            dependencies=["nonexistent-plugin"],
        )

        with pytest.raises(ValueError, match="requires 'nonexistent-plugin'"):
            await registry.register(plugin)

    @pytest.mark.asyncio
    async def test_register_with_dependency(self, registry):
        """Test registration with satisfied dependency."""
        base = MinimalPlugin(plugin_id="base-plugin")
        dependent = MinimalPlugin(
            plugin_id="dependent-plugin",
            entity_types=["DependentEntity"],
            event_types=["DEPENDENT_EVENT"],
            dependencies=["base-plugin"],
        )

        await registry.register(base)
        await registry.register(dependent)

        assert registry.is_registered("dependent-plugin")

    @pytest.mark.asyncio
    async def test_on_load_hook_called(self, registry):
        """Test that on_load hook is called during registration."""
        plugin = PluginWithHooks(
            plugin_id="hooked-plugin",
            entity_types=["HookedEntity"],
            event_types=["HOOKED_EVENT"],
        )

        await registry.register(plugin)

        assert plugin.load_called is True


class TestPluginUnregistration:
    """Tests for plugin unregistration."""

    @pytest.mark.asyncio
    async def test_unregister_plugin(self, registry):
        """Test basic plugin unregistration."""
        plugin = MinimalPlugin()
        await registry.register(plugin)
        await registry.unregister("minimal-plugin")

        assert not registry.is_registered("minimal-plugin")

    @pytest.mark.asyncio
    async def test_unregister_nonexistent_fails(self, registry):
        """Test that unregistering nonexistent plugin fails."""
        with pytest.raises(ValueError, match="not registered"):
            await registry.unregister("nonexistent")

    @pytest.mark.asyncio
    async def test_unregister_with_dependents_fails(self, registry):
        """Test that unregistering plugin with dependents fails."""
        base = MinimalPlugin(plugin_id="base-plugin")
        dependent = MinimalPlugin(
            plugin_id="dependent-plugin",
            entity_types=["DepEntity"],
            event_types=["DEP_EVENT"],
            dependencies=["base-plugin"],
        )

        await registry.register(base)
        await registry.register(dependent)

        with pytest.raises(ValueError, match="depends on it"):
            await registry.unregister("base-plugin")

    @pytest.mark.asyncio
    async def test_on_unload_hook_called(self, registry):
        """Test that on_unload hook is called during unregistration."""
        plugin = PluginWithHooks(
            plugin_id="hooked-plugin",
            entity_types=["HookedEntity"],
            event_types=["HOOKED_EVENT"],
        )

        await registry.register(plugin)
        await registry.unregister("hooked-plugin")

        assert plugin.unload_called is True

    @pytest.mark.asyncio
    async def test_entity_type_mappings_removed(self, registry):
        """Test that entity type mappings are removed on unregister."""
        plugin = MinimalPlugin(entity_types=["MyEntity"])
        await registry.register(plugin)

        assert registry.get_plugin_for_entity_type("MyEntity") is not None

        await registry.unregister("minimal-plugin")

        assert registry.get_plugin_for_entity_type("MyEntity") is None


class TestPluginLookup:
    """Tests for plugin lookup methods."""

    @pytest.mark.asyncio
    async def test_get_plugin(self, registry):
        """Test getting plugin by ID."""
        plugin = MinimalPlugin()
        await registry.register(plugin)

        result = registry.get_plugin("minimal-plugin")

        assert result is plugin

    @pytest.mark.asyncio
    async def test_get_plugin_nonexistent(self, registry):
        """Test getting nonexistent plugin returns None."""
        result = registry.get_plugin("nonexistent")

        assert result is None

    @pytest.mark.asyncio
    async def test_get_plugin_for_entity_type(self, registry):
        """Test getting plugin by entity type."""
        plugin = MinimalPlugin(entity_types=["MyEntity"])
        await registry.register(plugin)

        result = registry.get_plugin_for_entity_type("MyEntity")

        assert result is plugin

    @pytest.mark.asyncio
    async def test_get_plugin_for_entity_type_nonexistent(self, registry):
        """Test getting plugin for unknown entity type."""
        result = registry.get_plugin_for_entity_type("UnknownEntity")

        assert result is None

    @pytest.mark.asyncio
    async def test_get_plugin_for_event_type(self, registry):
        """Test getting plugin by event type."""
        plugin = MinimalPlugin(event_types=["MY_EVENT"])
        await registry.register(plugin)

        result = registry.get_plugin_for_event_type("MY_EVENT")

        assert result is plugin

    @pytest.mark.asyncio
    async def test_get_all_plugins(self, registry):
        """Test getting all registered plugins."""
        plugin1 = MinimalPlugin(
            plugin_id="plugin1",
            entity_types=["Entity1"],
            event_types=["EVENT1"],
        )
        plugin2 = MinimalPlugin(
            plugin_id="plugin2",
            entity_types=["Entity2"],
            event_types=["EVENT2"],
        )

        await registry.register(plugin1)
        await registry.register(plugin2)

        plugins = registry.get_all_plugins()

        assert len(plugins) == 2

    @pytest.mark.asyncio
    async def test_get_all_entity_types(self, registry):
        """Test getting all entity types from all plugins."""
        plugin1 = MinimalPlugin(
            plugin_id="plugin1",
            entity_types=["Entity1", "Entity2"],
            event_types=["EVENT1"],
        )
        plugin2 = MinimalPlugin(
            plugin_id="plugin2",
            entity_types=["Entity3"],
            event_types=["EVENT2"],
        )

        await registry.register(plugin1)
        await registry.register(plugin2)

        types = registry.get_all_entity_types()

        assert len(types) == 3

    @pytest.mark.asyncio
    async def test_get_all_default_rules(self, registry, test_plugin):
        """Test getting all default rules from all plugins."""
        await registry.register(test_plugin)

        rules = registry.get_all_default_rules()

        assert len(rules) == 1
        assert rules[0].name == "High Amount Check"


class TestRegistryEvents:
    """Tests for registry event handlers."""

    @pytest.mark.asyncio
    async def test_event_handler_subscription(self, registry):
        """Test event handler subscription."""
        events = []

        def handler(event):
            events.append(event)

        unsubscribe = registry.on(handler)

        plugin = MinimalPlugin()
        await registry.register(plugin)

        assert len(events) > 0
        assert any(e["type"] == "plugin:registered" for e in events)

        unsubscribe()

    @pytest.mark.asyncio
    async def test_event_handler_unsubscription(self, registry):
        """Test event handler unsubscription."""
        events = []

        def handler(event):
            events.append(event)

        unsubscribe = registry.on(handler)
        unsubscribe()

        plugin = MinimalPlugin()
        await registry.register(plugin)

        # Handler was removed, should not receive events
        assert len(events) == 0

    @pytest.mark.asyncio
    async def test_entity_type_registered_event(self, registry):
        """Test entity_type:registered event is emitted."""
        events = []

        def handler(event):
            if event["type"] == "entity_type:registered":
                events.append(event)

        registry.on(handler)

        plugin = MinimalPlugin(entity_types=["Entity1", "Entity2"])
        await registry.register(plugin)

        assert len(events) == 2

    @pytest.mark.asyncio
    async def test_plugin_unregistered_event(self, registry):
        """Test plugin:unregistered event is emitted."""
        events = []

        def handler(event):
            if event["type"] == "plugin:unregistered":
                events.append(event)

        registry.on(handler)

        plugin = MinimalPlugin()
        await registry.register(plugin)
        await registry.unregister("minimal-plugin")

        assert len(events) == 1
        assert events[0]["plugin_id"] == "minimal-plugin"


class TestRegistryStats:
    """Tests for registry statistics."""

    @pytest.mark.asyncio
    async def test_get_stats(self, registry):
        """Test getting registry statistics."""
        plugin1 = MinimalPlugin(
            plugin_id="plugin1",
            entity_types=["Entity1"],
            event_types=["EVENT1"],
        )
        plugin2 = MinimalPlugin(
            plugin_id="plugin2",
            entity_types=["Entity2", "Entity3"],
            event_types=["EVENT2", "EVENT3"],
        )

        await registry.register(plugin1)
        await registry.register(plugin2)

        stats = registry.get_stats()

        assert stats["plugin_count"] == 2
        assert stats["entity_type_count"] == 3
        assert stats["event_type_count"] == 3
        assert "plugin1" in stats["active_plugins"]
        assert "plugin2" in stats["active_plugins"]


class TestRegistryClear:
    """Tests for registry clear functionality."""

    @pytest.mark.asyncio
    async def test_clear_registry(self, registry):
        """Test clearing all plugins."""
        plugin = MinimalPlugin()
        await registry.register(plugin)

        registry.clear()

        assert not registry.is_registered("minimal-plugin")
        stats = registry.get_stats()
        assert stats["plugin_count"] == 0


class TestGlobalRegistry:
    """Tests for global registry singleton."""

    def test_get_plugin_registry_singleton(self, global_registry):
        """Test that get_plugin_registry returns singleton."""
        registry1 = get_plugin_registry()
        registry2 = get_plugin_registry()

        assert registry1 is registry2

    def test_reset_plugin_registry(self, global_registry):
        """Test resetting global registry."""
        registry1 = get_plugin_registry()
        reset_plugin_registry()
        registry2 = get_plugin_registry()

        assert registry1 is not registry2
