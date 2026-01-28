"""
Tests for PACT Client.
"""

import pytest
from datetime import datetime
from unittest.mock import AsyncMock, MagicMock, patch
import httpx

from arka_sdk import (
    PactClient,
    PactEvent,
    PactEntity,
    PactDecision,
    PactRule,
    ValidationResult,
    DecisionStatus,
    Severity,
    PactConsequence,
    Decision,
)
from arka_sdk.types import CompareCondition


class TestClientInitialization:
    """Tests for PactClient initialization."""

    def test_basic_initialization(self):
        """Test basic client initialization."""
        client = PactClient("http://localhost:8080")

        assert client.base_url == "http://localhost:8080"
        assert client.api_key is None
        assert client.timeout == 30.0

    def test_initialization_with_trailing_slash(self):
        """Test that trailing slash is stripped from base URL."""
        client = PactClient("http://localhost:8080/")

        assert client.base_url == "http://localhost:8080"

    def test_initialization_with_api_key(self):
        """Test initialization with API key."""
        client = PactClient(
            "http://localhost:8080",
            api_key="test-api-key",
        )

        assert client.api_key == "test-api-key"

    def test_initialization_with_custom_timeout(self):
        """Test initialization with custom timeout."""
        client = PactClient(
            "http://localhost:8080",
            timeout=60.0,
        )

        assert client.timeout == 60.0

    def test_initialization_with_custom_headers(self):
        """Test initialization with custom headers."""
        headers = {"X-Custom-Header": "custom-value"}
        client = PactClient(
            "http://localhost:8080",
            headers=headers,
        )

        assert client._headers == headers

    def test_build_headers_without_api_key(self):
        """Test header building without API key."""
        client = PactClient("http://localhost:8080")
        headers = client._build_headers()

        assert headers["Content-Type"] == "application/json"
        assert headers["Accept"] == "application/json"
        assert "Authorization" not in headers

    def test_build_headers_with_api_key(self):
        """Test header building with API key."""
        client = PactClient(
            "http://localhost:8080",
            api_key="test-key",
        )
        headers = client._build_headers()

        assert headers["Authorization"] == "Bearer test-key"

    def test_client_not_initialized_error(self):
        """Test error when accessing client without context manager."""
        client = PactClient("http://localhost:8080")

        with pytest.raises(RuntimeError, match="Client not initialized"):
            _ = client.client


class TestClientContextManager:
    """Tests for async context manager."""

    @pytest.mark.asyncio
    async def test_context_manager_enters_and_exits(self):
        """Test that context manager properly manages client lifecycle."""
        client = PactClient("http://localhost:8080")

        async with client as c:
            assert c._client is not None
            assert isinstance(c._client, httpx.AsyncClient)

        assert client._client is None


