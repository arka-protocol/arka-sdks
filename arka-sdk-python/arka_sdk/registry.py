"""
Plugin Registry

Central registry for managing domain plugins.
"""

from __future__ import annotations

import logging
from datetime import datetime
from typing import Any, Callable

from arka_sdk.types import PactEntityType, PactRule
from arka_sdk.base_plugin import PactDomainPlugin

logger = logging.getLogger(__name__)


class PluginRegistration:
    """Plugin registration entry."""

    def __init__(
        self,
        plugin: PactDomainPlugin,
        registered_at: datetime,
        status: str = "active",
        error: str | None = None,
    ):
        self.plugin = plugin
        self.registered_at = registered_at
        self.status = status
        self.error = error


PluginRegistryEventHandler = Callable[[dict[str, Any]], None]


class PluginRegistry:
    """
    Central registry for PACT domain plugins.

    Example:
        >>> registry = PluginRegistry()
        >>> await registry.register(my_plugin)
        >>> plugin = registry.get_plugin("my-plugin")
    """

    def __init__(self):
        self._plugins: dict[str, PluginRegistration] = {}
        self._entity_type_to_plugin: dict[str, str] = {}
        self._event_type_to_plugin: dict[str, str] = {}
        self._event_handlers: list[PluginRegistryEventHandler] = []

    async def register(
        self,
        plugin: PactDomainPlugin,
        config: dict[str, Any] | None = None,
    ) -> None:
        """Registers a domain plugin."""
        manifest = plugin.manifest

        # Check for duplicate registration
        if manifest.id in self._plugins:
            raise ValueError(f"Plugin '{manifest.id}' is already registered")

        # Check dependencies
        for dep in manifest.dependencies:
            if dep not in self._plugins:
                raise ValueError(
                    f"Plugin '{manifest.id}' requires '{dep}' which is not registered"
                )

        # Check for entity type conflicts
        for entity_type in manifest.entity_types:
            if entity_type in self._entity_type_to_plugin:
                existing = self._entity_type_to_plugin[entity_type]
                raise ValueError(
                    f"Entity type '{entity_type}' is already registered by plugin '{existing}'"
                )

        # Check for event type conflicts
        for event_type in manifest.event_types:
            if event_type in self._event_type_to_plugin:
                existing = self._event_type_to_plugin[event_type]
                raise ValueError(
                    f"Event type '{event_type}' is already registered by plugin '{existing}'"
                )

        logger.info(
            "Registering plugin",
            extra={
                "plugin_id": manifest.id,
                "version": manifest.version,
                "entity_types": manifest.entity_types,
                "event_types": manifest.event_types,
            },
        )

        try:
            # Call on_load hook
            if plugin.hooks:
                await plugin.hooks.on_load()

            # Register plugin
            registration = PluginRegistration(
                plugin=plugin,
                registered_at=datetime.utcnow(),
                status="active",
            )
            self._plugins[manifest.id] = registration

            # Map entity types to plugin
            for entity_type in manifest.entity_types:
                self._entity_type_to_plugin[entity_type] = manifest.id
                self._emit({
                    "type": "entity_type:registered",
                    "plugin_id": manifest.id,
                    "entity_type": entity_type,
                })

            # Map event types to plugin
            for event_type in manifest.event_types:
                self._event_type_to_plugin[event_type] = manifest.id

            self._emit({
                "type": "plugin:registered",
                "plugin_id": manifest.id,
                "manifest": manifest.model_dump(),
            })

            logger.info("Plugin registered successfully", extra={"plugin_id": manifest.id})

        except Exception as e:
            logger.error(
                "Failed to register plugin",
                extra={"plugin_id": manifest.id, "error": str(e)},
            )
            raise

    async def unregister(self, plugin_id: str) -> None:
        """Unregisters a plugin."""
        if plugin_id not in self._plugins:
            raise ValueError(f"Plugin '{plugin_id}' is not registered")

        registration = self._plugins[plugin_id]

        # Check if other plugins depend on this one
        for other_id, other_reg in self._plugins.items():
            if other_id != plugin_id:
                deps = other_reg.plugin.manifest.dependencies
                if plugin_id in deps:
                    raise ValueError(
                        f"Cannot unregister '{plugin_id}': plugin '{other_id}' depends on it"
                    )

        logger.info("Unregistering plugin", extra={"plugin_id": plugin_id})

        try:
            # Call on_unload hook
            if registration.plugin.hooks:
                await registration.plugin.hooks.on_unload()

            # Remove entity type mappings
            for entity_type in registration.plugin.manifest.entity_types:
                del self._entity_type_to_plugin[entity_type]

            # Remove event type mappings
            for event_type in registration.plugin.manifest.event_types:
                del self._event_type_to_plugin[event_type]

            # Remove plugin
            del self._plugins[plugin_id]

            self._emit({"type": "plugin:unregistered", "plugin_id": plugin_id})

            logger.info("Plugin unregistered successfully", extra={"plugin_id": plugin_id})

        except Exception as e:
            logger.error(
                "Failed to unregister plugin",
                extra={"plugin_id": plugin_id, "error": str(e)},
            )
            raise

    def get_plugin(self, plugin_id: str) -> PactDomainPlugin | None:
        """Gets a plugin by ID."""
        reg = self._plugins.get(plugin_id)
        return reg.plugin if reg else None

    def get_plugin_for_entity_type(self, entity_type: str) -> PactDomainPlugin | None:
        """Gets the plugin responsible for an entity type."""
        plugin_id = self._entity_type_to_plugin.get(entity_type)
        return self.get_plugin(plugin_id) if plugin_id else None

    def get_plugin_for_event_type(self, event_type: str) -> PactDomainPlugin | None:
        """Gets the plugin responsible for an event type."""
        plugin_id = self._event_type_to_plugin.get(event_type)
        return self.get_plugin(plugin_id) if plugin_id else None

    def get_all_plugins(self) -> list[PactDomainPlugin]:
        """Gets all registered plugins."""
        return [
            reg.plugin
            for reg in self._plugins.values()
            if reg.status == "active"
        ]

    def get_all_entity_types(self) -> list[PactEntityType]:
        """Gets all entity types from all plugins."""
        types: list[PactEntityType] = []
        for reg in self._plugins.values():
            if reg.status == "active":
                types.extend(reg.plugin.get_entity_types())
        return types

    def get_all_default_rules(self) -> list[PactRule]:
        """Gets all default rules from all plugins."""
        rules: list[PactRule] = []
        for reg in self._plugins.values():
            if reg.status == "active":
                rules.extend(reg.plugin.get_default_rules())
        return rules

    def is_registered(self, plugin_id: str) -> bool:
        """Checks if a plugin is registered."""
        return plugin_id in self._plugins

    def get_status(self, plugin_id: str) -> str | None:
        """Gets plugin status."""
        reg = self._plugins.get(plugin_id)
        return reg.status if reg else None

    def on(self, handler: PluginRegistryEventHandler) -> Callable[[], None]:
        """Subscribe to registry events."""
        self._event_handlers.append(handler)
        return lambda: self._event_handlers.remove(handler)

    def _emit(self, event: dict[str, Any]) -> None:
        """Emits an event to all handlers."""
        for handler in self._event_handlers:
            try:
                handler(event)
            except Exception as e:
                logger.error("Error in plugin event handler", extra={"error": str(e)})

    def get_stats(self) -> dict[str, Any]:
        """Gets registry statistics."""
        return {
            "plugin_count": len(self._plugins),
            "entity_type_count": len(self._entity_type_to_plugin),
            "event_type_count": len(self._event_type_to_plugin),
            "active_plugins": [
                plugin_id
                for plugin_id, reg in self._plugins.items()
                if reg.status == "active"
            ],
        }

    def clear(self) -> None:
        """Clears all plugins (for testing)."""
        self._plugins.clear()
        self._entity_type_to_plugin.clear()
        self._event_type_to_plugin.clear()


# Global singleton
_global_registry: PluginRegistry | None = None


def get_plugin_registry() -> PluginRegistry:
    """Gets the global plugin registry."""
    global _global_registry
    if _global_registry is None:
        _global_registry = PluginRegistry()
    return _global_registry


def reset_plugin_registry() -> None:
    """Resets the global plugin registry."""
    global _global_registry
    if _global_registry:
        _global_registry.clear()
    _global_registry = None
