using System.Text.Json;
using FluentAssertions;
using Xunit;

namespace Pact.Sdk.Tests;

/// <summary>
/// Tests for condition types and polymorphic serialization.
/// </summary>
public class ConditionTests
{
    private readonly JsonSerializerOptions _jsonOptions = new()
    {
        PropertyNamingPolicy = JsonNamingPolicy.SnakeCaseLower,
        WriteIndented = false
    };

    #region CompareCondition Tests

    [Fact]
    public void CompareCondition_Serializes_WithTypeDiscriminator()
    {
        Condition condition = new CompareCondition
        {
            Field = "payload.amount",
            Operator = "gt",
            Value = 1000
        };

        var json = JsonSerializer.Serialize(condition, _jsonOptions);

        json.Should().Contain("\"type\":\"compare\"");
        json.Should().Contain("\"field\"");
        json.Should().Contain("\"operator\"");
        json.Should().Contain("\"value\"");
    }

    [Fact]
    public void CompareCondition_Deserializes_Polymorphically()
    {
        var json = """{"type":"compare","field":"amount","operator":"eq","value":500}""";

        var condition = JsonSerializer.Deserialize<Condition>(json, _jsonOptions);

        condition.Should().BeOfType<CompareCondition>();
        var compare = (CompareCondition)condition!;
        compare.Field.Should().Be("amount");
        compare.Operator.Should().Be("eq");
    }

    [Theory]
    [InlineData("eq")]
    [InlineData("ne")]
    [InlineData("gt")]
    [InlineData("gte")]
    [InlineData("lt")]
    [InlineData("lte")]
    [InlineData("contains")]
    [InlineData("regex")]
    public void CompareCondition_SupportsAllOperators(string op)
    {
        var condition = new CompareCondition
        {
            Field = "test_field",
            Operator = op,
            Value = "test_value"
        };

        var json = JsonSerializer.Serialize<Condition>(condition, _jsonOptions);
        var deserialized = JsonSerializer.Deserialize<Condition>(json, _jsonOptions);

        deserialized.Should().BeOfType<CompareCondition>();
        ((CompareCondition)deserialized!).Operator.Should().Be(op);
    }

    #endregion

    #region AndCondition Tests

    [Fact]
    public void AndCondition_Serializes_WithNestedConditions()
    {
        Condition condition = new AndCondition
        {
            Conditions =
            [
                new CompareCondition { Field = "amount", Operator = "gt", Value = 100 },
                new CompareCondition { Field = "currency", Operator = "eq", Value = "USD" }
            ]
        };

        var json = JsonSerializer.Serialize(condition, _jsonOptions);

        json.Should().Contain("\"type\":\"and\"");
        json.Should().Contain("\"conditions\"");
    }

    [Fact]
    public void AndCondition_Deserializes_WithNestedConditions()
    {
        var json = """
        {
            "type": "and",
            "conditions": [
                {"type": "compare", "field": "a", "operator": "eq", "value": 1},
                {"type": "compare", "field": "b", "operator": "eq", "value": 2}
            ]
        }
        """;

        var condition = JsonSerializer.Deserialize<Condition>(json, _jsonOptions);

        condition.Should().BeOfType<AndCondition>();
        var andCondition = (AndCondition)condition!;
        andCondition.Conditions.Should().HaveCount(2);
        andCondition.Conditions[0].Should().BeOfType<CompareCondition>();
        andCondition.Conditions[1].Should().BeOfType<CompareCondition>();
    }

    #endregion

    #region OrCondition Tests

    [Fact]
    public void OrCondition_Serializes_WithNestedConditions()
    {
        Condition condition = new OrCondition
        {
            Conditions =
            [
                new CompareCondition { Field = "status", Operator = "eq", Value = "approved" },
                new CompareCondition { Field = "status", Operator = "eq", Value = "pending" }
            ]
        };

        var json = JsonSerializer.Serialize(condition, _jsonOptions);

        json.Should().Contain("\"type\":\"or\"");
        json.Should().Contain("\"conditions\"");
    }

