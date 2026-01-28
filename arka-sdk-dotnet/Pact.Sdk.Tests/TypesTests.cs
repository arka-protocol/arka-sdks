using System.Text.Json;
using FluentAssertions;
using Xunit;

namespace Pact.Sdk.Tests;

/// <summary>
/// Tests for type definitions and serialization/deserialization.
/// </summary>
public class TypesTests
{
    private readonly JsonSerializerOptions _jsonOptions = new()
    {
        PropertyNamingPolicy = JsonNamingPolicy.SnakeCaseLower,
        WriteIndented = false
    };

    #region Enum Serialization Tests

    [Theory]
    [InlineData(Severity.Low, "\"Low\"")]
    [InlineData(Severity.Medium, "\"Medium\"")]
    [InlineData(Severity.High, "\"High\"")]
    [InlineData(Severity.Critical, "\"Critical\"")]
    public void Severity_Serializes_AsString(Severity severity, string expected)
    {
        var json = JsonSerializer.Serialize(severity, _jsonOptions);
        json.Should().Be(expected);
    }

    [Theory]
    [InlineData("\"Low\"", Severity.Low)]
    [InlineData("\"Medium\"", Severity.Medium)]
    [InlineData("\"High\"", Severity.High)]
    [InlineData("\"Critical\"", Severity.Critical)]
    public void Severity_Deserializes_FromString(string json, Severity expected)
    {
        var severity = JsonSerializer.Deserialize<Severity>(json, _jsonOptions);
        severity.Should().Be(expected);
    }

    [Theory]
    [InlineData(Decision.Allow, "\"Allow\"")]
    [InlineData(Decision.Deny, "\"Deny\"")]
    [InlineData(Decision.Flag, "\"Flag\"")]
    public void Decision_Serializes_AsString(Decision decision, string expected)
    {
        var json = JsonSerializer.Serialize(decision, _jsonOptions);
        json.Should().Be(expected);
    }

    [Theory]
    [InlineData(DecisionStatus.Allow, "\"Allow\"")]
    [InlineData(DecisionStatus.AllowWithFlags, "\"AllowWithFlags\"")]
    [InlineData(DecisionStatus.Deny, "\"Deny\"")]
    public void DecisionStatus_Serializes_AsString(DecisionStatus status, string expected)
    {
        var json = JsonSerializer.Serialize(status, _jsonOptions);
        json.Should().Be(expected);
    }

    [Theory]
    [InlineData(EvaluationResult.Pass, "\"Pass\"")]
    [InlineData(EvaluationResult.Fail, "\"Fail\"")]
    [InlineData(EvaluationResult.Skip, "\"Skip\"")]
    [InlineData(EvaluationResult.Error, "\"Error\"")]
    public void EvaluationResult_Serializes_AsString(EvaluationResult result, string expected)
    {
        var json = JsonSerializer.Serialize(result, _jsonOptions);
        json.Should().Be(expected);
    }

    #endregion

    #region PluginManifest Tests

    [Fact]
    public void PluginManifest_Serializes_Correctly()
    {
        var manifest = new PluginManifest
        {
            Id = "test-plugin",
            Name = "Test Plugin",
            Version = "1.0.0",
            Author = "Test Author",
            Description = "A test plugin",
            EntityTypes = ["Entity1", "Entity2"],
            EventTypes = ["event_created", "event_updated"],
            Dependencies = ["dep-plugin"],
            PactCoreVersion = "1.0.0"
        };

        var json = JsonSerializer.Serialize(manifest, _jsonOptions);
        var deserialized = JsonSerializer.Deserialize<PluginManifest>(json, _jsonOptions);

        deserialized.Should().NotBeNull();
        deserialized!.Id.Should().Be("test-plugin");
        deserialized.Name.Should().Be("Test Plugin");
        deserialized.Version.Should().Be("1.0.0");
        deserialized.EntityTypes.Should().HaveCount(2);
        deserialized.EventTypes.Should().HaveCount(2);
        deserialized.Dependencies.Should().ContainSingle();
    }

