# PACT SDK for Go

The official Go SDK for the PACT Protocol, enabling seamless integration with PACT Core for policy-as-code compliance and rule evaluation.

## Installation

```bash
go get github.com/pact-engine/pact-sdk-go
```

## Quick Start

### Basic Client Usage

```go
package main

import (
    "context"
    "fmt"
    "log"
    "time"

    pact "github.com/pact-engine/pact-sdk-go/arka"
)

func main() {
    // Create a client with API key authentication
    client := pact.NewClient(
        "https://api.pact.example.com",
        pact.WithAPIKey("your-api-key"),
        pact.WithTimeout(30*time.Second),
    )

    ctx := context.Background()

    // Submit an event for evaluation
    event := &pact.PactEvent{
        Type:         "payment_initiated",
        EntityID:     "user_123",
        EntityType:   "User",
        Jurisdiction: "US",
        Payload: map[string]any{
            "amount":   1500.00,
            "currency": "USD",
            "recipient": "merchant_456",
        },
    }

    decision, err := client.SubmitEvent(ctx, event)
    if err != nil {
        log.Fatalf("Failed to submit event: %v", err)
    }

    fmt.Printf("Decision: %s\n", decision.Status)
    for _, eval := range decision.RuleEvaluations {
        fmt.Printf("  Rule %s: %s\n", eval.RuleName, eval.Result)
    }
}
```

### Building Rules with the Fluent API

```go
package main

import (
    pact "github.com/pact-engine/pact-sdk-go/arka"
)

func main() {
    // Build a simple rule using the fluent builder
    rule := pact.NewRule().
        ID("rule_high_amount").
        Name("High Amount Transaction").
        Description("Flag transactions over $10,000 for review").
        Jurisdiction("US").
        Severity(pact.SeverityHigh).
        When(
            pact.And(
                pact.NewCondition().Field("payload.amount").Gt(10000).Build(),
                pact.NewCondition().Field("payload.currency").Eq("USD").Build(),
            ),
        ).
        ThenFlag("HIGH_AMOUNT", "Transaction amount exceeds threshold").
        Tags("compliance", "aml").
        MustBuild()

    // Use the rule...
}
```

### Creating Complex Conditions

```go
package main

import (
    pact "github.com/pact-engine/pact-sdk-go/arka"
)

func main() {
    // Combine conditions with logical operators
    condition := pact.And(
        // Amount is high
        pact.NewCondition().Field("payload.amount").Gt(5000).Build(),

        // User risk score is elevated
        pact.Or(
            pact.NewCondition().Field("entity.risk_score").Gte(0.7).Build(),
            pact.NewCondition().Field("entity.is_new_user").Eq(true).Build(),
        ),

        // Not in whitelisted countries
        pact.Not(
            pact.NewCondition().Field("jurisdiction").In("US", "CA", "GB").Build(),
        ),
    )

    // Other condition types
    existsCondition := pact.NewCondition().Field("payload.verification_code").Exists().Build()
    rangeCondition := pact.NewCondition().Field("payload.amount").Between(1000, 10000).Build()
    containsCondition := pact.NewCondition().Field("payload.note").Contains("urgent").Build()
    regexCondition := pact.NewCondition().Field("payload.email").Matches(`^[a-z]+@example\.com$`).Build()
    expressionCondition := pact.NewCondition().Expression("event.amount * event.quantity > 10000", "cel").Build()
}
```

### Building Domain Plugins

