using System.Text;
using System.Text.Json;

namespace Pact.Sdk;

/// <summary>
/// Plugin lifecycle hooks.
/// </summary>
public class PluginHooks
{
    public Func<Task>? OnLoad { get; set; }
    public Func<Task>? OnUnload { get; set; }
    public Func<PactEvent, Task<PactEvent>>? BeforeEventProcess { get; set; }
    public Func<PactEvent, PactDecision, Task>? AfterDecision { get; set; }
    public Func<List<PactRule>, Task>? OnRulesUpdated { get; set; }
}

/// <summary>
/// Interface that all PACT domain plugins must implement.
/// </summary>
public interface IDomainPlugin
{
    /// <summary>Returns the plugin manifest.</summary>
    PluginManifest Manifest { get; }

    /// <summary>Returns optional lifecycle hooks.</summary>
    PluginHooks? Hooks => null;

    /// <summary>Returns entity types defined by this plugin.</summary>
    IReadOnlyList<PactEntityType> GetEntityTypes();

    /// <summary>Returns default rules for this domain.</summary>
    IReadOnlyList<PactRule> GetDefaultRules();

    /// <summary>Converts a domain event to canonical format.</summary>
    PactEvent MapToCanonicalEvent(DomainEvent domainEvent);

    /// <summary>Validates domain-specific data.</summary>
    ValidationResult ValidateDomainData(string entityType, Dictionary<string, object> data);

    /// <summary>Returns context for rule evaluation.</summary>
    Dictionary<string, object> GetEvaluationContext(PactEvent evt, PactEntity? entity) => new();

    /// <summary>Serializes data for blockchain.</summary>
    byte[] SerializeForChain(object data) =>
        Encoding.UTF8.GetBytes(JsonSerializer.Serialize(data, CanonicalJsonOptions.Default));

    /// <summary>Deserializes data from blockchain.</summary>
    object? DeserializeFromChain(byte[] data) =>
        JsonSerializer.Deserialize<object>(Encoding.UTF8.GetString(data));
}

/// <summary>
/// Base implementation of IDomainPlugin.
/// </summary>
public abstract class BasePlugin : IDomainPlugin
{
    private readonly PluginManifest _manifest;
    private readonly List<PactEntityType> _entityTypes;
    private readonly List<PactRule> _defaultRules;

    public PluginHooks? Hooks { get; set; }

    protected BasePlugin(
        PluginManifest manifest,
        List<PactEntityType> entityTypes,
        List<PactRule> defaultRules)
    {
        _manifest = manifest;
        _entityTypes = entityTypes;
        _defaultRules = defaultRules;
    }

    public PluginManifest Manifest => _manifest;

    public IReadOnlyList<PactEntityType> GetEntityTypes() => _entityTypes.AsReadOnly();

    public IReadOnlyList<PactRule> GetDefaultRules() => _defaultRules.AsReadOnly();

    public virtual PactEvent MapToCanonicalEvent(DomainEvent domainEvent)
    {
        var now = DateTimeOffset.UtcNow;
        var occurredAt = domainEvent.OccurredAt ?? now;

        return new PactEvent
        {
            Id = $"evt_{Guid.NewGuid().ToString()[..12]}",
            Source = _manifest.Id,
            Type = domainEvent.Type,
            EntityId = domainEvent.EntityId,
            EntityType = InferEntityType(domainEvent),
            Jurisdiction = domainEvent.Jurisdiction,
            Payload = domainEvent.Payload,
            OccurredAt = occurredAt,
            ReceivedAt = now,
            Metadata = domainEvent.Metadata
        };
    }

    protected virtual string? InferEntityType(DomainEvent domainEvent)
    {
        var parts = domainEvent.Type.Split('_');
        if (parts.Length >= 2)
        {
            var entityName = parts[0].ToLowerInvariant();
            return char.ToUpperInvariant(entityName[0]) + entityName[1..];
        }
        return null;
    }

    public virtual ValidationResult ValidateDomainData(string entityType, Dictionary<string, object> data)
    {
        var entityTypeDef = _entityTypes.FirstOrDefault(et => et.Name == entityType);

        if (entityTypeDef == null)
        {
            return ValidationResult.Failure("entity_type",
                $"Unknown entity type: {entityType}", "UNKNOWN_ENTITY_TYPE");
        }

        var errors = new List<ValidationError>();
        foreach (var field in entityTypeDef.RequiredFields)
        {
            if (!data.ContainsKey(field) || data[field] == null)
            {
                errors.Add(new ValidationError(field,
                    $"Required field '{field}' is missing", "REQUIRED_FIELD_MISSING"));
            }
        }

        return errors.Count == 0
            ? ValidationResult.Success()
            : ValidationResult.Failure(errors);
    }

    public Dictionary<string, object> GetEvaluationContext(PactEvent evt, PactEntity? entity) => new();

    public string CreateRuleId() => $"rule_{Guid.NewGuid().ToString()[..12]}";

    public PactRule CreateRule(string name, Condition condition, Consequence consequence)
    {
        return new PactRule
        {
            Id = CreateRuleId(),
            Name = name,
            Description = name,
            Severity = Severity.Medium,
            Condition = condition,
            Consequence = consequence,
            Tags = [_manifest.Id],
            Metadata = new Dictionary<string, object>
            {
                ["plugin_id"] = _manifest.Id,
                ["plugin_version"] = _manifest.Version
            }
        };
    }
}

/// <summary>
/// JSON serialization options for canonical format.
/// </summary>
internal static class CanonicalJsonOptions
{
    public static readonly JsonSerializerOptions Default = new()
    {
        WriteIndented = false,
        PropertyNamingPolicy = JsonNamingPolicy.SnakeCaseLower
    };
}
