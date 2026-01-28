# PACT SDK for .NET

The official .NET SDK for building domain plugins and integrating with the PACT Protocol compliance engine.

## Installation

Install via NuGet Package Manager:

```bash
dotnet add package Pact.Sdk
```

Or via the Package Manager Console in Visual Studio:

```powershell
Install-Package Pact.Sdk
```

## Quick Start

### Using the PACT Client

Connect to a PACT Core instance and submit events for compliance evaluation:

```csharp
using Pact.Sdk;

// Initialize the client
using var client = new PactClient(
    baseUrl: "https://api.pact.example.com",
    apiKey: "your-api-key",
    timeout: TimeSpan.FromSeconds(30)
);

// Submit an event for evaluation
var pactEvent = new PactEvent
{
    Id = $"evt_{Guid.NewGuid():N}",
    Source = "my-application",
    Type = "transaction_created",
    EntityId = "txn-12345",
    EntityType = "Transaction",
    Jurisdiction = "US",
    Payload = new Dictionary<string, object>
    {
        ["amount"] = 15000,
        ["currency"] = "USD",
        ["sender"] = "account-001",
        ["recipient"] = "account-002"
    },
    OccurredAt = DateTimeOffset.UtcNow,
    ReceivedAt = DateTimeOffset.UtcNow
};

var decision = await client.SubmitEventAsync(pactEvent);

Console.WriteLine($"Decision: {decision.Status}");
foreach (var eval in decision.RuleEvaluations)
{
    Console.WriteLine($"  Rule '{eval.RuleName}': {eval.Result}");
}
```

### Creating a Domain Plugin

Build custom domain plugins to extend PACT's compliance capabilities:

```csharp
using Pact.Sdk;

public class FinancePlugin : BasePlugin
{
    public FinancePlugin() : base(
        manifest: new PluginManifest
        {
            Id = "finance-plugin",
            Name = "Finance Compliance Plugin",
            Version = "1.0.0",
            Author = "Your Organization",
            Description = "Financial transaction compliance rules",
            EntityTypes = ["Transaction", "Account"],
            EventTypes = ["transaction_created", "transaction_updated"],
            PactCoreVersion = "1.0.0"
        },
        entityTypes: new List<PactEntityType>
        {
            new()
            {
                Name = "Transaction",
                Description = "Financial transaction",
                Schema = new Dictionary<string, object>
                {
                    ["amount"] = new { type = "number" },
                    ["currency"] = new { type = "string" },
                    ["sender"] = new { type = "string" },
                    ["recipient"] = new { type = "string" }
                },
                RequiredFields = ["amount", "currency"]
            }
        },
        defaultRules: new List<PactRule>
        {
            RuleBuilder.Create()
                .Id("finance-001")
                .Name("High Value Transaction Alert")
                .Description("Flag transactions over $10,000 for review")
                .WithSeverity(Severity.High)
                .Jurisdiction("US")
                .When(ConditionBuilder.Create()
                    .Field("payload.amount")
                    .Gt(10000)
                    .Build())
                .ThenFlag("HIGH_VALUE", "Transaction exceeds $10,000 threshold")
                .Tags("aml", "compliance")
                .Build()
        })
    {
    }
}
```

### Building Complex Conditions

Use the fluent condition builder for readable rule definitions:

```csharp
// Simple comparison
var amountCheck = ConditionBuilder.Create()
    .Field("payload.amount")
    .Gt(10000)
    .Build();

// Combined conditions with AND
var complexCondition = ConditionBuilder.And(
    ConditionBuilder.Create().Field("payload.amount").Gt(10000).Build(),
    ConditionBuilder.Create().Field("payload.currency").Eq("USD").Build()
);

// Combined conditions with OR
var multipleStatus = ConditionBuilder.Or(
    ConditionBuilder.Create().Field("status").Eq("pending").Build(),
    ConditionBuilder.Create().Field("status").Eq("review").Build()
);

// Negation
var notBlocked = ConditionBuilder.Not(
    ConditionBuilder.Create().Field("blocked").Eq(true).Build()
);

// Value in set
var allowedCountries = ConditionBuilder.Create()
    .Field("country")
    .In("US", "CA", "GB", "DE")
    .Build();

// Range check
var ageRange = ConditionBuilder.Create()
    .Field("age")
    .Between(18, 65)
    .Build();

// Field existence
var hasMetadata = ConditionBuilder.Create()
    .Field("metadata.verification_id")
    .Exists()
    .Build();

// Custom expression (CEL)
var customExpression = ConditionBuilder.Create()
    .Expression("payload.amount > 1000 && payload.risk_score < 50")
    .Build();
```

