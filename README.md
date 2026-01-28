# ARKA Protocol SDKs

Multi-language SDKs for building ARKA Protocol domain plugins.

## Available SDKs

| Language | Package | Status |
|----------|---------|--------|
| TypeScript | `@arka/plugin-sdk` | Production |
| Python | `arka-sdk` | Beta |
| Go | `github.com/arka-engine/arka-sdk-go` | Beta |
| Rust | `arka-sdk` | Beta |
| Java | `com.arka:arka-sdk` | Beta |
| .NET | `Pact.Sdk` | Beta |

## Shared Protocol Buffers

The `proto/` directory contains Protocol Buffer definitions shared across all SDKs for gRPC interoperability.

## SDK Features

All SDKs provide:

- **Plugin Interfaces** - Base classes/traits for implementing domain plugins
- **Type Definitions** - Core types (Events, Entities, Rules, Conditions, Consequences)
- **Rule Builder** - Fluent API for creating compliance rules
- **Condition Builder** - Fluent API for building rule conditions
- **Plugin Registry** - Central registry for managing plugins
- **HTTP Client** - REST client for communicating with ARKA Core
- **Validation Helpers** - Entity validation utilities
- **Serialization** - Canonical JSON serialization for blockchain anchoring

## Quick Start

### TypeScript (Original)

```typescript
import { BaseArkaPlugin, PluginManifest, rule, condition } from '@arka/plugin-sdk';

class MyPlugin extends BaseArkaPlugin {
  manifest = {
    id: 'my-plugin',
    name: 'My Plugin',
    version: '1.0.0',
    // ...
  };

  protected defaultRules = [
    rule()
      .name('High Value Check')
      .when(condition().field('amount').gt(10000).build())
      .thenFlag('HIGH_VALUE', 'Transaction exceeds threshold')
      .build()
  ];
}
```

### Python

```python
from pact_sdk import BaseArkaPlugin, PluginManifest, rule, condition

class MyPlugin(BaseArkaPlugin):
    @property
    def manifest(self) -> PluginManifest:
        return PluginManifest(
            id="my-plugin",
            name="My Plugin",
            version="1.0.0",
            # ...
        )

    @property
    def _default_rules(self):
        return [
            rule()
                .name("High Value Check")
                .when(condition().field("amount").gt(10000).build())
                .then_flag("HIGH_VALUE", "Transaction exceeds threshold")
                .build()
        ]
```

### Go

```go
import "github.com/arka-engine/arka-sdk-go/pact"

plugin := pact.NewBasePlugin(
    pact.PluginManifest{
        ID:      "my-plugin",
        Name:    "My Plugin",
        Version: "1.0.0",
    },
    entityTypes,
    []pact.ArkaRule{
        pact.NewRule().
            Name("High Value Check").
            When(pact.NewCondition().Field("amount").Gt(10000).Build()).
            ThenFlag("HIGH_VALUE", "Transaction exceeds threshold").
            MustBuild(),
    },
)
```

### Rust

```rust
use pact_sdk::{BasePlugin, PluginManifest, rule, condition};

let my_rule = rule()
    .name("High Value Check")
    .when(condition().field("amount").gt(json!(10000)).build().unwrap())
    .then_flag("HIGH_VALUE", "Transaction exceeds threshold")
    .build()?;
```

### Java

```java
import com.pact.sdk.plugin.*;
import com.pact.sdk.builder.*;
import com.pact.sdk.types.*;

ArkaRule rule = RuleBuilder.rule()
    .name("High Value Check")
    .when(ConditionBuilder.condition().field("amount").gt(10000).build())
    .thenFlag("HIGH_VALUE", "Transaction exceeds threshold")
    .build();
```

### C# (.NET)

```csharp
using Pact.Sdk;

var rule = RuleBuilder.Create()
    .Name("High Value Check")
    .When(ConditionBuilder.Create().Field("amount").Gt(10000).Build())
    .ThenFlag("HIGH_VALUE", "Transaction exceeds threshold")
    .Build();
```

## Building SDKs

### Python
```bash
cd arka-sdk-python
pip install -e ".[dev]"
```

### Go
```bash
cd arka-sdk-go
go build ./...
```

### Rust
```bash
cd arka-sdk-rust
cargo build
```

### Java
```bash
cd arka-sdk-java
mvn compile
```

### .NET
```bash
cd arka-sdk-dotnet
dotnet build
```

## Architecture

Each SDK mirrors the architecture of the TypeScript reference implementation:

```
sdk/
├── types/          # Core type definitions
├── plugin/         # Plugin interface and base implementation
├── builder/        # Rule and condition builders
├── registry/       # Plugin registry
└── client/         # HTTP/gRPC client
```

## License

MIT