    [Fact]
    public void OrCondition_Deserializes_Polymorphically()
    {
        var json = """
        {
            "type": "or",
            "conditions": [
                {"type": "compare", "field": "x", "operator": "eq", "value": 1},
                {"type": "exists", "field": "y"}
            ]
        }
        """;

        var condition = JsonSerializer.Deserialize<Condition>(json, _jsonOptions);

        condition.Should().BeOfType<OrCondition>();
        var orCondition = (OrCondition)condition!;
        orCondition.Conditions.Should().HaveCount(2);
        orCondition.Conditions[0].Should().BeOfType<CompareCondition>();
        orCondition.Conditions[1].Should().BeOfType<ExistsCondition>();
    }

    #endregion

    #region NotCondition Tests

    [Fact]
    public void NotCondition_Serializes_WithInnerCondition()
    {
        Condition condition = new NotCondition
        {
            Cond = new CompareCondition
            {
                Field = "blocked",
                Operator = "eq",
                Value = true
            }
        };

        var json = JsonSerializer.Serialize(condition, _jsonOptions);

        json.Should().Contain("\"type\":\"not\"");
        json.Should().Contain("\"condition\"");
    }

    [Fact]
    public void NotCondition_Deserializes_Polymorphically()
    {
        var json = """
        {
            "type": "not",
            "condition": {"type": "compare", "field": "active", "operator": "eq", "value": false}
        }
        """;

        var condition = JsonSerializer.Deserialize<Condition>(json, _jsonOptions);

        condition.Should().BeOfType<NotCondition>();
        var notCondition = (NotCondition)condition!;
        notCondition.Cond.Should().BeOfType<CompareCondition>();
    }

    #endregion

    #region ExistsCondition Tests

    [Fact]
    public void ExistsCondition_Serializes_Correctly()
    {
        Condition condition = new ExistsCondition
        {
            Field = "metadata.custom_field"
        };

        var json = JsonSerializer.Serialize(condition, _jsonOptions);

        json.Should().Contain("\"type\":\"exists\"");
        json.Should().Contain("\"field\":\"metadata.custom_field\"");
    }

    [Fact]
    public void ExistsCondition_Deserializes_Correctly()
    {
        var json = """{"type":"exists","field":"optional_field"}""";

        var condition = JsonSerializer.Deserialize<Condition>(json, _jsonOptions);

        condition.Should().BeOfType<ExistsCondition>();
        ((ExistsCondition)condition!).Field.Should().Be("optional_field");
    }

    #endregion

    #region InCondition Tests

    [Fact]
    public void InCondition_Serializes_WithValues()
    {
        Condition condition = new InCondition
        {
            Field = "country",
            Values = ["US", "CA", "MX"]
        };

        var json = JsonSerializer.Serialize(condition, _jsonOptions);

        json.Should().Contain("\"type\":\"in\"");
        json.Should().Contain("\"field\":\"country\"");
        json.Should().Contain("\"values\"");
    }

    [Fact]
    public void InCondition_Deserializes_WithMixedValues()
    {
        var json = """{"type":"in","field":"status","values":["active","pending",1,2]}""";

        var condition = JsonSerializer.Deserialize<Condition>(json, _jsonOptions);

        condition.Should().BeOfType<InCondition>();
        var inCondition = (InCondition)condition!;
        inCondition.Field.Should().Be("status");
        inCondition.Values.Should().HaveCount(4);
    }

    #endregion

    #region RangeCondition Tests

    [Fact]
    public void RangeCondition_Serializes_WithMinAndMax()
    {
        Condition condition = new RangeCondition
        {
            Field = "age",
            Min = 18,
            Max = 65,
            MinInclusive = true,
            MaxInclusive = false
        };

        var json = JsonSerializer.Serialize(condition, _jsonOptions);

        json.Should().Contain("\"type\":\"range\"");
        json.Should().Contain("\"min\":18");
        json.Should().Contain("\"max\":65");
        json.Should().Contain("\"min_inclusive\":true");
        json.Should().Contain("\"max_inclusive\":false");
    }

    [Fact]
    public void RangeCondition_Deserializes_WithDefaults()
    {
        var json = """{"type":"range","field":"score","min":0,"max":100}""";

        var condition = JsonSerializer.Deserialize<Condition>(json, _jsonOptions);

        condition.Should().BeOfType<RangeCondition>();
        var rangeCondition = (RangeCondition)condition!;
        rangeCondition.Min.Should().Be(0);
        rangeCondition.Max.Should().Be(100);
        rangeCondition.MinInclusive.Should().BeTrue();
        rangeCondition.MaxInclusive.Should().BeTrue();
    }

