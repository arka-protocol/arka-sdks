# Arka PACT SDK for Java

The official Java SDK for integrating with the Arka PACT Protocol. This SDK enables Java applications to build domain plugins, submit events, manage entities and rules, and interact with the PACT Core engine.

## Table of Contents

- [Installation](#installation)
- [Quick Start](#quick-start)
- [Features](#features)
- [API Reference](#api-reference)
  - [PactClient](#pactclient)
  - [Domain Plugins](#domain-plugins)
  - [Rule Builder](#rule-builder)
  - [Condition Builder](#condition-builder)
- [Configuration](#configuration)
- [Types](#types)
- [Error Handling](#error-handling)
- [Examples](#examples)
- [Testing](#testing)
- [Documentation](#documentation)
- [License](#license)

## Installation

### Maven

Add the following dependency to your `pom.xml`:

```xml
<dependency>
    <groupId>com.arka</groupId>
    <artifactId>arka-sdk-java</artifactId>
    <version>0.1.0</version>
</dependency>
```

### Gradle

Add to your `build.gradle`:

```groovy
implementation 'com.arka:arka-sdk-java:0.1.0'
```

Or for Kotlin DSL (`build.gradle.kts`):

```kotlin
implementation("com.arka:arka-sdk-java:0.1.0")
```

## Quick Start

### 1. Initialize the Client

```java
import com.arka.sdk.client.PactClient;

// Basic initialization
PactClient client = new PactClient("https://api.pact-core.example.com");

// With API key authentication
PactClient client = new PactClient(
    "https://api.pact-core.example.com",
    "your-api-key"
);

// With custom timeout
PactClient client = new PactClient(
    "https://api.pact-core.example.com",
    "your-api-key",
    Duration.ofSeconds(60)
);
```

### 2. Submit an Event

```java
import com.arka.sdk.types.*;
import java.time.Instant;
import java.util.Map;

PactEvent event = PactEvent.builder()
    .id("evt_" + UUID.randomUUID())
    .source("my-application")
    .type("transaction_created")
    .entityId("txn_123")
    .entityType("Transaction")
    .jurisdiction("US")
    .payload(Map.of(
        "amount", 5000.0,
        "currency", "USD",
        "recipient", "user_456"
    ))
    .occurredAt(Instant.now())
    .build();

PactDecision decision = client.submitEvent(event);

if (decision.isDenied()) {
    System.out.println("Transaction blocked: " + decision.triggeredRules());
} else if (decision.hasFlagged()) {
    System.out.println("Transaction flagged for review");
} else {
    System.out.println("Transaction approved");
}
```

### 3. Create a Rule

```java
import com.arka.sdk.builder.RuleBuilder;
import com.arka.sdk.types.*;

PactRule rule = RuleBuilder.rule()
    .name("High Value Transaction Alert")
    .description("Flag transactions over $10,000 for review")
    .severity(Severity.HIGH)
    .jurisdiction("US")
    .whenField("amount", "gt", 10000)
    .thenFlag("HIGH_VALUE", "Transaction exceeds $10,000 threshold")
    .tags("compliance", "aml", "high-value")
    .build();

PactRule createdRule = client.createRule(rule);
```

## Features

- **HTTP Client**: Full-featured client for PACT Core API communication
- **Domain Plugins**: Framework for building custom domain plugins
- **Type-Safe Builders**: Fluent builders for rules and conditions
- **JSON Serialization**: Automatic JSON serialization/deserialization with Jackson
- **Async Support**: Asynchronous API operations with CompletableFuture
- **Canonical JSON**: Deterministic JSON serialization for blockchain operations

## API Reference

### PactClient

The main client for interacting with PACT Core.

#### Constructor Options

```java
// Basic
new PactClient(String baseUrl)

// With API key
new PactClient(String baseUrl, String apiKey)

// With API key and timeout
new PactClient(String baseUrl, String apiKey, Duration timeout)
```

#### Event Operations

```java
// Submit an event for rule evaluation
PactDecision submitEvent(PactEvent event) throws IOException

// Get an event by ID
PactEvent getEvent(String eventId) throws IOException

// Async variants
CompletableFuture<PactDecision> submitEventAsync(PactEvent event)
CompletableFuture<PactEvent> getEventAsync(String eventId)
```

#### Entity Operations

```java
// Create a new entity
PactEntity createEntity(PactEntity entity) throws IOException

// Get an entity by ID
PactEntity getEntity(String entityId) throws IOException

// Update an entity's data
PactEntity updateEntity(String entityId, Map<String, Object> data) throws IOException

// Delete an entity
void deleteEntity(String entityId) throws IOException
```

#### Rule Operations

```java
// Create a new rule
PactRule createRule(PactRule rule) throws IOException

// Get a rule by ID
PactRule getRule(String ruleId) throws IOException

// Update a rule
PactRule updateRule(String ruleId, PactRule rule) throws IOException

// Delete a rule
void deleteRule(String ruleId) throws IOException
```

#### Decision Operations

```java
// Get a decision by ID
PactDecision getDecision(String decisionId) throws IOException
```

#### Validation Operations

```java
// Validate data against entity type schema
ValidationResult validate(String entityType, Map<String, Object> data) throws IOException
```

#### Health Operations

```java
// Check if the service is healthy
boolean isHealthy()

// Check if the service is ready
boolean isReady()
```

### Domain Plugins

Create custom domain plugins by extending `BasePlugin`:

```java
import com.arka.sdk.plugin.BasePlugin;
import com.arka.sdk.plugin.PluginHooks;
import com.arka.sdk.types.*;

public class FinancePlugin extends BasePlugin {

    public FinancePlugin() {
        super(
            PluginManifest.builder()
                .id("finance-plugin")
                .name("Finance Domain Plugin")
                .version("1.0.0")
                .author("Your Company")
                .description("Financial compliance rules")
                .entityTypes(List.of("Transaction", "Account", "Payment"))
                .eventTypes(List.of("transaction_created", "payment_processed"))
                .pactCoreVersion(">=1.0.0")
                .build(),
            List.of(
                PactEntityType.builder()
                    .name("Transaction")
                    .description("A financial transaction")
                    .requiredFields(List.of("amount", "currency", "type"))
                    .build()
            ),
            List.of(
                // Default rules
            )
        );

        // Set up lifecycle hooks
        setHooks(new PluginHooks()
            .onLoad(() -> System.out.println("Plugin loaded"))
            .onUnload(() -> System.out.println("Plugin unloaded"))
            .beforeEventProcess(event -> {
                // Pre-process event
                return event;
            })
            .afterDecision(event -> {
                // Post-decision processing
            })
        );
    }

    @Override
    public Map<String, Object> getEvaluationContext(PactEvent event, PactEntity entity) {
        // Provide custom context for rule evaluation
        return Map.of(
            "riskScore", calculateRiskScore(event),
            "accountTier", getAccountTier(entity)
        );
    }
}
```

### Rule Builder

Build rules with a fluent API:

```java
import com.arka.sdk.builder.RuleBuilder;
import static com.arka.sdk.builder.RuleBuilder.rule;

// Simple rule
PactRule rule = rule()
    .name("Blocked Countries")
    .severity(Severity.CRITICAL)
    .whenField("country", "in", List.of("XX", "YY"))
    .thenDeny("BLOCKED_COUNTRY", "Transactions from this country are not allowed")
    .build();

// Rule with custom condition
PactRule complexRule = rule()
    .id("rule_custom_id")  // Optional: auto-generated if not provided
    .name("Complex Compliance Rule")
    .description("Detailed description of the rule")
    .jurisdiction("US")
    .severity(Severity.HIGH)
    .when(myComplexCondition)  // Custom Condition object
    .consequence(myConsequence)  // Custom Consequence object
    .tags("compliance", "aml")
    .effectiveFrom(Instant.parse("2024-01-01T00:00:00Z"))
    .effectiveTo(Instant.parse("2024-12-31T23:59:59Z"))
    .metadata("author", "compliance-team")
    .build();
```

### Condition Builder

Build conditions programmatically:

```java
import com.arka.sdk.builder.ConditionBuilder;
import static com.arka.sdk.builder.ConditionBuilder.*;

// Simple comparison
Condition amountCheck = condition()
    .field("amount")
    .gt(10000)
    .build();

// Other comparison operators
condition().field("status").eq("ACTIVE").build();      // equals
condition().field("status").ne("BLOCKED").build();     // not equals
condition().field("age").gte(18).build();              // greater than or equal
condition().field("balance").lt(0).build();            // less than
condition().field("count").lte(100).build();           // less than or equal
condition().field("email").contains("@company.com").build();
condition().field("phone").matches("^\\+1\\d{10}$").build();

// Field existence
condition().field("optionalField").exists().build();

// Value in list
condition().field("country").in(List.of("US", "CA", "UK")).build();

// Range check
condition().field("price").between(10.0, 100.0).build();

// CEL expression
condition().expression("event.payload.amount > 1000 && entity.data.verified", "cel").build();

// Logical combinations
Condition and = ConditionBuilder.and(List.of(cond1, cond2, cond3));
Condition or = ConditionBuilder.or(List.of(cond1, cond2));
Condition not = ConditionBuilder.not(condition);

// Complex nested conditions
Condition complex = ConditionBuilder.and(List.of(
    ConditionBuilder.or(List.of(
        condition().field("amount").gt(10000).build(),
        condition().field("risk_level").eq("HIGH").build()
    )),
    ConditionBuilder.not(
        condition().field("verified").eq(true).build()
    )
));
```

## Configuration

### Environment Variables

| Variable | Description | Default |
|----------|-------------|---------|
| `PACT_API_URL` | PACT Core API base URL | - |
| `PACT_API_KEY` | API key for authentication | - |
| `PACT_TIMEOUT_SECONDS` | Request timeout in seconds | 30 |

### Client Configuration

```java
// Using environment variables
String baseUrl = System.getenv("PACT_API_URL");
String apiKey = System.getenv("PACT_API_KEY");
int timeout = Integer.parseInt(System.getenv().getOrDefault("PACT_TIMEOUT_SECONDS", "30"));

PactClient client = new PactClient(
    baseUrl,
    apiKey,
    Duration.ofSeconds(timeout)
);
```

## Types

### Core Types

| Type | Description |
|------|-------------|
| `PactEvent` | Canonical event format for rule evaluation |
| `PactEntity` | Entity representation with type, data, and metadata |
| `PactRule` | Rule definition with conditions and consequences |
| `PactDecision` | Decision result from rule evaluation |
| `DomainEvent` | Domain-specific event before canonical conversion |

### Enums

| Enum | Values |
|------|--------|
| `Severity` | `LOW`, `MEDIUM`, `HIGH`, `CRITICAL` |
| `Decision` | `ALLOW`, `DENY`, `FLAG` |
| `DecisionStatus` | `ALLOW`, `ALLOW_WITH_FLAGS`, `DENY` |

### Condition Types

| Type | Description |
|------|-------------|
| `Condition.Compare` | Field comparison (eq, ne, gt, gte, lt, lte, contains, regex) |
| `Condition.And` | Logical AND of multiple conditions |
| `Condition.Or` | Logical OR of multiple conditions |
| `Condition.Not` | Logical NOT of a condition |
| `Condition.Exists` | Check if field exists |
| `Condition.In` | Check if value is in a list |
| `Condition.Range` | Check if value is within a range |
| `Condition.Expression` | CEL or custom expression |

## Error Handling

The SDK throws `IOException` for network and API errors:

```java
try {
    PactDecision decision = client.submitEvent(event);
} catch (IOException e) {
    if (e.getMessage().contains("HTTP 401")) {
        // Handle authentication error
    } else if (e.getMessage().contains("HTTP 404")) {
        // Handle not found
    } else if (e.getMessage().contains("HTTP 400")) {
        // Handle validation error
    } else {
        // Handle other errors
    }
}
```

For async operations, errors are wrapped in `CompletionException`:

```java
client.submitEventAsync(event)
    .thenAccept(decision -> {
        // Handle success
    })
    .exceptionally(ex -> {
        Throwable cause = ex.getCause();
        if (cause instanceof IOException) {
            // Handle API error
        }
        return null;
    });
```

## Examples

### Complete Transaction Monitoring Example

```java
import com.arka.sdk.client.PactClient;
import com.arka.sdk.builder.*;
import com.arka.sdk.types.*;

import java.time.Duration;
import java.time.Instant;
import java.util.List;
import java.util.Map;
import java.util.UUID;

public class TransactionMonitor {
    private final PactClient client;

    public TransactionMonitor(String apiUrl, String apiKey) {
        this.client = new PactClient(apiUrl, apiKey, Duration.ofSeconds(30));
    }

    public void setupRules() throws Exception {
        // High value transaction rule
        PactRule highValueRule = RuleBuilder.rule()
            .name("High Value Transaction")
            .description("Flag transactions over $10,000")
            .severity(Severity.HIGH)
            .jurisdiction("US")
            .whenField("amount", "gt", 10000)
            .thenFlag("HIGH_VALUE", "Transaction exceeds $10,000")
            .tags("aml", "compliance")
            .build();

        client.createRule(highValueRule);

        // Blocked country rule
        PactRule blockedCountryRule = RuleBuilder.rule()
            .name("Blocked Countries")
            .severity(Severity.CRITICAL)
            .whenField("country", "in", List.of("XX", "YY", "ZZ"))
            .thenDeny("BLOCKED_COUNTRY", "Transactions from blocked countries are prohibited")
            .build();

        client.createRule(blockedCountryRule);

        // Velocity check rule
        Condition velocityCondition = ConditionBuilder.and(List.of(
            ConditionBuilder.condition().field("velocity_24h").gt(5).build(),
            ConditionBuilder.condition().field("amount").gt(1000).build()
        ));

        PactRule velocityRule = RuleBuilder.rule()
            .name("Transaction Velocity")
            .severity(Severity.MEDIUM)
            .when(velocityCondition)
            .thenFlag("HIGH_VELOCITY", "Unusual transaction velocity detected")
            .build();

        client.createRule(velocityRule);
    }

    public TransactionResult processTransaction(Transaction transaction) throws Exception {
        PactEvent event = PactEvent.builder()
            .id("evt_" + UUID.randomUUID().toString().substring(0, 12))
            .source("transaction-service")
            .type("transaction_submitted")
            .entityId(transaction.getId())
            .entityType("Transaction")
            .jurisdiction(transaction.getJurisdiction())
            .payload(Map.of(
                "amount", transaction.getAmount(),
                "currency", transaction.getCurrency(),
                "country", transaction.getCountry(),
                "sender_id", transaction.getSenderId(),
                "recipient_id", transaction.getRecipientId(),
                "velocity_24h", transaction.getVelocity24h()
            ))
            .occurredAt(Instant.now())
            .build();

        PactDecision decision = client.submitEvent(event);

        if (decision.isDenied()) {
            return new TransactionResult(
                TransactionStatus.REJECTED,
                decision.triggeredRules().stream()
                    .map(PactDecision.TriggeredRule::message)
                    .toList()
            );
        } else if (decision.hasFlagged()) {
            return new TransactionResult(
                TransactionStatus.PENDING_REVIEW,
                decision.triggeredRules().stream()
                    .filter(r -> r.decision() == Decision.FLAG)
                    .map(PactDecision.TriggeredRule::message)
                    .toList()
            );
        } else {
            return new TransactionResult(TransactionStatus.APPROVED, List.of());
        }
    }

    public void close() {
        client.close();
    }
}
```

### Async Event Processing

```java
import java.util.concurrent.CompletableFuture;
import java.util.List;
import java.util.stream.Collectors;

public class AsyncEventProcessor {
    private final PactClient client;

    public AsyncEventProcessor(PactClient client) {
        this.client = client;
    }

    public CompletableFuture<List<PactDecision>> processEvents(List<PactEvent> events) {
        List<CompletableFuture<PactDecision>> futures = events.stream()
            .map(client::submitEventAsync)
            .toList();

        return CompletableFuture.allOf(futures.toArray(new CompletableFuture[0]))
            .thenApply(v -> futures.stream()
                .map(CompletableFuture::join)
                .collect(Collectors.toList()));
    }
}
```

## Testing

Run tests with Maven:

```bash
mvn test
```

Run a specific test class:

```bash
mvn test -Dtest=PactClientTest
```

Run tests with coverage:

```bash
mvn test jacoco:report
```

## Documentation

For comprehensive documentation, visit:

- **Main Documentation**: [https://www.arkaprotocol.com/docs](https://www.arkaprotocol.com/docs)
- **API Reference**: [https://www.arkaprotocol.com/docs/api](https://www.arkaprotocol.com/docs/api)
- **Plugin Development Guide**: [https://www.arkaprotocol.com/docs/plugins](https://www.arkaprotocol.com/docs/plugins)
- **Rule Language Reference**: [https://www.arkaprotocol.com/docs/rules](https://www.arkaprotocol.com/docs/rules)

## Requirements

- Java 17 or later
- Maven 3.8+ or Gradle 8+

## Dependencies

- Jackson (JSON processing)
- OkHttp (HTTP client)
- SLF4J (Logging)

## License

This SDK is released under the MIT License. See [LICENSE](LICENSE) for details.

## Support

- **Issues**: [GitHub Issues](https://github.com/arka-protocol/arka-sdk-java/issues)
- **Discussions**: [GitHub Discussions](https://github.com/arka-protocol/arka-sdk-java/discussions)
- **Email**: support@arkaprotocol.com