class TestEventOperations:
    """Tests for event operations."""

    @pytest.mark.asyncio
    async def test_submit_event(self, sample_pact_event):
        """Test submitting an event."""
        mock_response = {
            "id": "dec_123",
            "event_id": sample_pact_event.id,
            "status": "ALLOW",
            "rule_evaluations": [],
            "created_at": datetime.utcnow().isoformat(),
        }

        with patch.object(httpx.AsyncClient, "post") as mock_post:
            mock_post.return_value = MagicMock(
                status_code=200,
                json=MagicMock(return_value=mock_response),
                raise_for_status=MagicMock(),
            )

            client = PactClient("http://localhost:8080")
            client._client = httpx.AsyncClient()

            decision = await client.submit_event(sample_pact_event)

            assert decision.event_id == sample_pact_event.id
            assert decision.status == DecisionStatus.ALLOW

    @pytest.mark.asyncio
    async def test_get_event(self):
        """Test getting an event by ID."""
        now = datetime.utcnow()
        mock_response = {
            "id": "evt_123",
            "source": "test-plugin",
            "type": "TEST_EVENT",
            "entity_id": "entity_123",
            "payload": {},
            "occurred_at": now.isoformat(),
            "received_at": now.isoformat(),
        }

        with patch.object(httpx.AsyncClient, "get") as mock_get:
            mock_get.return_value = MagicMock(
                status_code=200,
                json=MagicMock(return_value=mock_response),
                raise_for_status=MagicMock(),
            )

            client = PactClient("http://localhost:8080")
            client._client = httpx.AsyncClient()

            event = await client.get_event("evt_123")

            assert event.id == "evt_123"
            assert event.source == "test-plugin"

    @pytest.mark.asyncio
    async def test_list_events(self):
        """Test listing events."""
        now = datetime.utcnow()
        mock_response = {
            "events": [
                {
                    "id": "evt_1",
                    "source": "test",
                    "type": "TEST",
                    "occurred_at": now.isoformat(),
                    "received_at": now.isoformat(),
                },
                {
                    "id": "evt_2",
                    "source": "test",
                    "type": "TEST",
                    "occurred_at": now.isoformat(),
                    "received_at": now.isoformat(),
                },
            ]
        }

        with patch.object(httpx.AsyncClient, "get") as mock_get:
            mock_get.return_value = MagicMock(
                status_code=200,
                json=MagicMock(return_value=mock_response),
                raise_for_status=MagicMock(),
            )

            client = PactClient("http://localhost:8080")
            client._client = httpx.AsyncClient()

            events = await client.list_events(limit=10)

            assert len(events) == 2
            assert events[0].id == "evt_1"


class TestEntityOperations:
    """Tests for entity operations."""

    @pytest.mark.asyncio
    async def test_create_entity(self, sample_entity):
        """Test creating an entity."""
        mock_response = sample_entity.model_dump(mode="json")

        with patch.object(httpx.AsyncClient, "post") as mock_post:
            mock_post.return_value = MagicMock(
                status_code=201,
                json=MagicMock(return_value=mock_response),
                raise_for_status=MagicMock(),
            )

            client = PactClient("http://localhost:8080")
            client._client = httpx.AsyncClient()

            entity = await client.create_entity(sample_entity)

            assert entity.id == sample_entity.id
            assert entity.type == sample_entity.type

    @pytest.mark.asyncio
    async def test_get_entity(self):
        """Test getting an entity."""
        now = datetime.utcnow()
        mock_response = {
            "id": "ent_123",
            "type": "TestEntity",
            "data": {"name": "Test"},
            "created_at": now.isoformat(),
            "updated_at": now.isoformat(),
        }

        with patch.object(httpx.AsyncClient, "get") as mock_get:
            mock_get.return_value = MagicMock(
                status_code=200,
                json=MagicMock(return_value=mock_response),
                raise_for_status=MagicMock(),
            )

            client = PactClient("http://localhost:8080")
            client._client = httpx.AsyncClient()

            entity = await client.get_entity("ent_123")

            assert entity.id == "ent_123"
            assert entity.data["name"] == "Test"

    @pytest.mark.asyncio
    async def test_update_entity(self):
        """Test updating an entity."""
        now = datetime.utcnow()
        mock_response = {
            "id": "ent_123",
            "type": "TestEntity",
            "data": {"name": "Updated"},
            "created_at": now.isoformat(),
            "updated_at": now.isoformat(),
        }

        with patch.object(httpx.AsyncClient, "patch") as mock_patch:
            mock_patch.return_value = MagicMock(
                status_code=200,
                json=MagicMock(return_value=mock_response),
                raise_for_status=MagicMock(),
            )

            client = PactClient("http://localhost:8080")
            client._client = httpx.AsyncClient()

            entity = await client.update_entity("ent_123", {"name": "Updated"})

            assert entity.data["name"] == "Updated"

    @pytest.mark.asyncio
    async def test_delete_entity(self):
        """Test deleting an entity."""
        with patch.object(httpx.AsyncClient, "delete") as mock_delete:
            mock_delete.return_value = MagicMock(
                status_code=204,
                raise_for_status=MagicMock(),
            )

            client = PactClient("http://localhost:8080")
            client._client = httpx.AsyncClient()

            # Should not raise
            await client.delete_entity("ent_123")

    @pytest.mark.asyncio
    async def test_list_entities(self):
        """Test listing entities."""
        now = datetime.utcnow()
        mock_response = {
            "entities": [
                {
                    "id": "ent_1",
                    "type": "TestEntity",
                    "data": {},
                    "created_at": now.isoformat(),
                    "updated_at": now.isoformat(),
                },
            ]
        }

        with patch.object(httpx.AsyncClient, "get") as mock_get:
            mock_get.return_value = MagicMock(
                status_code=200,
                json=MagicMock(return_value=mock_response),
                raise_for_status=MagicMock(),
            )

            client = PactClient("http://localhost:8080")
            client._client = httpx.AsyncClient()

            entities = await client.list_entities(entity_type="TestEntity")

            assert len(entities) == 1


