# PACT SDK for Rust

A Rust SDK for building domain plugins and integrating with the PACT Protocol compliance engine.

## Overview

The PACT SDK provides:

- **Plugin Interfaces**: Base traits and implementations for building domain-specific compliance plugins
- **Type-Safe Builders**: Fluent APIs for creating rules and conditions
- **HTTP Client**: Async client for communicating with PACT Core
- **Plugin Registry**: Central management for domain plugins
- **Validation Utilities**: Helpers for data validation and event mapping

## Installation

Add the SDK to your `Cargo.toml`:

```toml
[dependencies]
pact-sdk = "0.1.0"
```

### Feature Flags

- `default`: Basic HTTP client functionality
- `grpc`: Enable gRPC transport support (requires `tonic` and `prost`)

```toml
# With gRPC support
pact-sdk = { version = "0.1.0", features = ["grpc"] }
```

## Quick Start

### Creating a Domain Plugin

```rust
use pact_sdk::{
    BasePlugin, DomainPlugin, PluginManifest, PactEntityType, PactRule,
    DomainEvent, PactEvent, ValidationResult, Severity, Decision, Condition, Consequence,
};
use async_trait::async_trait;
use std::collections::HashMap;

struct FinancePlugin {
    base: BasePlugin,
}

impl FinancePlugin {
    pub fn new() -> Self {
        let manifest = PluginManifest {
            id: "finance-plugin".to_string(),
            name: "Finance Domain Plugin".to_string(),
            version: "1.0.0".to_string(),
            author: "Your Company".to_string(),
            description: "Compliance rules for financial transactions".to_string(),
            entity_types: vec!["Transaction".to_string(), "Account".to_string()],
            event_types: vec!["TRANSACTION_CREATED".to_string(), "ACCOUNT_OPENED".to_string()],
            dependencies: vec![],
            pact_core_version: "0.1.0".to_string(),
            config_schema: None,
        };

        let entity_types = vec![
            PactEntityType {
                name: "Transaction".to_string(),
                description: "A financial transaction".to_string(),
                schema: HashMap::new(),
                required_fields: vec!["amount".to_string(), "currency".to_string()],
            },
        ];

        let default_rules = vec![];

        Self {
            base: BasePlugin::new(manifest, entity_types, default_rules),
        }
    }
}

#[async_trait]
impl DomainPlugin for FinancePlugin {
    fn manifest(&self) -> &PluginManifest {
        self.base.manifest()
    }

    fn get_entity_types(&self) -> &[PactEntityType] {
        self.base.get_entity_types()
    }

    fn get_default_rules(&self) -> &[PactRule] {
        self.base.get_default_rules()
    }

    fn map_to_canonical_event(&self, event: &DomainEvent) -> Result<PactEvent, Box<dyn std::error::Error>> {
        self.base.map_to_canonical_event(event)
    }

    fn validate_domain_data(&self, entity_type: &str, data: &HashMap<String, serde_json::Value>) -> ValidationResult {
        self.base.validate_domain_data(entity_type, data)
    }
}
```

### Building Rules with the Fluent API

```rust
use pact_sdk::{rule, condition, and, or, not, Severity};
use serde_json::json;

// Simple rule
let high_value_rule = rule()
    .id("fin-001")
    .name("High Value Transaction")
    .description("Flag transactions over $10,000")
    .severity(Severity::High)
    .when_field("amount", "gt", json!(10000))
    .then_flag("HIGH_VALUE", "Transaction exceeds reporting threshold")
    .tags(vec!["aml", "compliance"])
    .build()
    .unwrap();

// Complex rule with multiple conditions
let suspicious_rule = rule()
    .id("fin-002")
    .name("Suspicious Activity Detection")
    .jurisdiction("US")
    .severity(Severity::Critical)
    .when(and(vec![
        condition().field("amount").gt(json!(5000)).build().unwrap(),
        or(vec![
            condition().field("international").eq(json!(true)).build().unwrap(),
            condition().field("destination_country")
                .in_values(vec![json!("XX"), json!("YY")])
                .build()
                .unwrap(),
        ]),
        not(condition().field("verified_recipient").eq(json!(true)).build().unwrap()),
    ]))
    .then_deny("SUSPICIOUS_ACTIVITY", "Transaction blocked for review")
    .build()
    .unwrap();
```

