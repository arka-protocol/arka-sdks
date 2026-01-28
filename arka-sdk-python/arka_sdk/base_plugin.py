"""
Base Plugin Implementation

Abstract base class for PACT domain plugins.
"""

from __future__ import annotations

import json
from abc import ABC, abstractmethod
from datetime import datetime
from typing import Any, Callable, Awaitable
from uuid import uuid4

from arka_sdk.types import (
    PluginManifest,
    DomainEvent,
    PactEvent,
    PactEntity,
    PactEntityType,
    PactRule,
    ValidationResult,
    ValidationError,
)


class PluginHooks:
    """Plugin lifecycle hooks."""

    def __init__(
        self,
        on_load: Callable[[], Awaitable[None]] | None = None,
        on_unload: Callable[[], Awaitable[None]] | None = None,
        before_event_process: Callable[[PactEvent], Awaitable[PactEvent]] | None = None,
        after_decision: Callable[[PactEvent, Any], Awaitable[None]] | None = None,
        on_rules_updated: Callable[[list[PactRule]], Awaitable[None]] | None = None,
    ):
        self._on_load = on_load
        self._on_unload = on_unload
        self._before_event_process = before_event_process
        self._after_decision = after_decision
        self._on_rules_updated = on_rules_updated

    async def on_load(self) -> None:
        """Called when plugin is loaded."""
        if self._on_load:
            await self._on_load()

    async def on_unload(self) -> None:
        """Called when plugin is unloaded."""
        if self._on_unload:
            await self._on_unload()

    async def before_event_process(self, event: PactEvent) -> PactEvent:
        """Called before an event is processed."""
        if self._before_event_process:
            return await self._before_event_process(event)
        return event

    async def after_decision(self, event: PactEvent, decision: Any) -> None:
        """Called after a decision is made."""
        if self._after_decision:
            await self._after_decision(event, decision)

    async def on_rules_updated(self, rules: list[PactRule]) -> None:
        """Called when rules are updated."""
        if self._on_rules_updated:
            await self._on_rules_updated(rules)


class PactDomainPlugin(ABC):
    """Interface that all PACT domain plugins must implement."""

    @property
    @abstractmethod
    def manifest(self) -> PluginManifest:
        """Plugin identification and manifest."""
        ...

    @property
    def hooks(self) -> PluginHooks | None:
        """Plugin lifecycle hooks."""
        return None

    @abstractmethod
    def get_entity_types(self) -> list[PactEntityType]:
        """Returns entity types defined by this plugin."""
        ...

    @abstractmethod
    def get_default_rules(self) -> list[PactRule]:
        """Returns default rules for this domain."""
        ...

    @abstractmethod
    def map_to_canonical_event(self, domain_event: DomainEvent) -> PactEvent:
        """Converts a domain-specific event to canonical PACT format."""
        ...

    @abstractmethod
    def validate_domain_data(
        self, entity_type: str, data: dict[str, Any]
    ) -> ValidationResult:
        """Validates domain-specific data."""
        ...

    def get_evaluation_context(
        self, event: PactEvent, entity: PactEntity | None = None
    ) -> dict[str, Any]:
        """Returns domain-specific context for rule evaluation."""
        return {}

    def serialize_for_chain(self, data: Any) -> bytes:
        """Serializes domain data deterministically for blockchain."""
        return self._canonical_json(data).encode("utf-8")

    def deserialize_from_chain(self, data: bytes) -> Any:
        """Deserializes data from blockchain format."""
        return json.loads(data.decode("utf-8"))

    @staticmethod
    def _canonical_json(obj: Any) -> str:
        """Produces canonical JSON with sorted keys."""
        return json.dumps(obj, sort_keys=True, separators=(",", ":"))