class TestRuleOperations:
    """Tests for rule operations."""

    @pytest.mark.asyncio
    async def test_create_rule(self, sample_rule):
        """Test creating a rule."""
        mock_response = sample_rule.model_dump(mode="json")

        with patch.object(httpx.AsyncClient, "post") as mock_post:
            mock_post.return_value = MagicMock(
                status_code=201,
                json=MagicMock(return_value=mock_response),
                raise_for_status=MagicMock(),
            )

            client = PactClient("http://localhost:8080")
            client._client = httpx.AsyncClient()

            rule = await client.create_rule(sample_rule)

            assert rule.id == sample_rule.id
            assert rule.name == sample_rule.name

    @pytest.mark.asyncio
    async def test_get_rule(self):
        """Test getting a rule."""
        mock_response = {
            "id": "rule_123",
            "name": "Test Rule",
            "condition": {"type": "compare", "field": "x", "operator": "eq", "value": 1},
            "consequence": {"decision": "FLAG", "code": "TEST", "message": "Test"},
        }

        with patch.object(httpx.AsyncClient, "get") as mock_get:
            mock_get.return_value = MagicMock(
                status_code=200,
                json=MagicMock(return_value=mock_response),
                raise_for_status=MagicMock(),
            )

            client = PactClient("http://localhost:8080")
            client._client = httpx.AsyncClient()

            rule = await client.get_rule("rule_123")

            assert rule.id == "rule_123"
            assert rule.name == "Test Rule"

    @pytest.mark.asyncio
    async def test_list_rules(self):
        """Test listing rules."""
        mock_response = {
            "rules": [
                {
                    "id": "rule_1",
                    "name": "Rule 1",
                    "condition": {"type": "compare", "field": "x", "operator": "eq", "value": 1},
                    "consequence": {"decision": "FLAG", "code": "R1", "message": "Rule 1"},
                },
            ]
        }

        with patch.object(httpx.AsyncClient, "get") as mock_get:
            mock_get.return_value = MagicMock(
                status_code=200,
                json=MagicMock(return_value=mock_response),
                raise_for_status=MagicMock(),
            )

            client = PactClient("http://localhost:8080")
            client._client = httpx.AsyncClient()

            rules = await client.list_rules(jurisdiction="US")

            assert len(rules) == 1


class TestDecisionOperations:
    """Tests for decision operations."""

    @pytest.mark.asyncio
    async def test_get_decision(self):
        """Test getting a decision."""
        mock_response = {
            "id": "dec_123",
            "event_id": "evt_123",
            "status": "ALLOW",
            "rule_evaluations": [],
            "created_at": datetime.utcnow().isoformat(),
        }

        with patch.object(httpx.AsyncClient, "get") as mock_get:
            mock_get.return_value = MagicMock(
                status_code=200,
                json=MagicMock(return_value=mock_response),
                raise_for_status=MagicMock(),
            )

            client = PactClient("http://localhost:8080")
            client._client = httpx.AsyncClient()

            decision = await client.get_decision("dec_123")

            assert decision.id == "dec_123"
            assert decision.status == DecisionStatus.ALLOW

    @pytest.mark.asyncio
    async def test_list_decisions(self):
        """Test listing decisions."""
        mock_response = {
            "decisions": [
                {
                    "id": "dec_1",
                    "event_id": "evt_1",
                    "status": "ALLOW",
                    "rule_evaluations": [],
                    "created_at": datetime.utcnow().isoformat(),
                },
            ]
        }

        with patch.object(httpx.AsyncClient, "get") as mock_get:
            mock_get.return_value = MagicMock(
                status_code=200,
                json=MagicMock(return_value=mock_response),
                raise_for_status=MagicMock(),
            )

            client = PactClient("http://localhost:8080")
            client._client = httpx.AsyncClient()

            decisions = await client.list_decisions(status="ALLOW")

            assert len(decisions) == 1