### Using the PACT Client

```rust
use pact_sdk::{PactClient, PactEvent};
use std::collections::HashMap;
use chrono::Utc;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create client
    let client = PactClient::new("http://localhost:8080")
        .with_api_key("your-api-key")
        .with_timeout(std::time::Duration::from_secs(30));

    // Check service health
    if client.ready().await {
        println!("PACT Core is ready");
    }

    // Submit an event for evaluation
    let now = Utc::now();
    let mut payload = HashMap::new();
    payload.insert("amount".to_string(), serde_json::json!(15000));
    payload.insert("currency".to_string(), serde_json::json!("USD"));
    payload.insert("destination".to_string(), serde_json::json!("external-account"));

    let event = PactEvent {
        id: "evt_123".to_string(),
        source: "finance-plugin".to_string(),
        event_type: "TRANSACTION_CREATED".to_string(),
        entity_id: Some("txn_456".to_string()),
        entity_type: Some("Transaction".to_string()),
        jurisdiction: Some("US".to_string()),
        payload,
        occurred_at: now,
        received_at: now,
        metadata: HashMap::new(),
    };

    let decision = client.submit_event(&event).await?;

    match decision.status {
        pact_sdk::DecisionStatus::Allow => println!("Transaction allowed"),
        pact_sdk::DecisionStatus::AllowWithFlags => {
            println!("Transaction allowed with flags:");
            for eval in &decision.rule_evaluations {
                if eval.result == pact_sdk::EvaluationResult::Fail {
                    println!("  - {}: {}", eval.rule_name, eval.message.as_deref().unwrap_or(""));
                }
            }
        }
        pact_sdk::DecisionStatus::Deny => {
            println!("Transaction denied");
            for eval in &decision.rule_evaluations {
                if eval.result == pact_sdk::EvaluationResult::Fail {
                    println!("  - {}: {}", eval.rule_name, eval.message.as_deref().unwrap_or(""));
                }
            }
        }
    }

    Ok(())
}
```

### Registering Plugins

```rust
use pact_sdk::{Registry, get_global_registry};
use std::sync::Arc;

// Create and register a plugin
let plugin = Arc::new(FinancePlugin::new());

// Using the global registry
let registry = get_global_registry();
registry.register(plugin.clone())?;

// Or create a local registry
let local_registry = Registry::new();
local_registry.register(plugin)?;

// Query plugins by entity or event type
if let Some(plugin) = registry.get_plugin_for_event_type("TRANSACTION_CREATED") {
    println!("Found plugin: {}", plugin.manifest().name);
}

// Get all entity types across plugins
let all_entity_types = registry.get_all_entity_types();
```

## API Reference

### Types

| Type | Description |
|------|-------------|
| `PactEvent` | Canonical event format for the PACT engine |
| `PactEntity` | Entity with typed data and metadata |
| `PactRule` | Compliance rule with condition and consequence |
| `PactDecision` | Result of rule evaluation |
| `Condition` | Rule condition (Compare, And, Or, Not, Exists, In, Range, Expression) |
| `Consequence` | Rule outcome (Allow, Deny, Flag) |
| `Severity` | Rule severity level (Low, Medium, High, Critical) |
| `ValidationResult` | Result of data validation |

### Client Methods