```go
package main

import (
    "context"

    pact "github.com/pact-engine/pact-sdk-go/arka"
)

// PaymentPlugin is a custom domain plugin for payment processing
type PaymentPlugin struct {
    *pact.BasePlugin
}

func NewPaymentPlugin() *PaymentPlugin {
    manifest := pact.PluginManifest{
        ID:              "payment-plugin",
        Name:            "Payment Domain Plugin",
        Version:         "1.0.0",
        Author:          "Your Company",
        Description:     "Handles payment-related events and rules",
        EntityTypes:     []string{"Payment", "Transaction"},
        EventTypes:      []string{"payment_initiated", "payment_completed", "payment_failed"},
        PactCoreVersion: "1.0.0",
    }

    entityTypes := []pact.PactEntityType{
        {
            Name:           "Payment",
            Description:    "A payment transaction",
            RequiredFields: []string{"amount", "currency", "sender_id", "recipient_id"},
            Schema: map[string]any{
                "type": "object",
                "properties": map[string]any{
                    "amount":       map[string]any{"type": "number", "minimum": 0},
                    "currency":     map[string]any{"type": "string", "enum": []string{"USD", "EUR", "GBP"}},
                    "sender_id":    map[string]any{"type": "string"},
                    "recipient_id": map[string]any{"type": "string"},
                },
            },
        },
    }

    base := pact.NewBasePlugin(manifest, entityTypes, nil)

    plugin := &PaymentPlugin{BasePlugin: base}

    // Set lifecycle hooks
    base.SetHooks(&pact.PluginHooks{
        OnLoad: func(ctx context.Context) error {
            // Initialize plugin resources
            return nil
        },
        BeforeEventProcess: func(ctx context.Context, event *pact.PactEvent) (*pact.PactEvent, error) {
            // Enrich event before processing
            return event, nil
        },
    })

    return plugin
}

// GetDefaultRules returns default rules for the payment domain
func (p *PaymentPlugin) GetDefaultRules() []pact.PactRule {
    return []pact.PactRule{
        p.CreateRule(
            "Large Transaction Alert",
            pact.NewCondition().Field("payload.amount").Gt(50000).Build(),
            pact.Consequence{
                Decision: pact.DecisionFlag,
                Code:     "LARGE_TRANSACTION",
                Message:  "Transaction exceeds large transaction threshold",
            },
            pact.WithSeverity(pact.SeverityHigh),
            pact.WithTags("aml", "monitoring"),
        ),
    }
}

func main() {
    // Create plugin registry
    registry := pact.NewRegistry()

    // Register the payment plugin
    plugin := NewPaymentPlugin()
    if err := registry.Register(context.Background(), plugin); err != nil {
        panic(err)
    }

    // Get plugin for a specific entity type
    p := registry.GetPluginForEntityType("Payment")
    if p != nil {
        rules := p.GetDefaultRules()
        // Use rules...
    }
}
```

## API Reference

### Client

The `Client` type provides methods for interacting with PACT Core.

#### Creating a Client

```go
client := pact.NewClient(baseURL string, opts ...ClientOption)
```

##### Client Options

| Option | Description |
|--------|-------------|
| `WithAPIKey(key string)` | Set API key for authentication |
| `WithTimeout(timeout time.Duration)` | Set HTTP client timeout (default: 30s) |
| `WithHeader(key, value string)` | Add custom HTTP header |

#### Event Operations

```go
// Submit an event for evaluation
decision, err := client.SubmitEvent(ctx, event *PactEvent) (*PactDecision, error)

// Get an event by ID
event, err := client.GetEvent(ctx, eventID string) (*PactEvent, error)

// List events with filters
events, err := client.ListEvents(ctx, opts ListEventsOptions) ([]PactEvent, error)
```

#### Entity Operations

```go
// Create an entity
entity, err := client.CreateEntity(ctx, entity *PactEntity) (*PactEntity, error)

// Get an entity by ID
entity, err := client.GetEntity(ctx, entityID string) (*PactEntity, error)

// Update an entity
entity, err := client.UpdateEntity(ctx, entityID string, data map[string]any) (*PactEntity, error)

// Delete an entity
err := client.DeleteEntity(ctx, entityID string) error
```

#### Rule Operations

```go
// Create a rule
rule, err := client.CreateRule(ctx, rule *PactRule) (*PactRule, error)

// Get a rule by ID
rule, err := client.GetRule(ctx, ruleID string) (*PactRule, error)

// Update a rule
rule, err := client.UpdateRule(ctx, ruleID string, rule *PactRule) (*PactRule, error)

// Delete a rule
err := client.DeleteRule(ctx, ruleID string) error
```

#### Other Operations

```go
// Get a decision by ID
decision, err := client.GetDecision(ctx, decisionID string) (*PactDecision, error)

// Validate data against schema
result, err := client.Validate(ctx, entityType string, data map[string]any) (*ValidationResult, error)

// Check service health
health, err := client.Health(ctx) (map[string]any, error)

// Check if service is ready
ready := client.Ready(ctx) bool
```

### Types

#### Core Types

| Type | Description |
|------|-------------|
| `PactEvent` | Canonical event representation |
| `PactEntity` | Entity with type and data |
| `PactRule` | Rule definition with condition and consequence |
| `PactDecision` | Decision result from rule evaluation |
| `ValidationResult` | Validation result with errors and warnings |

#### Condition Types

| Type | Description |
|------|-------------|
| `CompareCondition` | Field comparison (eq, ne, gt, gte, lt, lte, contains, etc.) |
| `AndCondition` | Logical AND of multiple conditions |
| `OrCondition` | Logical OR of multiple conditions |
| `NotCondition` | Logical negation of a condition |
| `ExistsCondition` | Check if field exists |
| `InCondition` | Check if value is in a set |
| `RangeCondition` | Check if value is in a numeric range |
| `ExpressionCondition` | Custom expression (CEL, etc.) |

#### Enums

