using System.Text.Json.Serialization;

namespace Pact.Sdk;

/// <summary>
/// Rule severity levels.
/// </summary>
[JsonConverter(typeof(JsonStringEnumConverter))]
public enum Severity
{
    Low,
    Medium,
    High,
    Critical
}

/// <summary>
/// Rule consequence decisions.
/// </summary>
[JsonConverter(typeof(JsonStringEnumConverter))]
public enum Decision
{
    Allow,
    Deny,
    Flag
}

/// <summary>
/// Overall decision status.
/// </summary>
[JsonConverter(typeof(JsonStringEnumConverter))]
public enum DecisionStatus
{
    Allow,
    AllowWithFlags,
    Deny
}

/// <summary>
/// Rule evaluation result.
/// </summary>
[JsonConverter(typeof(JsonStringEnumConverter))]
public enum EvaluationResult
{
    Pass,
    Fail,
    Skip,
    Error
}

/// <summary>
/// Plugin manifest describing capabilities.
/// </summary>
public record PluginManifest
{
    [JsonPropertyName("id")]
    public required string Id { get; init; }

    [JsonPropertyName("name")]
    public required string Name { get; init; }

    [JsonPropertyName("version")]
    public required string Version { get; init; }

    [JsonPropertyName("author")]
    public required string Author { get; init; }

    [JsonPropertyName("description")]
    public required string Description { get; init; }

    [JsonPropertyName("entity_types")]
    public List<string> EntityTypes { get; init; } = [];

    [JsonPropertyName("event_types")]
    public List<string> EventTypes { get; init; } = [];

    [JsonPropertyName("dependencies")]
    public List<string> Dependencies { get; init; } = [];

    [JsonPropertyName("pact_core_version")]
    public required string PactCoreVersion { get; init; }

    [JsonPropertyName("config_schema")]
    public Dictionary<string, object>? ConfigSchema { get; init; }
}

/// <summary>
/// Domain-specific event before conversion.
/// </summary>
public record DomainEvent
{
    [JsonPropertyName("type")]
    public required string Type { get; init; }

    [JsonPropertyName("payload")]
    public Dictionary<string, object> Payload { get; init; } = [];

    [JsonPropertyName("entity_id")]
    public string? EntityId { get; init; }

    [JsonPropertyName("jurisdiction")]
    public string? Jurisdiction { get; init; }

    [JsonPropertyName("occurred_at")]
    public DateTimeOffset? OccurredAt { get; init; }

    [JsonPropertyName("metadata")]
    public Dictionary<string, object> Metadata { get; init; } = [];
}

/// <summary>
/// Canonical PACT event format.
/// </summary>
public record PactEvent
{
    [JsonPropertyName("id")]
    public required string Id { get; init; }

    [JsonPropertyName("source")]
    public required string Source { get; init; }

    [JsonPropertyName("type")]
    public required string Type { get; init; }

    [JsonPropertyName("entity_id")]
    public string? EntityId { get; init; }

    [JsonPropertyName("entity_type")]
    public string? EntityType { get; init; }

    [JsonPropertyName("jurisdiction")]
    public string? Jurisdiction { get; init; }

    [JsonPropertyName("payload")]
    public Dictionary<string, object> Payload { get; init; } = [];

    [JsonPropertyName("occurred_at")]
    public required DateTimeOffset OccurredAt { get; init; }

    [JsonPropertyName("received_at")]
    public required DateTimeOffset ReceivedAt { get; init; }

    [JsonPropertyName("metadata")]
    public Dictionary<string, object> Metadata { get; init; } = [];
}

/// <summary>
/// PACT entity representation.
/// </summary>
public record PactEntity
{
    [JsonPropertyName("id")]
    public required string Id { get; init; }

    [JsonPropertyName("type")]
    public required string Type { get; init; }

    [JsonPropertyName("data")]
    public Dictionary<string, object> Data { get; init; } = [];

    [JsonPropertyName("created_at")]
    public required DateTimeOffset CreatedAt { get; init; }

    [JsonPropertyName("updated_at")]
    public required DateTimeOffset UpdatedAt { get; init; }

    [JsonPropertyName("jurisdiction")]
    public string? Jurisdiction { get; init; }

    [JsonPropertyName("metadata")]
    public Dictionary<string, object> Metadata { get; init; } = [];
}

