namespace Pact.Sdk;

/// <summary>
/// Fluent builder for creating conditions.
/// </summary>
public class ConditionBuilder
{
    private string? _field;
    private Condition? _condition;

    public ConditionBuilder Field(string path)
    {
        _field = path;
        return this;
    }

    public ConditionBuilder Eq(object value)
    {
        _condition = new CompareCondition { Field = _field!, Operator = "eq", Value = value };
        return this;
    }

    public ConditionBuilder Ne(object value)
    {
        _condition = new CompareCondition { Field = _field!, Operator = "ne", Value = value };
        return this;
    }

    public ConditionBuilder Gt(object value)
    {
        _condition = new CompareCondition { Field = _field!, Operator = "gt", Value = value };
        return this;
    }

    public ConditionBuilder Gte(object value)
    {
        _condition = new CompareCondition { Field = _field!, Operator = "gte", Value = value };
        return this;
    }

    public ConditionBuilder Lt(object value)
    {
        _condition = new CompareCondition { Field = _field!, Operator = "lt", Value = value };
        return this;
    }

    public ConditionBuilder Lte(object value)
    {
        _condition = new CompareCondition { Field = _field!, Operator = "lte", Value = value };
        return this;
    }

    public ConditionBuilder Contains(string value)
    {
        _condition = new CompareCondition { Field = _field!, Operator = "contains", Value = value };
        return this;
    }

    public ConditionBuilder Matches(string pattern)
    {
        _condition = new CompareCondition { Field = _field!, Operator = "regex", Value = pattern };
        return this;
    }

    public ConditionBuilder Exists()
    {
        _condition = new ExistsCondition { Field = _field! };
        return this;
    }

    public ConditionBuilder In(params object[] values)
    {
        _condition = new InCondition { Field = _field!, Values = values.ToList() };
        return this;
    }

    public ConditionBuilder Between(double? min, double? max)
    {
        _condition = new RangeCondition { Field = _field!, Min = min, Max = max };
        return this;
    }

    public ConditionBuilder Expression(string expr, string language = "cel")
    {
        _condition = new ExpressionCondition { Expression = expr, Language = language };
        return this;
    }

    public Condition Build() => _condition!;

    public static ConditionBuilder Create() => new();

    public static Condition And(params Condition[] conditions) =>
        new AndCondition { Conditions = conditions.ToList() };

    public static Condition Or(params Condition[] conditions) =>
        new OrCondition { Conditions = conditions.ToList() };

    public static Condition Not(Condition condition) =>
        new NotCondition { Cond = condition };
}

/// <summary>
/// Fluent builder for creating rules.
/// </summary>
public class RuleBuilder
{
    private string? _id;
    private string? _name;
    private string? _description;
    private string? _jurisdiction;
    private Severity _severity = Severity.Medium;
    private Condition? _condition;
    private Consequence? _consequence;
    private readonly List<string> _tags = [];
    private DateTimeOffset? _effectiveFrom;
    private DateTimeOffset? _effectiveTo;
    private readonly Dictionary<string, object> _metadata = new();

    public RuleBuilder Id(string id) { _id = id; return this; }
    public RuleBuilder Name(string name) { _name = name; return this; }
    public RuleBuilder Description(string description) { _description = description; return this; }
    public RuleBuilder Jurisdiction(string jurisdiction) { _jurisdiction = jurisdiction; return this; }
    public RuleBuilder WithSeverity(Severity severity) { _severity = severity; return this; }

    public RuleBuilder When(Condition condition)
    {
        _condition = condition;
        return this;
    }

    public RuleBuilder WhenField(string field, string op, object value)
    {
        _condition = new CompareCondition { Field = field, Operator = op, Value = value };
        return this;
    }

    public RuleBuilder ThenDeny(string code, string message)
    {
        _consequence = Consequence.Deny(code, message);
        return this;
    }

    public RuleBuilder ThenFlag(string code, string message)
    {
        _consequence = Consequence.Flag(code, message);
        return this;
    }

    public RuleBuilder ThenAllow(string code, string message)
    {
        _consequence = Consequence.Allow(code, message);
        return this;
    }

    public RuleBuilder WithConsequence(Consequence consequence)
    {
        _consequence = consequence;
        return this;
    }

    public RuleBuilder Tags(params string[] tags)
    {
        _tags.AddRange(tags);
        return this;
    }

    public RuleBuilder EffectiveFrom(DateTimeOffset date) { _effectiveFrom = date; return this; }
    public RuleBuilder EffectiveTo(DateTimeOffset date) { _effectiveTo = date; return this; }

    public RuleBuilder Metadata(string key, object value)
    {
        _metadata[key] = value;
        return this;
    }

    public PactRule Build()
    {
        if (string.IsNullOrEmpty(_name))
            throw new InvalidOperationException("Rule name is required");
        if (_condition == null)
            throw new InvalidOperationException("Rule condition is required");
        if (_consequence == null)
            throw new InvalidOperationException("Rule consequence is required");

        return new PactRule
        {
            Id = _id ?? $"rule_{Guid.NewGuid().ToString()[..12]}",
            Name = _name,
            Description = _description ?? _name,
            Jurisdiction = _jurisdiction,
            Severity = _severity,
            Condition = _condition,
            Consequence = _consequence,
            Tags = _tags,
            EffectiveFrom = _effectiveFrom,
            EffectiveTo = _effectiveTo,
            Metadata = new Dictionary<string, object>(_metadata)
        };
    }

    public static RuleBuilder Create() => new();
}