| Method | Description |
|--------|-------------|
| `submit_event(event)` | Submit an event for rule evaluation |
| `get_event(id)` | Retrieve an event by ID |
| `list_events(...)` | List events with optional filters |
| `create_entity(entity)` | Create a new entity |
| `get_entity(id)` | Retrieve an entity by ID |
| `update_entity(id, data)` | Update entity data |
| `delete_entity(id)` | Delete an entity |
| `create_rule(rule)` | Create a new rule |
| `get_rule(id)` | Retrieve a rule by ID |
| `update_rule(id, rule)` | Update a rule |
| `delete_rule(id)` | Delete a rule |
| `get_decision(id)` | Retrieve a decision by ID |
| `validate(entity_type, data)` | Validate data against schema |
| `health()` | Check service health |
| `ready()` | Check if service is ready |

### Condition Builder Methods

| Method | Description |
|--------|-------------|
| `field(path)` | Set the field path for comparison |
| `eq(value)` | Equal comparison |
| `ne(value)` | Not equal comparison |
| `gt(value)` | Greater than comparison |
| `gte(value)` | Greater than or equal comparison |
| `lt(value)` | Less than comparison |
| `lte(value)` | Less than or equal comparison |
| `contains(value)` | String contains check |
| `matches(pattern)` | Regex pattern match |
| `exists()` | Field existence check |
| `in_values(values)` | Value in set check |
| `between(min, max)` | Range check |
| `expression(expr, lang)` | Custom expression (CEL, etc.) |

### Rule Builder Methods

| Method | Description |
|--------|-------------|
| `id(id)` | Set rule ID |
| `name(name)` | Set rule name (required) |
| `description(desc)` | Set rule description |
| `jurisdiction(code)` | Set jurisdiction code |
| `severity(level)` | Set severity level |
| `when(condition)` | Set rule condition (required) |
| `when_field(field, op, value)` | Simple field comparison |
| `then_allow(code, msg)` | Allow consequence |
| `then_deny(code, msg)` | Deny consequence (required) |
| `then_flag(code, msg)` | Flag consequence |
| `tags(tags)` | Add tags |
| `effective_from(date)` | Set effective start date |
| `effective_to(date)` | Set effective end date |
| `metadata(key, value)` | Add metadata |

## Configuration

### Client Configuration

```rust
use pact_sdk::PactClient;
use std::time::Duration;

let client = PactClient::new("http://localhost:8080")
    // Authentication
    .with_api_key("your-api-key")
    // Request timeout (default: 30 seconds)
    .with_timeout(Duration::from_secs(60));
```

### Environment Variables

The SDK respects standard environment variables for HTTP clients:

- `HTTP_PROXY` / `HTTPS_PROXY`: Proxy configuration
- `NO_PROXY`: Hosts to bypass proxy

## Error Handling

```rust
use pact_sdk::{PactClient, ClientError};

let client = PactClient::new("http://localhost:8080");

match client.get_event("evt_123").await {
    Ok(event) => println!("Found event: {}", event.id),
    Err(ClientError::Api { status, message }) => {
        if status == 404 {
            println!("Event not found");
        } else {
            println!("API error {}: {}", status, message);
        }
    }
    Err(ClientError::Http(e)) => println!("Network error: {}", e),
    Err(ClientError::Serialization(e)) => println!("JSON error: {}", e),
}
```

## Testing

Run the test suite:

```bash
cargo test
```

Run tests with output:

```bash
cargo test -- --nocapture
```

Run specific test modules:

```bash
cargo test types::tests
cargo test builder::tests
cargo test client::tests
cargo test plugin::tests
cargo test registry::tests
```

## Documentation

- [PACT Protocol Documentation](https://www.arkaprotocol.com/docs)
- [API Reference](https://docs.pact-protocol.org/sdks/rust)
- [Examples](https://github.com/pact-engine/pact-sdk-rust/tree/main/examples)

## License

MIT License - see [LICENSE](LICENSE) for details.

## Contributing

Contributions are welcome! Please read our [Contributing Guide](CONTRIBUTING.md) for details on the process for submitting pull requests.

## Support

- GitHub Issues: [pact-engine/pact-sdk-rust/issues](https://github.com/pact-engine/pact-sdk-rust/issues)
- Documentation: [https://www.arkaprotocol.com/docs](https://www.arkaprotocol.com/docs)