class TestValidationOperations:
    """Tests for validation operations."""

    @pytest.mark.asyncio
    async def test_validate_success(self):
        """Test successful validation."""
        mock_response = {"valid": True, "errors": [], "warnings": []}

        with patch.object(httpx.AsyncClient, "post") as mock_post:
            mock_post.return_value = MagicMock(
                status_code=200,
                json=MagicMock(return_value=mock_response),
                raise_for_status=MagicMock(),
            )

            client = PactClient("http://localhost:8080")
            client._client = httpx.AsyncClient()

            result = await client.validate("TestEntity", {"name": "Test"})

            assert result.valid is True

    @pytest.mark.asyncio
    async def test_validate_failure(self):
        """Test validation failure."""
        mock_response = {
            "valid": False,
            "errors": [
                {"field": "name", "message": "Name required", "code": "REQUIRED"},
            ],
            "warnings": [],
        }

        with patch.object(httpx.AsyncClient, "post") as mock_post:
            mock_post.return_value = MagicMock(
                status_code=200,
                json=MagicMock(return_value=mock_response),
                raise_for_status=MagicMock(),
            )

            client = PactClient("http://localhost:8080")
            client._client = httpx.AsyncClient()

            result = await client.validate("TestEntity", {})

            assert result.valid is False
            assert len(result.errors) == 1


class TestHealthOperations:
    """Tests for health operations."""

    @pytest.mark.asyncio
    async def test_health_check(self):
        """Test health check."""
        mock_response = {"status": "healthy", "version": "0.1.0"}

        with patch.object(httpx.AsyncClient, "get") as mock_get:
            mock_get.return_value = MagicMock(
                status_code=200,
                json=MagicMock(return_value=mock_response),
                raise_for_status=MagicMock(),
            )

            client = PactClient("http://localhost:8080")
            client._client = httpx.AsyncClient()

            health = await client.health()

            assert health["status"] == "healthy"

    @pytest.mark.asyncio
    async def test_ready_check_success(self):
        """Test ready check success."""
        with patch.object(httpx.AsyncClient, "get") as mock_get:
            mock_get.return_value = MagicMock(status_code=200)

            client = PactClient("http://localhost:8080")
            client._client = httpx.AsyncClient()

            ready = await client.ready()

            assert ready is True

    @pytest.mark.asyncio
    async def test_ready_check_failure(self):
        """Test ready check failure."""
        with patch.object(httpx.AsyncClient, "get") as mock_get:
            mock_get.side_effect = Exception("Connection refused")

            client = PactClient("http://localhost:8080")
            client._client = httpx.AsyncClient()

            ready = await client.ready()

            assert ready is False


class TestErrorHandling:
    """Tests for error handling."""

    @pytest.mark.asyncio
    async def test_http_error_propagation(self):
        """Test that HTTP errors are propagated."""
        with patch.object(httpx.AsyncClient, "get") as mock_get:
            mock_response = MagicMock()
            mock_response.raise_for_status.side_effect = httpx.HTTPStatusError(
                "Not Found",
                request=MagicMock(),
                response=MagicMock(status_code=404),
            )
            mock_get.return_value = mock_response

            client = PactClient("http://localhost:8080")
            client._client = httpx.AsyncClient()

            with pytest.raises(httpx.HTTPStatusError):
                await client.get_event("nonexistent")