    [Fact]
    public void RangeCondition_WithOnlyMin_Serializes_Correctly()
    {
        Condition condition = new RangeCondition
        {
            Field = "amount",
            Min = 1000
        };

        var json = JsonSerializer.Serialize(condition, _jsonOptions);
        var deserialized = JsonSerializer.Deserialize<Condition>(json, _jsonOptions);

        deserialized.Should().BeOfType<RangeCondition>();
        var range = (RangeCondition)deserialized!;
        range.Min.Should().Be(1000);
        range.Max.Should().BeNull();
    }

    [Fact]
    public void RangeCondition_WithOnlyMax_Serializes_Correctly()
    {
        Condition condition = new RangeCondition
        {
            Field = "discount",
            Max = 50
        };

        var json = JsonSerializer.Serialize(condition, _jsonOptions);
        var deserialized = JsonSerializer.Deserialize<Condition>(json, _jsonOptions);

        deserialized.Should().BeOfType<RangeCondition>();
        var range = (RangeCondition)deserialized!;
        range.Min.Should().BeNull();
        range.Max.Should().Be(50);
    }

    #endregion

    #region ExpressionCondition Tests

    [Fact]
    public void ExpressionCondition_Serializes_WithDefaults()
    {
        Condition condition = new ExpressionCondition
        {
            Expression = "payload.amount > 1000 && payload.currency == 'USD'"
        };

        var json = JsonSerializer.Serialize(condition, _jsonOptions);

        json.Should().Contain("\"type\":\"expression\"");
        json.Should().Contain("\"expression\"");
        json.Should().Contain("\"language\":\"cel\"");
    }

    [Fact]
    public void ExpressionCondition_Serializes_WithCustomLanguage()
    {
        Condition condition = new ExpressionCondition
        {
            Expression = "return event.amount > threshold",
            Language = "javascript"
        };

        var json = JsonSerializer.Serialize(condition, _jsonOptions);
        var deserialized = JsonSerializer.Deserialize<Condition>(json, _jsonOptions);

        deserialized.Should().BeOfType<ExpressionCondition>();
        var expr = (ExpressionCondition)deserialized!;
        expr.Language.Should().Be("javascript");
    }

    #endregion

    #region Complex Nested Conditions Tests

    [Fact]
    public void ComplexNestedCondition_Serializes_Correctly()
    {
        // (amount > 1000 AND currency == "USD") OR (amount > 500 AND VIP == true)
        Condition condition = new OrCondition
        {
            Conditions =
            [
                new AndCondition
                {
                    Conditions =
                    [
                        new CompareCondition { Field = "amount", Operator = "gt", Value = 1000 },
                        new CompareCondition { Field = "currency", Operator = "eq", Value = "USD" }
                    ]
                },
                new AndCondition
                {
                    Conditions =
                    [
                        new CompareCondition { Field = "amount", Operator = "gt", Value = 500 },
                        new CompareCondition { Field = "vip", Operator = "eq", Value = true }
                    ]
                }
            ]
        };

        var json = JsonSerializer.Serialize(condition, _jsonOptions);
        var deserialized = JsonSerializer.Deserialize<Condition>(json, _jsonOptions);

        deserialized.Should().BeOfType<OrCondition>();
        var orCondition = (OrCondition)deserialized!;
        orCondition.Conditions.Should().HaveCount(2);
        orCondition.Conditions[0].Should().BeOfType<AndCondition>();
        orCondition.Conditions[1].Should().BeOfType<AndCondition>();
    }

    [Fact]
    public void DeeplyNestedCondition_RoundTrips_Correctly()
    {
        // NOT (field EXISTS AND field IN [1, 2, 3])
        Condition condition = new NotCondition
        {
            Cond = new AndCondition
            {
                Conditions =
                [
                    new ExistsCondition { Field = "data.optional" },
                    new InCondition { Field = "data.optional", Values = [1, 2, 3] }
                ]
            }
        };

        var json = JsonSerializer.Serialize(condition, _jsonOptions);
        var deserialized = JsonSerializer.Deserialize<Condition>(json, _jsonOptions);

        deserialized.Should().BeOfType<NotCondition>();
        var notCond = (NotCondition)deserialized!;
        notCond.Cond.Should().BeOfType<AndCondition>();
        var andCond = (AndCondition)notCond.Cond;
        andCond.Conditions.Should().HaveCount(2);
    }

    #endregion
}