### Using the Rule Builder

Create compliance rules with the fluent rule builder:

```csharp
var rule = RuleBuilder.Create()
    .Id("rule-001")
    .Name("Suspicious Transaction Pattern")
    .Description("Detects potentially suspicious transaction patterns")
    .Jurisdiction("US")
    .WithSeverity(Severity.Critical)
    .When(ConditionBuilder.And(
        ConditionBuilder.Create().Field("payload.amount").Gt(5000).Build(),
        ConditionBuilder.Create().Field("payload.frequency").Gt(10).Build(),
        ConditionBuilder.Create().Field("payload.country").In("XX", "YY").Build()
    ))
    .ThenDeny("SUSPICIOUS_PATTERN", "Transaction blocked due to suspicious pattern")
    .Tags("aml", "fraud", "high-risk")
    .EffectiveFrom(DateTimeOffset.Parse("2024-01-01T00:00:00Z"))
    .Metadata("regulation", "BSA/AML")
    .Build();
```

### Registering Plugins

Use the plugin registry to manage domain plugins:

```csharp
using Pact.Sdk;

// Create and register plugins
var financePlugin = new FinancePlugin();
var kycPlugin = new KycPlugin();

var registry = new PluginRegistry();

// Subscribe to registry events
registry.OnEvent(evt =>
{
    Console.WriteLine($"Registry event: {evt.Type} for plugin {evt.PluginId}");
});

// Register plugins (validates dependencies and conflicts)
await registry.RegisterAsync(financePlugin);
await registry.RegisterAsync(kycPlugin);

// Look up plugins by entity or event type
var handler = registry.GetPluginForEntityType("Transaction");
var eventHandler = registry.GetPluginForEventType("transaction_created");

// Get all registered rules
var allRules = registry.GetAllDefaultRules();

// Get registry statistics
var stats = registry.GetStats();
Console.WriteLine($"Registered plugins: {stats["plugin_count"]}");
```

### Using the Global Registry

For simpler applications, use the singleton global registry:

```csharp
using Pact.Sdk;

// Register with global registry
await GlobalRegistry.Instance.RegisterAsync(new FinancePlugin());

// Access plugins globally
var plugin = GlobalRegistry.Instance.GetPlugin("finance-plugin");

// Reset registry (useful for testing)
GlobalRegistry.Reset();
```

### Plugin Lifecycle Hooks

Implement lifecycle hooks for advanced plugin behavior:

```csharp
public class AdvancedPlugin : BasePlugin
{
    public AdvancedPlugin() : base(/* ... */)
    {
        Hooks = new PluginHooks
        {
            OnLoad = async () =>
            {
                // Initialize resources when plugin loads
                Console.WriteLine("Plugin loaded");
            },
            OnUnload = async () =>
            {
                // Cleanup resources when plugin unloads
                Console.WriteLine("Plugin unloaded");
            },
            BeforeEventProcess = async (evt) =>
            {
                // Modify or enrich event before processing
                return evt with
                {
                    Metadata = new Dictionary<string, object>(evt.Metadata)
                    {
                        ["processed_at"] = DateTimeOffset.UtcNow
                    }
                };
            },
            AfterDecision = async (evt, decision) =>
            {
                // React to decisions (logging, notifications, etc.)
                if (decision.Status == DecisionStatus.Deny)
                {
                    await NotifyComplianceTeam(evt, decision);
                }
            },
            OnRulesUpdated = async (rules) =>
            {
                // React to rule changes
                Console.WriteLine($"Rules updated: {rules.Count} rules");
            }
        };
    }
}
```