##### Severity
- `SeverityLow` - Low priority
- `SeverityMedium` - Medium priority (default)
- `SeverityHigh` - High priority
- `SeverityCritical` - Critical priority

##### Decision
- `DecisionAllow` - Allow the action
- `DecisionDeny` - Deny the action
- `DecisionFlag` - Flag for review

##### DecisionStatus
- `DecisionStatusAllow` - Action allowed
- `DecisionStatusAllowWithFlags` - Action allowed with flags
- `DecisionStatusDeny` - Action denied

##### EvaluationResult
- `EvaluationResultPass` - Rule passed
- `EvaluationResultFail` - Rule failed
- `EvaluationResultSkip` - Rule skipped
- `EvaluationResultError` - Error evaluating rule

### Builders

#### ConditionBuilder

```go
condition := pact.NewCondition().
    Field("path.to.field").
    Gt(value).  // or Eq, Ne, Lt, Lte, Gte, Contains, StartsWith, EndsWith, Matches, In, Between, Exists
    Build()
```

#### RuleBuilder

```go
rule, err := pact.NewRule().
    ID("custom_id").              // Optional, auto-generated if not set
    Name("Rule Name").            // Required
    Description("Description").   // Optional, defaults to name
    Jurisdiction("US").           // Optional
    Severity(pact.SeverityHigh).  // Optional, defaults to Medium
    When(condition).              // Required
    ThenDeny("CODE", "Message").  // Required: ThenDeny, ThenFlag, or ThenAllow
    Tags("tag1", "tag2").         // Optional
    EffectiveFrom(time.Now()).    // Optional
    EffectiveTo(time.Now()).      // Optional
    Metadata("key", "value").     // Optional
    Build()
```

### Plugin System

#### DomainPlugin Interface

```go
type DomainPlugin interface {
    Manifest() PluginManifest
    Hooks() *PluginHooks
    GetEntityTypes() []PactEntityType
    GetDefaultRules() []PactRule
    MapToCanonicalEvent(event DomainEvent) (*PactEvent, error)
    ValidateDomainData(entityType string, data map[string]any) ValidationResult
    GetEvaluationContext(event *PactEvent, entity *PactEntity) map[string]any
    SerializeForChain(data any) ([]byte, error)
    DeserializeFromChain(data []byte) (any, error)
}
```

#### Registry

```go
// Create a registry
registry := pact.NewRegistry()

// Register a plugin
err := registry.Register(ctx, plugin)

// Unregister a plugin
err := registry.Unregister(ctx, pluginID)

// Get plugin by ID
plugin := registry.GetPlugin(pluginID)

// Get plugin for entity type
plugin := registry.GetPluginForEntityType(entityType)

// Get plugin for event type
plugin := registry.GetPluginForEventType(eventType)

// Get all plugins
plugins := registry.GetAllPlugins()

// Subscribe to registry events
registry.On(func(event RegistryEvent) {
    // Handle event
})
```

## Configuration Options

### Environment Variables

| Variable | Description | Default |
|----------|-------------|---------|
| `PACT_API_URL` | Base URL for PACT Core API | - |
| `PACT_API_KEY` | API key for authentication | - |
| `PACT_TIMEOUT` | HTTP client timeout | 30s |

### Client Configuration Example

```go
client := pact.NewClient(
    os.Getenv("PACT_API_URL"),
    pact.WithAPIKey(os.Getenv("PACT_API_KEY")),
    pact.WithTimeout(60*time.Second),
    pact.WithHeader("X-Request-ID", uuid.New().String()),
    pact.WithHeader("X-Tenant-ID", "tenant_123"),
)
```

## Testing

Run the test suite:

```bash
go test ./...
```

Run with verbose output:

```bash
go test -v ./...
```

Run with coverage:

```bash
go test -cover ./...
```

## Error Handling

The SDK returns errors for various failure cases:

```go
decision, err := client.SubmitEvent(ctx, event)
if err != nil {
    // Check for specific error types
    if strings.Contains(err.Error(), "HTTP 401") {
        // Authentication error
    } else if strings.Contains(err.Error(), "HTTP 404") {
        // Not found
    } else if strings.Contains(err.Error(), "HTTP 5") {
        // Server error
    }
}
```

## Documentation

For comprehensive documentation, visit:
- [PACT Protocol Documentation](https://www.arkaprotocol.com/docs)
- [API Reference](https://www.arkaprotocol.com/docs/api)
- [Plugin Development Guide](https://www.arkaprotocol.com/docs/plugins)

## License

This SDK is released under the MIT License. See the LICENSE file for details.

## Support

- Documentation: https://www.arkaprotocol.com/docs
- Issues: https://github.com/pact-engine/pact-sdk-go/issues
- Email: support@arkaprotocol.com