    [Fact]
    public void PluginManifest_WithConfigSchema_Serializes_Correctly()
    {
        var manifest = new PluginManifest
        {
            Id = "config-plugin",
            Name = "Config Plugin",
            Version = "1.0.0",
            Author = "Test",
            Description = "Plugin with config",
            PactCoreVersion = "1.0.0",
            ConfigSchema = new Dictionary<string, object>
            {
                ["apiUrl"] = new { type = "string", required = true },
                ["maxRetries"] = new { type = "integer", default_value = 3 }
            }
        };

        var json = JsonSerializer.Serialize(manifest, _jsonOptions);
        json.Should().Contain("config_schema");
        json.Should().Contain("apiUrl");
    }

    #endregion

    #region DomainEvent Tests

    [Fact]
    public void DomainEvent_Serializes_Correctly()
    {
        var domainEvent = new DomainEvent
        {
            Type = "user_created",
            Payload = new Dictionary<string, object>
            {
                ["email"] = "test@example.com",
                ["name"] = "Test User"
            },
            EntityId = "user-123",
            Jurisdiction = "US",
            OccurredAt = DateTimeOffset.Parse("2024-01-15T10:30:00Z"),
            Metadata = new Dictionary<string, object>
            {
                ["source"] = "api"
            }
        };

        var json = JsonSerializer.Serialize(domainEvent, _jsonOptions);
        var deserialized = JsonSerializer.Deserialize<DomainEvent>(json, _jsonOptions);

        deserialized.Should().NotBeNull();
        deserialized!.Type.Should().Be("user_created");
        deserialized.EntityId.Should().Be("user-123");
        deserialized.Jurisdiction.Should().Be("US");
    }

    [Fact]
    public void DomainEvent_WithNullOptionalFields_Serializes_Correctly()
    {
        var domainEvent = new DomainEvent
        {
            Type = "simple_event"
        };

        var json = JsonSerializer.Serialize(domainEvent, _jsonOptions);
        var deserialized = JsonSerializer.Deserialize<DomainEvent>(json, _jsonOptions);

        deserialized.Should().NotBeNull();
        deserialized!.Type.Should().Be("simple_event");
        deserialized.EntityId.Should().BeNull();
        deserialized.Jurisdiction.Should().BeNull();
        deserialized.OccurredAt.Should().BeNull();
    }

    #endregion

    #region PactEvent Tests

    [Fact]
    public void PactEvent_Serializes_WithSnakeCaseProperties()
    {
        var now = DateTimeOffset.UtcNow;
        var pactEvent = new PactEvent
        {
            Id = "evt_abc123",
            Source = "test-plugin",
            Type = "user_created",
            EntityId = "user-123",
            EntityType = "User",
            Jurisdiction = "US",
            Payload = new Dictionary<string, object> { ["key"] = "value" },
            OccurredAt = now,
            ReceivedAt = now
        };

        var json = JsonSerializer.Serialize(pactEvent, _jsonOptions);

        json.Should().Contain("\"id\"");
        json.Should().Contain("\"source\"");
        json.Should().Contain("\"entity_id\"");
        json.Should().Contain("\"entity_type\"");
        json.Should().Contain("\"occurred_at\"");
        json.Should().Contain("\"received_at\"");
    }

    [Fact]
    public void PactEvent_RoundTrip_PreservesData()
    {
        var now = DateTimeOffset.Parse("2024-01-15T10:30:00Z");
        var pactEvent = new PactEvent
        {
            Id = "evt_roundtrip",
            Source = "test-source",
            Type = "test_event",
            EntityId = "entity-456",
            EntityType = "TestEntity",
            Jurisdiction = "EU",
            Payload = new Dictionary<string, object>
            {
                ["amount"] = 100,
                ["currency"] = "EUR"
            },
            OccurredAt = now,
            ReceivedAt = now,
            Metadata = new Dictionary<string, object>
            {
                ["trace_id"] = "trace-789"
            }
        };

        var json = JsonSerializer.Serialize(pactEvent, _jsonOptions);
        var deserialized = JsonSerializer.Deserialize<PactEvent>(json, _jsonOptions);

        deserialized.Should().NotBeNull();
        deserialized!.Id.Should().Be("evt_roundtrip");
        deserialized.Source.Should().Be("test-source");
        deserialized.Type.Should().Be("test_event");
        deserialized.EntityId.Should().Be("entity-456");
        deserialized.EntityType.Should().Be("TestEntity");
        deserialized.Jurisdiction.Should().Be("EU");
    }

