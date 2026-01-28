"""
PACT Plugin SDK for Python

SDK for building PACT Protocol domain plugins in Python.

Features:
- Plugin interfaces and base classes
- Type-safe rule and condition builders
- Event mapping utilities
- Validation helpers
- gRPC service integration

Example:
    >>> from arka_sdk import BasePactPlugin, PluginManifest
    >>>
    >>> class MyPlugin(BasePactPlugin):
    ...     @property
    ...     def manifest(self) -> PluginManifest:
    ...         return PluginManifest(
    ...             id="my-plugin",
    ...             name="My Plugin",
    ...             version="1.0.0",
    ...             author="My Company",
    ...             description="My custom PACT plugin",
    ...             entity_types=["MyEntity"],
    ...             event_types=["MY_EVENT"],
    ...             pact_core_version="0.1.0",
    ...         )
"""

__version__ = "0.1.0"
__all__ = [
    # Types
    "PluginManifest",
    "DomainEvent",
    "PactEvent",
    "PactEntity",
    "PactEntityType",
    "PactRule",
    "PactCondition",
    "PactConsequence",
    "PactDecision",
    "ValidationResult",
    "ValidationError",
    "Severity",
    "Decision",
    "DecisionStatus",
    # Base plugin
    "BasePactPlugin",
    "PactDomainPlugin",
    "PluginHooks",
    # Builders
    "RuleBuilder",
    "ConditionBuilder",
    "rule",
    "condition",
    # Registry
    "PluginRegistry",
    "get_plugin_registry",
    # Testing
    "PluginTestHarness",
    "create_mock_event",
    "create_mock_entity",
    "create_mock_decision",
    # Client
    "PactClient",
]

from arka_sdk.types import (
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
from arka_sdk.base_plugin import BasePactPlugin, PactDomainPlugin, PluginHooks
from arka_sdk.builders import RuleBuilder, ConditionBuilder, rule, condition
from arka_sdk.registry import PluginRegistry, get_plugin_registry
from arka_sdk.testing import (
    PluginTestHarness,
    create_mock_event,
    create_mock_entity,
    create_mock_decision,
)
from arka_sdk.client import PactClient