class BasePactPlugin(PactDomainPlugin):
    """
    Abstract base class for PACT domain plugins.

    Provides common functionality that plugins can inherit and override.

    Example:
        >>> class LoanPlugin(BasePactPlugin):
        ...     @property
        ...     def manifest(self) -> PluginManifest:
        ...         return PluginManifest(
        ...             id="pact-loans",
        ...             name="PACT Loans",
        ...             version="1.0.0",
        ...             author="PACT Team",
        ...             description="Loan compliance plugin",
        ...             entity_types=["Loan", "Borrower"],
        ...             event_types=["LOAN_CREATED", "LOAN_APPROVED"],
        ...             pact_core_version="0.1.0",
        ...         )
        ...
        ...     @property
        ...     def _entity_types(self) -> list[PactEntityType]:
        ...         return [...]
        ...
        ...     @property
        ...     def _default_rules(self) -> list[PactRule]:
        ...         return [...]
    """

    @property
    @abstractmethod
    def _entity_types(self) -> list[PactEntityType]:
        """Entity types defined by this plugin."""
        ...

    @property
    @abstractmethod
    def _default_rules(self) -> list[PactRule]:
        """Default rules for this domain."""
        ...

    def get_entity_types(self) -> list[PactEntityType]:
        """Returns entity types defined by this plugin."""
        return self._entity_types

    def get_default_rules(self) -> list[PactRule]:
        """Returns default rules for this domain."""
        return self._default_rules

    def map_to_canonical_event(self, domain_event: DomainEvent) -> PactEvent:
        """
        Converts a domain-specific event to canonical PACT format.

        Override in subclass for custom mapping logic.
        """
        now = datetime.utcnow()
        return PactEvent(
            id=f"evt_{uuid4().hex[:12]}",
            source=self.manifest.id,
            type=domain_event.type,
            entity_id=domain_event.entity_id,
            entity_type=self._infer_entity_type(domain_event),
            jurisdiction=domain_event.jurisdiction,
            payload=domain_event.payload,
            occurred_at=domain_event.occurred_at or now,
            received_at=now,
            metadata=domain_event.metadata,
        )

    def _infer_entity_type(self, domain_event: DomainEvent) -> str | None:
        """
        Infers entity type from domain event.

        Override in subclass for custom logic.
        """
        # Default: try to infer from event type
        # e.g., "LOAN_CREATED" -> "Loan"
        parts = domain_event.type.split("_")
        if len(parts) >= 2:
            entity_name = parts[0]
            return entity_name.capitalize()
        return None

    def validate_domain_data(
        self, entity_type: str, data: dict[str, Any]
    ) -> ValidationResult:
        """
        Validates domain-specific data against entity type schema.

        Override in subclass for custom validation.
        """
        # Find the entity type
        entity_type_def = next(
            (t for t in self._entity_types if t.name == entity_type), None
        )

        if entity_type_def is None:
            return ValidationResult(
                valid=False,
                errors=[
                    ValidationError(
                        field="entity_type",
                        message=f"Unknown entity type: {entity_type}",
                        code="UNKNOWN_ENTITY_TYPE",
                    )
                ],
            )

        # Basic required field validation
        errors: list[ValidationError] = []
        for field in entity_type_def.required_fields:
            if field not in data or data[field] is None:
                errors.append(
                    ValidationError(
                        field=field,
                        message=f"Required field '{field}' is missing",
                        code="REQUIRED_FIELD_MISSING",
                    )
                )

        return ValidationResult(valid=len(errors) == 0, errors=errors)

    def create_rule_id(self) -> str:
        """Helper to create a rule ID."""
        return f"rule_{uuid4().hex[:12]}"

    def create_rule(
        self,
        name: str,
        condition: Any,
        consequence: Any,
        *,
        description: str = "",
        jurisdiction: str | None = None,
        severity: str = "MEDIUM",
        tags: list[str] | None = None,
        metadata: dict[str, Any] | None = None,
    ) -> PactRule:
        """Helper to create a rule with plugin defaults."""
        from arka_sdk.types import Severity

        return PactRule(
            id=self.create_rule_id(),
            name=name,
            description=description or name,
            jurisdiction=jurisdiction,
            severity=Severity(severity),
            condition=condition,
            consequence=consequence,
            tags=[*(tags or []), self.manifest.id],
            metadata={
                **(metadata or {}),
                "plugin_id": self.manifest.id,
                "plugin_version": self.manifest.version,
            },
        )