/// <summary>
/// Entity type definition with schema.
/// </summary>
public record PactEntityType
{
    [JsonPropertyName("name")]
    public required string Name { get; init; }

    [JsonPropertyName("description")]
    public string Description { get; init; } = "";

    [JsonPropertyName("schema")]
    public required Dictionary<string, object> Schema { get; init; }

    [JsonPropertyName("required_fields")]
    public List<string> RequiredFields { get; init; } = [];
}

/// <summary>
/// Rule consequence definition.
/// </summary>
public record Consequence
{
    [JsonPropertyName("decision")]
    public required Decision Decision { get; init; }

    [JsonPropertyName("code")]
    public required string Code { get; init; }

    [JsonPropertyName("message")]
    public required string Message { get; init; }

    [JsonPropertyName("metadata")]
    public Dictionary<string, object> Metadata { get; init; } = [];

    public static Consequence Deny(string code, string message) =>
        new() { Decision = Decision.Deny, Code = code, Message = message };

    public static Consequence Flag(string code, string message) =>
        new() { Decision = Decision.Flag, Code = code, Message = message };

    public static Consequence Allow(string code, string message) =>
        new() { Decision = Decision.Allow, Code = code, Message = message };
}

/// <summary>
/// PACT rule definition.
/// </summary>
public record PactRule
{
    [JsonPropertyName("id")]
    public required string Id { get; init; }

    [JsonPropertyName("name")]
    public required string Name { get; init; }

    [JsonPropertyName("description")]
    public string Description { get; init; } = "";

    [JsonPropertyName("jurisdiction")]
    public string? Jurisdiction { get; init; }

    [JsonPropertyName("severity")]
    public Severity Severity { get; init; } = Severity.Medium;

    [JsonPropertyName("condition")]
    public required Condition Condition { get; init; }

    [JsonPropertyName("consequence")]
    public required Consequence Consequence { get; init; }

    [JsonPropertyName("tags")]
    public List<string> Tags { get; init; } = [];

    [JsonPropertyName("effective_from")]
    public DateTimeOffset? EffectiveFrom { get; init; }

    [JsonPropertyName("effective_to")]
    public DateTimeOffset? EffectiveTo { get; init; }

    [JsonPropertyName("metadata")]
    public Dictionary<string, object> Metadata { get; init; } = [];
}

/// <summary>
/// Result of evaluating a rule.
/// </summary>
public record RuleEvaluation
{
    [JsonPropertyName("rule_id")]
    public required string RuleId { get; init; }

    [JsonPropertyName("rule_name")]
    public required string RuleName { get; init; }

    [JsonPropertyName("result")]
    public required EvaluationResult Result { get; init; }

    [JsonPropertyName("code")]
    public string? Code { get; init; }

    [JsonPropertyName("message")]
    public string? Message { get; init; }

    [JsonPropertyName("duration_ms")]
    public long DurationMs { get; init; }
}

/// <summary>
/// Decision from rule evaluation.
/// </summary>
public record PactDecision
{
    [JsonPropertyName("id")]
    public required string Id { get; init; }

    [JsonPropertyName("event_id")]
    public required string EventId { get; init; }

    [JsonPropertyName("status")]
    public required DecisionStatus Status { get; init; }

    [JsonPropertyName("rule_evaluations")]
    public List<RuleEvaluation> RuleEvaluations { get; init; } = [];

    [JsonPropertyName("created_at")]
    public required DateTimeOffset CreatedAt { get; init; }

    [JsonPropertyName("metadata")]
    public Dictionary<string, object> Metadata { get; init; } = [];
}

/// <summary>
/// Validation error details.
/// </summary>
public record ValidationError(string Field, string Message, string Code);

/// <summary>
/// Validation warning details.
/// </summary>
public record ValidationWarning(string Field, string Message);

/// <summary>
/// Result of data validation.
/// </summary>
public record ValidationResult
{
    public bool Valid { get; init; }
    public List<ValidationError> Errors { get; init; } = [];
    public List<ValidationWarning> Warnings { get; init; } = [];

    public static ValidationResult Success() =>
        new() { Valid = true };

    public static ValidationResult Failure(List<ValidationError> errors) =>
        new() { Valid = false, Errors = errors };

    public static ValidationResult Failure(string field, string message, string code) =>
        Failure([new ValidationError(field, message, code)]);
}