    #endregion

    #region PactEntity Tests

    [Fact]
    public void PactEntity_Serializes_Correctly()
    {
        var now = DateTimeOffset.UtcNow;
        var entity = new PactEntity
        {
            Id = "entity-123",
            Type = "Customer",
            Data = new Dictionary<string, object>
            {
                ["name"] = "John Doe",
                ["email"] = "john@example.com"
            },
            CreatedAt = now,
            UpdatedAt = now,
            Jurisdiction = "US"
        };

        var json = JsonSerializer.Serialize(entity, _jsonOptions);
        var deserialized = JsonSerializer.Deserialize<PactEntity>(json, _jsonOptions);

        deserialized.Should().NotBeNull();
        deserialized!.Id.Should().Be("entity-123");
        deserialized.Type.Should().Be("Customer");
        deserialized.Jurisdiction.Should().Be("US");
    }

    #endregion

    #region PactEntityType Tests

    [Fact]
    public void PactEntityType_Serializes_WithSchema()
    {
        var entityType = new PactEntityType
        {
            Name = "Transaction",
            Description = "Financial transaction",
            Schema = new Dictionary<string, object>
            {
                ["amount"] = new { type = "number" },
                ["currency"] = new { type = "string" }
            },
            RequiredFields = ["amount", "currency"]
        };

        var json = JsonSerializer.Serialize(entityType, _jsonOptions);
        var deserialized = JsonSerializer.Deserialize<PactEntityType>(json, _jsonOptions);

        deserialized.Should().NotBeNull();
        deserialized!.Name.Should().Be("Transaction");
        deserialized.RequiredFields.Should().HaveCount(2);
        deserialized.RequiredFields.Should().Contain("amount");
        deserialized.RequiredFields.Should().Contain("currency");
    }

    #endregion

    #region Consequence Tests

    [Fact]
    public void Consequence_Deny_CreatesCorrectConsequence()
    {
        var consequence = Consequence.Deny("DENY_001", "Transaction denied");

        consequence.Decision.Should().Be(Decision.Deny);
        consequence.Code.Should().Be("DENY_001");
        consequence.Message.Should().Be("Transaction denied");
    }

    [Fact]
    public void Consequence_Flag_CreatesCorrectConsequence()
    {
        var consequence = Consequence.Flag("FLAG_001", "Suspicious activity");

        consequence.Decision.Should().Be(Decision.Flag);
        consequence.Code.Should().Be("FLAG_001");
        consequence.Message.Should().Be("Suspicious activity");
    }

    [Fact]
    public void Consequence_Allow_CreatesCorrectConsequence()
    {
        var consequence = Consequence.Allow("ALLOW_001", "Transaction approved");

        consequence.Decision.Should().Be(Decision.Allow);
        consequence.Code.Should().Be("ALLOW_001");
        consequence.Message.Should().Be("Transaction approved");
    }

    [Fact]
    public void Consequence_WithMetadata_Serializes_Correctly()
    {
        var consequence = new Consequence
        {
            Decision = Decision.Deny,
            Code = "DENY_002",
            Message = "Amount exceeds limit",
            Metadata = new Dictionary<string, object>
            {
                ["limit"] = 10000,
                ["actual"] = 15000
            }
        };

        var json = JsonSerializer.Serialize(consequence, _jsonOptions);
        var deserialized = JsonSerializer.Deserialize<Consequence>(json, _jsonOptions);

        deserialized.Should().NotBeNull();
        deserialized!.Metadata.Should().ContainKey("limit");
        deserialized.Metadata.Should().ContainKey("actual");
    }

    #endregion

    #region PactRule Tests

    [Fact]
    public void PactRule_Serializes_WithAllProperties()
    {
        var rule = new PactRule
        {
            Id = "rule-123",
            Name = "Amount Limit Rule",
            Description = "Denies transactions over limit",
            Jurisdiction = "US",
            Severity = Severity.High,
            Condition = new CompareCondition
            {
                Field = "payload.amount",
                Operator = "gt",
                Value = 10000
            },
            Consequence = Consequence.Deny("AMOUNT_EXCEEDED", "Transaction amount exceeds limit"),
            Tags = ["financial", "limit"],
            EffectiveFrom = DateTimeOffset.Parse("2024-01-01T00:00:00Z"),
            EffectiveTo = DateTimeOffset.Parse("2024-12-31T23:59:59Z")
        };

        var json = JsonSerializer.Serialize(rule, _jsonOptions);

        json.Should().Contain("\"id\"");
        json.Should().Contain("\"severity\"");
        json.Should().Contain("\"condition\"");
        json.Should().Contain("\"consequence\"");
        json.Should().Contain("\"effective_from\"");
        json.Should().Contain("\"effective_to\"");
    }