## API Reference

### PactClient

| Method | Description |
|--------|-------------|
| `SubmitEventAsync(PactEvent)` | Submit an event for compliance evaluation |
| `GetEventAsync(string)` | Retrieve an event by ID |
| `CreateEntityAsync(PactEntity)` | Create a new entity |
| `GetEntityAsync(string)` | Retrieve an entity by ID |
| `UpdateEntityAsync(string, Dictionary)` | Update entity data |
| `DeleteEntityAsync(string)` | Delete an entity |
| `CreateRuleAsync(PactRule)` | Create a compliance rule |
| `GetRuleAsync(string)` | Retrieve a rule by ID |
| `UpdateRuleAsync(string, PactRule)` | Update a rule |
| `DeleteRuleAsync(string)` | Delete a rule |
| `GetDecisionAsync(string)` | Retrieve a decision by ID |
| `ValidateAsync(string, Dictionary)` | Validate data against entity schema |
| `IsHealthyAsync()` | Check if PACT Core is healthy |
| `IsReadyAsync()` | Check if PACT Core is ready |

### Types

| Type | Description |
|------|-------------|
| `PactEvent` | Canonical event format for compliance evaluation |
| `PactEntity` | Entity representation with typed data |
| `PactRule` | Compliance rule definition |
| `PactDecision` | Result of rule evaluation |
| `Condition` | Base class for rule conditions (polymorphic) |
| `Consequence` | Rule consequence (Allow, Deny, Flag) |
| `PluginManifest` | Plugin metadata and capabilities |
| `ValidationResult` | Data validation result |

### Condition Types

| Type | Description |
|------|-------------|
| `CompareCondition` | Field comparison (eq, ne, gt, gte, lt, lte, contains, regex) |
| `AndCondition` | Logical AND of multiple conditions |
| `OrCondition` | Logical OR of multiple conditions |
| `NotCondition` | Logical NOT of a condition |
| `ExistsCondition` | Check if field exists |
| `InCondition` | Check if field value is in a set |
| `RangeCondition` | Check if field is within numeric range |
| `ExpressionCondition` | Custom CEL expression |

### Enums

| Enum | Values |
|------|--------|
| `Severity` | Low, Medium, High, Critical |
| `Decision` | Allow, Deny, Flag |
| `DecisionStatus` | Allow, AllowWithFlags, Deny |
| `EvaluationResult` | Pass, Fail, Skip, Error |

## Configuration

### Client Configuration

```csharp
var client = new PactClient(
    baseUrl: "https://api.pact.example.com",  // PACT Core URL
    apiKey: "your-api-key",                   // Optional API key
    timeout: TimeSpan.FromSeconds(30)         // Request timeout
);
```

### Environment Variables

Configure the client using environment variables:

| Variable | Description |
|----------|-------------|
| `PACT_API_URL` | Base URL for PACT Core |
| `PACT_API_KEY` | API key for authentication |
| `PACT_TIMEOUT` | Request timeout in seconds |

Example:

```csharp
var client = new PactClient(
    baseUrl: Environment.GetEnvironmentVariable("PACT_API_URL")
        ?? "http://localhost:8080",
    apiKey: Environment.GetEnvironmentVariable("PACT_API_KEY")
);
```

## Testing

Run the test suite:

```bash
cd arka-sdk-dotnet
dotnet test
```

Run with coverage:

```bash
dotnet test --collect:"XPlat Code Coverage"
```

## Requirements

- .NET 8.0 or later
- System.Text.Json 8.0.0+

## Documentation

For comprehensive documentation, tutorials, and API references, visit:

**[https://www.arkaprotocol.com/docs](https://www.arkaprotocol.com/docs)**

## License

This SDK is licensed under the MIT License. See the LICENSE file for details.

## Support

- Documentation: [https://www.arkaprotocol.com/docs](https://www.arkaprotocol.com/docs)
- Issues: [GitHub Issues](https://github.com/pact-engine/pact-sdk-dotnet/issues)
- Email: support@arkaprotocol.com
