using System.Text.Json.Serialization;

namespace Pact.Sdk;

/// <summary>
/// Base class for rule conditions.
/// </summary>
[JsonPolymorphic(TypeDiscriminatorPropertyName = "type")]
[JsonDerivedType(typeof(CompareCondition), "compare")]
[JsonDerivedType(typeof(AndCondition), "and")]
[JsonDerivedType(typeof(OrCondition), "or")]
[JsonDerivedType(typeof(NotCondition), "not")]
[JsonDerivedType(typeof(ExistsCondition), "exists")]
[JsonDerivedType(typeof(InCondition), "in")]
[JsonDerivedType(typeof(RangeCondition), "range")]
[JsonDerivedType(typeof(ExpressionCondition), "expression")]
public abstract record Condition;

/// <summary>
/// Comparison condition.
/// </summary>
public record CompareCondition : Condition
{
    [JsonPropertyName("field")]
    public required string Field { get; init; }

    [JsonPropertyName("operator")]
    public required string Operator { get; init; }

    [JsonPropertyName("value")]
    public required object Value { get; init; }
}

/// <summary>
/// Logical AND condition.
/// </summary>
public record AndCondition : Condition
{
    [JsonPropertyName("conditions")]
    public required List<Condition> Conditions { get; init; }
}

/// <summary>
/// Logical OR condition.
/// </summary>
public record OrCondition : Condition
{
    [JsonPropertyName("conditions")]
    public required List<Condition> Conditions { get; init; }
}

/// <summary>
/// Logical NOT condition.
/// </summary>
public record NotCondition : Condition
{
    [JsonPropertyName("condition")]
    public required Condition Cond { get; init; }
}

/// <summary>
/// Field existence condition.
/// </summary>
public record ExistsCondition : Condition
{
    [JsonPropertyName("field")]
    public required string Field { get; init; }
}

/// <summary>
/// Value in set condition.
/// </summary>
public record InCondition : Condition
{
    [JsonPropertyName("field")]
    public required string Field { get; init; }

    [JsonPropertyName("values")]
    public required List<object> Values { get; init; }
}

/// <summary>
/// Numeric range condition.
/// </summary>
public record RangeCondition : Condition
{
    [JsonPropertyName("field")]
    public required string Field { get; init; }

    [JsonPropertyName("min")]
    public double? Min { get; init; }

    [JsonPropertyName("max")]
    public double? Max { get; init; }

    [JsonPropertyName("min_inclusive")]
    public bool MinInclusive { get; init; } = true;

    [JsonPropertyName("max_inclusive")]
    public bool MaxInclusive { get; init; } = true;
}

/// <summary>
/// Custom expression condition.
/// </summary>
public record ExpressionCondition : Condition
{
    [JsonPropertyName("expression")]
    public required string Expression { get; init; }

    [JsonPropertyName("language")]
    public string Language { get; init; } = "cel";
}
