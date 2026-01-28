"""
PACT Client

HTTP/gRPC client for communicating with PACT Core.
"""

from __future__ import annotations

import json
import logging
from typing import Any

import httpx

from arka_sdk.types import (
    PactEvent,
    PactEntity,
    PactDecision,
    PactRule,
    ValidationResult,
)

logger = logging.getLogger(__name__)


class PactClient:
    """
    Client for communicating with PACT Core.

    Example:
        >>> async with PactClient("http://localhost:8080") as client:
        ...     decision = await client.evaluate_event(event)
        ...     print(decision.status)
    """

    def __init__(
        self,
        base_url: str,
        *,
        api_key: str | None = None,
        timeout: float = 30.0,
        headers: dict[str, str] | None = None,
    ):
        self.base_url = base_url.rstrip("/")
        self.api_key = api_key
        self.timeout = timeout
        self._headers = headers or {}
        self._client: httpx.AsyncClient | None = None

    async def __aenter__(self) -> PactClient:
        """Async context manager entry."""
        self._client = httpx.AsyncClient(
            base_url=self.base_url,
            timeout=self.timeout,
            headers=self._build_headers(),
        )
        return self

    async def __aexit__(self, exc_type, exc_val, exc_tb) -> None:
        """Async context manager exit."""
        if self._client:
            await self._client.aclose()
            self._client = None

    def _build_headers(self) -> dict[str, str]:
        """Builds request headers."""
        headers = {
            "Content-Type": "application/json",
            "Accept": "application/json",
            **self._headers,
        }
        if self.api_key:
            headers["Authorization"] = f"Bearer {self.api_key}"
        return headers

    @property
    def client(self) -> httpx.AsyncClient:
        """Gets the HTTP client."""
        if self._client is None:
            raise RuntimeError("Client not initialized. Use 'async with' context manager.")
        return self._client

    # =========================================================================
    # Event Operations
    # =========================================================================

    async def submit_event(self, event: PactEvent) -> PactDecision:
        """
        Submits an event for evaluation.

        Args:
            event: The PACT event to evaluate

        Returns:
            The decision result
        """
        response = await self.client.post(
            "/api/v1/events",
            json=event.model_dump(mode="json"),
        )
        response.raise_for_status()
        return PactDecision.model_validate(response.json())

    async def get_event(self, event_id: str) -> PactEvent:
        """Gets an event by ID."""
        response = await self.client.get(f"/api/v1/events/{event_id}")
        response.raise_for_status()
        return PactEvent.model_validate(response.json())

    async def list_events(
        self,
        *,
        entity_id: str | None = None,
        event_type: str | None = None,
        limit: int = 100,
        offset: int = 0,
    ) -> list[PactEvent]:
        """Lists events with optional filters."""
        params = {"limit": limit, "offset": offset}
        if entity_id:
            params["entity_id"] = entity_id
        if event_type:
            params["type"] = event_type

        response = await self.client.get("/api/v1/events", params=params)
        response.raise_for_status()
        data = response.json()
        return [PactEvent.model_validate(e) for e in data.get("events", [])]

    # =========================================================================
    # Entity Operations
    # =========================================================================

    async def create_entity(self, entity: PactEntity) -> PactEntity:
        """Creates a new entity."""
        response = await self.client.post(
            "/api/v1/entities",
            json=entity.model_dump(mode="json"),
        )
        response.raise_for_status()
        return PactEntity.model_validate(response.json())

    async def get_entity(self, entity_id: str) -> PactEntity:
        """Gets an entity by ID."""
        response = await self.client.get(f"/api/v1/entities/{entity_id}")
        response.raise_for_status()
        return PactEntity.model_validate(response.json())

    async def update_entity(self, entity_id: str, data: dict[str, Any]) -> PactEntity:
        """Updates an entity."""
        response = await self.client.patch(
            f"/api/v1/entities/{entity_id}",
            json={"data": data},
        )
        response.raise_for_status()
        return PactEntity.model_validate(response.json())

    async def delete_entity(self, entity_id: str) -> None:
        """Deletes an entity."""
        response = await self.client.delete(f"/api/v1/entities/{entity_id}")
        response.raise_for_status()

    async def list_entities(
        self,
        *,
        entity_type: str | None = None,
        jurisdiction: str | None = None,
        limit: int = 100,
        offset: int = 0,
    ) -> list[PactEntity]:
        """Lists entities with optional filters."""
        params = {"limit": limit, "offset": offset}
        if entity_type:
            params["type"] = entity_type
        if jurisdiction:
            params["jurisdiction"] = jurisdiction

        response = await self.client.get("/api/v1/entities", params=params)
        response.raise_for_status()
        data = response.json()
        return [PactEntity.model_validate(e) for e in data.get("entities", [])]

    # =========================================================================
    # Rule Operations
    # =========================================================================

    async def create_rule(self, rule: PactRule) -> PactRule:
        """Creates a new rule."""
        response = await self.client.post(
            "/api/v1/rules",
            json=rule.model_dump(mode="json"),
        )
        response.raise_for_status()
        return PactRule.model_validate(response.json())

    async def get_rule(self, rule_id: str) -> PactRule:
        """Gets a rule by ID."""
        response = await self.client.get(f"/api/v1/rules/{rule_id}")
        response.raise_for_status()
        return PactRule.model_validate(response.json())

    async def update_rule(self, rule_id: str, rule: PactRule) -> PactRule:
        """Updates a rule."""
        response = await self.client.put(
            f"/api/v1/rules/{rule_id}",
            json=rule.model_dump(mode="json"),
        )
        response.raise_for_status()
        return PactRule.model_validate(response.json())

    async def delete_rule(self, rule_id: str) -> None:
        """Deletes a rule."""
        response = await self.client.delete(f"/api/v1/rules/{rule_id}")
        response.raise_for_status()

    async def list_rules(
        self,
        *,
        jurisdiction: str | None = None,
        tags: list[str] | None = None,
        limit: int = 100,
        offset: int = 0,
    ) -> list[PactRule]:
        """Lists rules with optional filters."""
        params: dict[str, Any] = {"limit": limit, "offset": offset}
        if jurisdiction:
            params["jurisdiction"] = jurisdiction
        if tags:
            params["tags"] = ",".join(tags)

        response = await self.client.get("/api/v1/rules", params=params)
        response.raise_for_status()
        data = response.json()
        return [PactRule.model_validate(r) for r in data.get("rules", [])]

    # =========================================================================
    # Decision Operations
    # =========================================================================

    async def get_decision(self, decision_id: str) -> PactDecision:
        """Gets a decision by ID."""
        response = await self.client.get(f"/api/v1/decisions/{decision_id}")
        response.raise_for_status()
        return PactDecision.model_validate(response.json())

    async def list_decisions(
        self,
        *,
        event_id: str | None = None,
        entity_id: str | None = None,
        status: str | None = None,
        limit: int = 100,
        offset: int = 0,
    ) -> list[PactDecision]:
        """Lists decisions with optional filters."""
        params: dict[str, Any] = {"limit": limit, "offset": offset}
        if event_id:
            params["event_id"] = event_id
        if entity_id:
            params["entity_id"] = entity_id
        if status:
            params["status"] = status

        response = await self.client.get("/api/v1/decisions", params=params)
        response.raise_for_status()
        data = response.json()
        return [PactDecision.model_validate(d) for d in data.get("decisions", [])]

    # =========================================================================
    # Validation Operations
    # =========================================================================

    async def validate(
        self,
        entity_type: str,
        data: dict[str, Any],
    ) -> ValidationResult:
        """Validates data against an entity type schema."""
        response = await self.client.post(
            "/api/v1/validate",
            json={"entity_type": entity_type, "data": data},
        )
        response.raise_for_status()
        return ValidationResult.model_validate(response.json())

    # =========================================================================
    # Health Operations
    # =========================================================================

    async def health(self) -> dict[str, Any]:
        """Checks service health."""
        response = await self.client.get("/health")
        response.raise_for_status()
        return response.json()

    async def ready(self) -> bool:
        """Checks if service is ready."""
        try:
            response = await self.client.get("/ready")
            return response.status_code == 200
        except Exception:
            return False
