# ARKA SDK for Python

The official Python SDK for building PACT Protocol domain plugins and integrating with PACT Core.

[![Python 3.10+](https://img.shields.io/badge/python-3.10+-blue.svg)](https://www.python.org/downloads/)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

## Overview

The ARKA SDK provides everything you need to:

- Build custom domain plugins for the PACT Protocol
- Define entity types with JSON Schema validation
- Create compliance rules using a fluent builder API
- Map domain events to the canonical PACT format
- Communicate with PACT Core via async HTTP client

## Installation

```bash
pip install arka-sdk
```

For development dependencies (testing, type checking, linting):

```bash
pip install arka-sdk[dev]
```

## Quick Start

### Creating a Domain Plugin

```python
from arka_sdk import (
    BasePactPlugin,
    PluginManifest,
    PactEntityType,
    PactRule,
    DomainEvent,
    PactEvent,
    rule,
    condition,
    Severity,
)


class LoanPlugin(BasePactPlugin):
    """Plugin for loan compliance."""

    @property
    def manifest(self) -> PluginManifest:
        return PluginManifest(
            id="pact-loans",
            name="PACT Loans",
            version="1.0.0",
            author="My Company",
            description="Loan compliance plugin",
            entity_types=["Loan", "Borrower"],
            event_types=["LOAN_CREATED", "LOAN_APPROVED", "LOAN_DISBURSED"],
            pact_core_version="0.1.0",
        )

    @property
    def _entity_types(self) -> list[PactEntityType]:
        return [
            PactEntityType(
                name="Loan",
                description="A loan application",
                schema={
                    "type": "object",
                    "properties": {
                        "amount": {"type": "number"},
                        "term_months": {"type": "integer"},
                        "interest_rate": {"type": "number"},
                        "borrower_id": {"type": "string"},
                    },
                },
                required_fields=["amount", "borrower_id"],
            ),
        ]

    @property
    def _default_rules(self) -> list[PactRule]:
        return [
            rule()
            .name("High Value Loan Check")
            .description("Flag loans over $100,000 for manual review")
            .jurisdiction("US")
            .severity("HIGH")
            .when(condition().field("payload.amount").gt(100000).build())
            .then_flag("HIGH_VALUE_LOAN", "Loan exceeds $100,000 threshold")
            .tags("lending", "threshold", "review")
            .build(),
        ]
```

### Using the PACT Client

```python
import asyncio
from arka_sdk import PactClient, PactEvent
from datetime import datetime


async def main():
    async with PactClient(
        "http://localhost:8080",
        api_key="your-api-key",
    ) as client:
        # Check service health
        health = await client.health()
        print(f"Service status: {health['status']}")

        # Submit an event for evaluation
        event = PactEvent(
            id="evt_123",
            source="my-app",
            type="LOAN_CREATED",
            entity_id="loan_456",
            entity_type="Loan",
            jurisdiction="US",
            payload={"amount": 150000, "borrower_id": "borrower_789"},
            occurred_at=datetime.utcnow(),
            received_at=datetime.utcnow(),
        )

        decision = await client.submit_event(event)
        print(f"Decision: {decision.status}")

        # List recent decisions
        decisions = await client.list_decisions(limit=10)
        for d in decisions:
            print(f"  {d.id}: {d.status}")


asyncio.run(main())
```

### Building Rules with the Fluent API

```python
from arka_sdk import rule, condition, Severity

# Simple comparison rule
simple_rule = (
    rule()
    .name("Amount Threshold")
    .when(condition().field("amount").gt(10000).build())
    .then_flag("HIGH_AMOUNT", "Amount exceeds threshold")
    .build()
)

# Complex rule with multiple conditions
complex_rule = (
    rule()
    .name("Sanctions Check")
    .description("Block transactions involving sanctioned countries")
    .jurisdiction("US")
    .severity("CRITICAL")
    .when(
        condition()
        .and_([
            condition().field("amount").gt(5000).build(),
            condition().field("counterparty.country").in_(["KP", "IR", "SY"]).build(),
        ])
        .build()
    )
    .then_deny("SANCTIONS_VIOLATION", "Transaction blocked due to sanctions")
    .tags("sanctions", "ofac", "compliance")
    .build()
)

# Rule with date range validity
from datetime import datetime

temporary_rule = (
    rule()
    .name("Holiday Promotion")
    .when(condition().field("type").eq("PROMO").build())
    .then_allow()
    .effective_from(datetime(2024, 12, 1))
    .effective_to(datetime(2024, 12, 31))
    .build()
)
```

### Condition Types

The SDK supports various condition types:

```python
from arka_sdk import condition

# Comparison operators
condition().field("amount").eq(100).build()      # Equal
condition().field("amount").ne(0).build()        # Not equal
condition().field("amount").gt(100).build()      # Greater than
condition().field("amount").gte(100).build()     # Greater than or equal
condition().field("amount").lt(100).build()      # Less than
condition().field("amount").lte(100).build()     # Less than or equal

# String operations
condition().field("name").contains("test").build()
condition().field("code").starts_with("US-").build()
condition().field("email").ends_with("@example.com").build()
condition().field("pattern").matches(r"^\d{3}-\d{4}$").build()

# Set membership
condition().field("status").in_(["active", "pending"]).build()

# Range checks
condition().field("score").between(0, 100).build()
condition().field("amount").between(100, 1000, min_inclusive=False).build()

# Field existence
condition().field("optional_field").exists().build()

# Logical operators
condition().and_([cond1, cond2]).build()
condition().or_([cond1, cond2]).build()
condition().not_(inner_condition).build()

# Custom expressions (CEL)
condition().expression("event.amount > entity.limit").build()
```

## API Reference

### PactClient

The async HTTP client for communicating with PACT Core.

| Method | Description |
|--------|-------------|
| `submit_event(event)` | Submit an event for rule evaluation |
| `get_event(event_id)` | Get an event by ID |
| `list_events(**filters)` | List events with optional filters |
| `create_entity(entity)` | Create a new entity |
| `get_entity(entity_id)` | Get an entity by ID |
| `update_entity(entity_id, data)` | Update an entity |
| `delete_entity(entity_id)` | Delete an entity |
| `list_entities(**filters)` | List entities with optional filters |
| `create_rule(rule)` | Create a new rule |
| `get_rule(rule_id)` | Get a rule by ID |
| `update_rule(rule_id, rule)` | Update a rule |
| `delete_rule(rule_id)` | Delete a rule |
| `list_rules(**filters)` | List rules with optional filters |
| `get_decision(decision_id)` | Get a decision by ID |
| `list_decisions(**filters)` | List decisions with optional filters |
| `validate(entity_type, data)` | Validate data against schema |
| `health()` | Check service health |
| `ready()` | Check if service is ready |

### BasePactPlugin

Abstract base class for domain plugins.

| Property/Method | Description |
|-----------------|-------------|
| `manifest` | Plugin manifest (abstract property) |
| `hooks` | Plugin lifecycle hooks (optional) |
| `get_entity_types()` | Returns entity type definitions |
| `get_default_rules()` | Returns default compliance rules |
| `map_to_canonical_event(event)` | Maps domain event to PACT format |
| `validate_domain_data(type, data)` | Validates domain data |
| `get_evaluation_context(event, entity)` | Returns custom evaluation context |
| `serialize_for_chain(data)` | Serializes data for blockchain |
| `create_rule(...)` | Helper to create rules with plugin metadata |

### PluginRegistry

Central registry for managing domain plugins.

| Method | Description |
|--------|-------------|
| `register(plugin)` | Register a plugin |
| `unregister(plugin_id)` | Unregister a plugin |
| `get_plugin(plugin_id)` | Get plugin by ID |
| `get_plugin_for_entity_type(type)` | Get plugin for entity type |
| `get_plugin_for_event_type(type)` | Get plugin for event type |
| `get_all_plugins()` | Get all registered plugins |
| `get_all_entity_types()` | Get all entity types |
| `get_all_default_rules()` | Get all default rules |
| `on(handler)` | Subscribe to registry events |
| `get_stats()` | Get registry statistics |

### Testing Utilities

```python
from arka_sdk import (
    PluginTestHarness,
    create_mock_event,
    create_mock_entity,
    create_mock_decision,
)

# Create mock objects for testing
event = create_mock_event({"type": "MY_EVENT", "payload": {"key": "value"}})
entity = create_mock_entity({"type": "MyEntity", "data": {"name": "Test"}})
decision = create_mock_decision({"status": "ALLOW"})

# Test plugin with harness
harness = PluginTestHarness(my_plugin)
results = harness.run_all()
print(f"Passed: {results.passed}/{results.total_tests}")

# Individual tests
harness.test_entity_types()
harness.test_default_rules()
harness.test_event_mapping(domain_event)
harness.test_validation("MyEntity", valid_data, invalid_data)
```

## Configuration Options

### Client Configuration

```python
from arka_sdk import PactClient

client = PactClient(
    base_url="http://localhost:8080",  # PACT Core URL
    api_key="your-api-key",            # Optional API key for auth
    timeout=30.0,                       # Request timeout in seconds
    headers={"X-Custom": "value"},     # Additional headers
)
```

### Plugin Configuration

Plugins can define a configuration schema in their manifest:

```python
@property
def manifest(self) -> PluginManifest:
    return PluginManifest(
        id="my-plugin",
        name="My Plugin",
        version="1.0.0",
        author="My Company",
        description="My plugin",
        pact_core_version="0.1.0",
        config_schema={
            "type": "object",
            "properties": {
                "api_endpoint": {"type": "string"},
                "max_retries": {"type": "integer", "default": 3},
            },
            "required": ["api_endpoint"],
        },
    )
```

## Type Definitions

The SDK uses Pydantic models for type-safe data handling:

| Type | Description |
|------|-------------|
| `PluginManifest` | Plugin identification and capabilities |
| `DomainEvent` | Domain-specific event before mapping |
| `PactEvent` | Canonical PACT event format |
| `PactEntity` | Entity representation |
| `PactEntityType` | Entity type definition with schema |
| `PactRule` | Compliance rule definition |
| `PactCondition` | Rule condition (union type) |
| `PactConsequence` | Rule consequence (ALLOW/DENY/FLAG) |
| `PactDecision` | Decision result from evaluation |
| `ValidationResult` | Data validation result |
| `Severity` | Rule severity enum (LOW/MEDIUM/HIGH/CRITICAL) |
| `Decision` | Consequence decision enum (ALLOW/DENY/FLAG) |
| `DecisionStatus` | Overall decision status enum |

## Running Tests

```bash
# Install dev dependencies
pip install -e ".[dev]"

# Run all tests
pytest

# Run with coverage
pytest --cov=arka_sdk --cov-report=html

# Run specific test file
pytest tests/test_client.py

# Run with verbose output
pytest -v
```

## Documentation

For complete documentation, visit:

**[https://www.arkaprotocol.com/docs](https://www.arkaprotocol.com/docs)**

## Requirements

- Python 3.10+
- pydantic >= 2.0.0
- httpx >= 0.25.0
- grpcio >= 1.60.0
- protobuf >= 4.25.0

## License

MIT License - see [LICENSE](LICENSE) for details.

## Contributing

Contributions are welcome! Please read our contributing guidelines before submitting a pull request.

## Support

- GitHub Issues: [Report bugs or request features](https://github.com/pact-engine/arka-sdk-python/issues)
- Documentation: [https://www.arkaprotocol.com/docs](https://www.arkaprotocol.com/docs)