    #endregion

    #region RuleEvaluation Tests

    [Fact]
    public void RuleEvaluation_Serializes_Correctly()
    {
        var evaluation = new RuleEvaluation
        {
            RuleId = "rule-123",
            RuleName = "Test Rule",
            Result = EvaluationResult.Pass,
            Code = "PASS_001",
            Message = "Rule passed",
            DurationMs = 15
        };

        var json = JsonSerializer.Serialize(evaluation, _jsonOptions);
        var deserialized = JsonSerializer.Deserialize<RuleEvaluation>(json, _jsonOptions);

        deserialized.Should().NotBeNull();
        deserialized!.RuleId.Should().Be("rule-123");
        deserialized.Result.Should().Be(EvaluationResult.Pass);
        deserialized.DurationMs.Should().Be(15);
    }

    #endregion

    #region PactDecision Tests

    [Fact]
    public void PactDecision_Serializes_WithEvaluations()
    {
        var decision = new PactDecision
        {
            Id = "dec-123",
            EventId = "evt-456",
            Status = DecisionStatus.AllowWithFlags,
            RuleEvaluations =
            [
                new RuleEvaluation
                {
                    RuleId = "rule-1",
                    RuleName = "Rule 1",
                    Result = EvaluationResult.Pass
                },
                new RuleEvaluation
                {
                    RuleId = "rule-2",
                    RuleName = "Rule 2",
                    Result = EvaluationResult.Fail,
                    Code = "FLAG_001",
                    Message = "Flagged for review"
                }
            ],
            CreatedAt = DateTimeOffset.UtcNow
        };

        var json = JsonSerializer.Serialize(decision, _jsonOptions);
        var deserialized = JsonSerializer.Deserialize<PactDecision>(json, _jsonOptions);

        deserialized.Should().NotBeNull();
        deserialized!.Status.Should().Be(DecisionStatus.AllowWithFlags);
        deserialized.RuleEvaluations.Should().HaveCount(2);
    }

    #endregion

    #region ValidationResult Tests

    [Fact]
    public void ValidationResult_Success_ReturnsValidResult()
    {
        var result = ValidationResult.Success();

        result.Valid.Should().BeTrue();
        result.Errors.Should().BeEmpty();
        result.Warnings.Should().BeEmpty();
    }

    [Fact]
    public void ValidationResult_Failure_WithSingleError_ReturnsInvalidResult()
    {
        var result = ValidationResult.Failure("email", "Invalid email format", "INVALID_EMAIL");

        result.Valid.Should().BeFalse();
        result.Errors.Should().HaveCount(1);
        result.Errors[0].Field.Should().Be("email");
        result.Errors[0].Message.Should().Be("Invalid email format");
        result.Errors[0].Code.Should().Be("INVALID_EMAIL");
    }

    [Fact]
    public void ValidationResult_Failure_WithMultipleErrors_ReturnsInvalidResult()
    {
        var errors = new List<ValidationError>
        {
            new("email", "Invalid email format", "INVALID_EMAIL"),
            new("name", "Name is required", "REQUIRED_FIELD")
        };

        var result = ValidationResult.Failure(errors);

        result.Valid.Should().BeFalse();
        result.Errors.Should().HaveCount(2);
    }

    [Fact]
    public void ValidationError_Record_StoresAllProperties()
    {
        var error = new ValidationError("field", "message", "code");

        error.Field.Should().Be("field");
        error.Message.Should().Be("message");
        error.Code.Should().Be("code");
    }

    [Fact]
    public void ValidationWarning_Record_StoresAllProperties()
    {
        var warning = new ValidationWarning("field", "warning message");

        warning.Field.Should().Be("field");
        warning.Message.Should().Be("warning message");
    }

    #endregion
}
