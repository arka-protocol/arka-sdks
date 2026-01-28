using FluentAssertions;
using Xunit;

namespace Pact.Sdk.Tests;

/// <summary>
/// Tests for fluent builder classes.
/// </summary>
public class BuilderTests
{
    #region ConditionBuilder Tests

    [Fact]
    public void ConditionBuilder_Create_ReturnsNewBuilder()
    {
        var builder = ConditionBuilder.Create();

        builder.Should().NotBeNull();
    }

    [Fact]
    public void ConditionBuilder_Field_SetsField()
    {
        var condition = ConditionBuilder.Create()
            .Field("payload.amount")
            .Eq(1000)
            .Build();

        condition.Should().BeOfType<CompareCondition>();
        var compare = (CompareCondition)condition;
        compare.Field.Should().Be("payload.amount");
    }

    [Fact]
    public void ConditionBuilder_Eq_CreatesCompareCondition()
    {
        var condition = ConditionBuilder.Create()
            .Field("status")
            .Eq("active")
            .Build();

        condition.Should().BeOfType<CompareCondition>();
        var compare = (CompareCondition)condition;
        compare.Operator.Should().Be("eq");
        compare.Value.Should().Be("active");
    }

    [Fact]
    public void ConditionBuilder_Ne_CreatesCompareCondition()
    {
        var condition = ConditionBuilder.Create()
            .Field("status")
            .Ne("deleted")
            .Build();

        condition.Should().BeOfType<CompareCondition>();
        var compare = (CompareCondition)condition;
        compare.Operator.Should().Be("ne");
    }

    [Fact]
    public void ConditionBuilder_Gt_CreatesCompareCondition()
    {
        var condition = ConditionBuilder.Create()
            .Field("amount")
            .Gt(100)
            .Build();

        condition.Should().BeOfType<CompareCondition>();
        var compare = (CompareCondition)condition;
        compare.Operator.Should().Be("gt");
    }

    [Fact]
    public void ConditionBuilder_Gte_CreatesCompareCondition()
    {
        var condition = ConditionBuilder.Create()
            .Field("amount")
            .Gte(100)
            .Build();

        condition.Should().BeOfType<CompareCondition>();
        var compare = (CompareCondition)condition;
        compare.Operator.Should().Be("gte");
    }

    [Fact]
    public void ConditionBuilder_Lt_CreatesCompareCondition()
    {
        var condition = ConditionBuilder.Create()
            .Field("amount")
            .Lt(100)
            .Build();

        condition.Should().BeOfType<CompareCondition>();
        var compare = (CompareCondition)condition;
        compare.Operator.Should().Be("lt");
    }

    [Fact]
    public void ConditionBuilder_Lte_CreatesCompareCondition()
    {
        var condition = ConditionBuilder.Create()
            .Field("amount")
            .Lte(100)
            .Build();

        condition.Should().BeOfType<CompareCondition>();
        var compare = (CompareCondition)condition;
        compare.Operator.Should().Be("lte");
    }

    [Fact]
    public void ConditionBuilder_Contains_CreatesCompareCondition()
    {
        var condition = ConditionBuilder.Create()
            .Field("email")
            .Contains("@example.com")
            .Build();

        condition.Should().BeOfType<CompareCondition>();
        var compare = (CompareCondition)condition;
        compare.Operator.Should().Be("contains");
    }

    [Fact]
    public void ConditionBuilder_Matches_CreatesCompareCondition()
    {
        var condition = ConditionBuilder.Create()
            .Field("phone")
            .Matches(@"^\+1\d{10}$")
            .Build();

        condition.Should().BeOfType<CompareCondition>();
        var compare = (CompareCondition)condition;
        compare.Operator.Should().Be("regex");
    }

    [Fact]
    public void ConditionBuilder_Exists_CreatesExistsCondition()
    {
        var condition = ConditionBuilder.Create()
            .Field("metadata.custom")
            .Exists()
            .Build();

        condition.Should().BeOfType<ExistsCondition>();
        var exists = (ExistsCondition)condition;
        exists.Field.Should().Be("metadata.custom");
    }

    [Fact]
    public void ConditionBuilder_In_CreatesInCondition()
    {
        var condition = ConditionBuilder.Create()
            .Field("country")
            .In("US", "CA", "MX")
            .Build();

        condition.Should().BeOfType<InCondition>();
        var inCond = (InCondition)condition;
        inCond.Field.Should().Be("country");
        inCond.Values.Should().HaveCount(3);
        inCond.Values.Should().Contain("US");
        inCond.Values.Should().Contain("CA");
        inCond.Values.Should().Contain("MX");
    }

    [Fact]
    public void ConditionBuilder_Between_CreatesRangeCondition()
    {
        var condition = ConditionBuilder.Create()
            .Field("age")
            .Between(18, 65)
            .Build();

        condition.Should().BeOfType<RangeCondition>();
        var range = (RangeCondition)condition;
        range.Field.Should().Be("age");
        range.Min.Should().Be(18);
        range.Max.Should().Be(65);
    }

    [Fact]
    public void ConditionBuilder_Between_WithNullMin_CreatesRangeCondition()
    {
        var condition = ConditionBuilder.Create()
            .Field("discount")
            .Between(null, 50)
            .Build();

        condition.Should().BeOfType<RangeCondition>();
        var range = (RangeCondition)condition;
        range.Min.Should().BeNull();
        range.Max.Should().Be(50);
    }

    [Fact]
    public void ConditionBuilder_Between_WithNullMax_CreatesRangeCondition()
    {
        var condition = ConditionBuilder.Create()
            .Field("minimum")
            .Between(100, null)
            .Build();

        condition.Should().BeOfType<RangeCondition>();
        var range = (RangeCondition)condition;
        range.Min.Should().Be(100);
        range.Max.Should().BeNull();
    }

    [Fact]
    public void ConditionBuilder_Expression_CreatesExpressionCondition()
    {
        var condition = ConditionBuilder.Create()
            .Expression("payload.amount > 1000 && payload.currency == 'USD'")
            .Build();

        condition.Should().BeOfType<ExpressionCondition>();
        var expr = (ExpressionCondition)condition;
        expr.Expression.Should().Contain("payload.amount");
        expr.Language.Should().Be("cel");
    }

    [Fact]
    public void ConditionBuilder_Expression_WithCustomLanguage_CreatesExpressionCondition()
    {
        var condition = ConditionBuilder.Create()
            .Expression("return event.amount > 1000", "javascript")
            .Build();

        condition.Should().BeOfType<ExpressionCondition>();
        var expr = (ExpressionCondition)condition;
        expr.Language.Should().Be("javascript");
    }

    [Fact]
    public void ConditionBuilder_And_CreatesAndCondition()
    {
        var cond1 = ConditionBuilder.Create().Field("a").Eq(1).Build();
        var cond2 = ConditionBuilder.Create().Field("b").Eq(2).Build();

        var condition = ConditionBuilder.And(cond1, cond2);

        condition.Should().BeOfType<AndCondition>();
        var andCond = (AndCondition)condition;
        andCond.Conditions.Should().HaveCount(2);
    }

    [Fact]
    public void ConditionBuilder_Or_CreatesOrCondition()
    {
        var cond1 = ConditionBuilder.Create().Field("status").Eq("approved").Build();
        var cond2 = ConditionBuilder.Create().Field("status").Eq("pending").Build();

        var condition = ConditionBuilder.Or(cond1, cond2);

        condition.Should().BeOfType<OrCondition>();
        var orCond = (OrCondition)condition;
        orCond.Conditions.Should().HaveCount(2);
    }

    [Fact]
    public void ConditionBuilder_Not_CreatesNotCondition()
    {
        var innerCond = ConditionBuilder.Create().Field("blocked").Eq(true).Build();

        var condition = ConditionBuilder.Not(innerCond);

        condition.Should().BeOfType<NotCondition>();
        var notCond = (NotCondition)condition;
        notCond.Cond.Should().BeOfType<CompareCondition>();
    }

    [Fact]
    public void ConditionBuilder_FluentChaining_Works()
    {
        var condition = ConditionBuilder.Create()
            .Field("payload.amount")
            .Gt(1000)
            .Build();

        condition.Should().NotBeNull();
    }

    [Fact]
    public void ConditionBuilder_ComplexExpression_CanBeBuilt()
    {
        // (amount > 1000 AND currency == "USD") OR vip == true
        var condition = ConditionBuilder.Or(
            ConditionBuilder.And(
                ConditionBuilder.Create().Field("amount").Gt(1000).Build(),
                ConditionBuilder.Create().Field("currency").Eq("USD").Build()
            ),
            ConditionBuilder.Create().Field("vip").Eq(true).Build()
        );

        condition.Should().BeOfType<OrCondition>();
        var orCond = (OrCondition)condition;
        orCond.Conditions.Should().HaveCount(2);
        orCond.Conditions[0].Should().BeOfType<AndCondition>();
    }

    #endregion

    #region RuleBuilder Tests

    [Fact]
    public void RuleBuilder_Create_ReturnsNewBuilder()
    {
        var builder = RuleBuilder.Create();

        builder.Should().NotBeNull();
    }

    [Fact]
    public void RuleBuilder_MinimalRule_CanBeBuilt()
    {
        var rule = RuleBuilder.Create()
            .Name("Test Rule")
            .When(ConditionBuilder.Create().Field("amount").Gt(100).Build())
            .ThenDeny("DENY_001", "Amount too high")
            .Build();

        rule.Should().NotBeNull();
        rule.Name.Should().Be("Test Rule");
        rule.Id.Should().StartWith("rule_");
    }

    [Fact]
    public void RuleBuilder_WithId_SetsId()
    {
        var rule = RuleBuilder.Create()
            .Id("custom-rule-id")
            .Name("Test Rule")
            .When(ConditionBuilder.Create().Field("x").Eq(1).Build())
            .ThenAllow("OK", "ok")
            .Build();

        rule.Id.Should().Be("custom-rule-id");
    }

    [Fact]
    public void RuleBuilder_WithDescription_SetsDescription()
    {
        var rule = RuleBuilder.Create()
            .Name("Test Rule")
            .Description("This is a detailed description")
            .When(ConditionBuilder.Create().Field("x").Eq(1).Build())
            .ThenAllow("OK", "ok")
            .Build();

        rule.Description.Should().Be("This is a detailed description");
    }

    [Fact]
    public void RuleBuilder_WithoutDescription_UsesNameAsDescription()
    {
        var rule = RuleBuilder.Create()
            .Name("Test Rule")
            .When(ConditionBuilder.Create().Field("x").Eq(1).Build())
            .ThenAllow("OK", "ok")
            .Build();

        rule.Description.Should().Be("Test Rule");
    }

    [Fact]
    public void RuleBuilder_WithJurisdiction_SetsJurisdiction()
    {
        var rule = RuleBuilder.Create()
            .Name("Test Rule")
            .Jurisdiction("US")
            .When(ConditionBuilder.Create().Field("x").Eq(1).Build())
            .ThenAllow("OK", "ok")
            .Build();

        rule.Jurisdiction.Should().Be("US");
    }

    [Fact]
    public void RuleBuilder_WithSeverity_SetsSeverity()
    {
        var rule = RuleBuilder.Create()
            .Name("Critical Rule")
            .WithSeverity(Severity.Critical)
            .When(ConditionBuilder.Create().Field("x").Eq(1).Build())
            .ThenDeny("CRITICAL", "Critical issue")
            .Build();

        rule.Severity.Should().Be(Severity.Critical);
    }

    [Fact]
    public void RuleBuilder_DefaultSeverity_IsMedium()
    {
        var rule = RuleBuilder.Create()
            .Name("Test Rule")
            .When(ConditionBuilder.Create().Field("x").Eq(1).Build())
            .ThenAllow("OK", "ok")
            .Build();

        rule.Severity.Should().Be(Severity.Medium);
    }

    [Fact]
    public void RuleBuilder_WhenField_CreatesCompareCondition()
    {
        var rule = RuleBuilder.Create()
            .Name("Test Rule")
            .WhenField("amount", "gt", 1000)
            .ThenDeny("AMOUNT_HIGH", "Amount is too high")
            .Build();

        rule.Condition.Should().BeOfType<CompareCondition>();
        var compare = (CompareCondition)rule.Condition;
        compare.Field.Should().Be("amount");
        compare.Operator.Should().Be("gt");
    }

    [Fact]
    public void RuleBuilder_ThenDeny_CreatesCorrectConsequence()
    {
        var rule = RuleBuilder.Create()
            .Name("Deny Rule")
            .When(ConditionBuilder.Create().Field("x").Eq(1).Build())
            .ThenDeny("DENY_001", "Denied")
            .Build();

        rule.Consequence.Decision.Should().Be(Decision.Deny);
        rule.Consequence.Code.Should().Be("DENY_001");
        rule.Consequence.Message.Should().Be("Denied");
    }

    [Fact]
    public void RuleBuilder_ThenFlag_CreatesCorrectConsequence()
    {
        var rule = RuleBuilder.Create()
            .Name("Flag Rule")
            .When(ConditionBuilder.Create().Field("x").Eq(1).Build())
            .ThenFlag("FLAG_001", "Flagged for review")
            .Build();

        rule.Consequence.Decision.Should().Be(Decision.Flag);
        rule.Consequence.Code.Should().Be("FLAG_001");
    }

    [Fact]
    public void RuleBuilder_ThenAllow_CreatesCorrectConsequence()
    {
        var rule = RuleBuilder.Create()
            .Name("Allow Rule")
            .When(ConditionBuilder.Create().Field("x").Eq(1).Build())
            .ThenAllow("ALLOW_001", "Allowed")
            .Build();

        rule.Consequence.Decision.Should().Be(Decision.Allow);
    }

    [Fact]
    public void RuleBuilder_WithConsequence_SetsCustomConsequence()
    {
        var consequence = new Consequence
        {
            Decision = Decision.Deny,
            Code = "CUSTOM_001",
            Message = "Custom denial",
            Metadata = new Dictionary<string, object> { ["reason"] = "custom" }
        };

        var rule = RuleBuilder.Create()
            .Name("Custom Rule")
            .When(ConditionBuilder.Create().Field("x").Eq(1).Build())
            .WithConsequence(consequence)
            .Build();

        rule.Consequence.Should().Be(consequence);
    }

    [Fact]
    public void RuleBuilder_WithTags_AddsTags()
    {
        var rule = RuleBuilder.Create()
            .Name("Tagged Rule")
            .When(ConditionBuilder.Create().Field("x").Eq(1).Build())
            .ThenAllow("OK", "ok")
            .Tags("compliance", "aml", "kyc")
            .Build();

        rule.Tags.Should().HaveCount(3);
        rule.Tags.Should().Contain("compliance");
        rule.Tags.Should().Contain("aml");
        rule.Tags.Should().Contain("kyc");
    }

    [Fact]
    public void RuleBuilder_WithEffectiveDates_SetsDates()
    {
        var from = DateTimeOffset.Parse("2024-01-01T00:00:00Z");
        var to = DateTimeOffset.Parse("2024-12-31T23:59:59Z");

        var rule = RuleBuilder.Create()
            .Name("Dated Rule")
            .When(ConditionBuilder.Create().Field("x").Eq(1).Build())
            .ThenAllow("OK", "ok")
            .EffectiveFrom(from)
            .EffectiveTo(to)
            .Build();

        rule.EffectiveFrom.Should().Be(from);
        rule.EffectiveTo.Should().Be(to);
    }

    [Fact]
    public void RuleBuilder_WithMetadata_AddsMetadata()
    {
        var rule = RuleBuilder.Create()
            .Name("Metadata Rule")
            .When(ConditionBuilder.Create().Field("x").Eq(1).Build())
            .ThenAllow("OK", "ok")
            .Metadata("key1", "value1")
            .Metadata("key2", 42)
            .Build();

        rule.Metadata.Should().ContainKey("key1");
        rule.Metadata["key1"].Should().Be("value1");
        rule.Metadata.Should().ContainKey("key2");
        rule.Metadata["key2"].Should().Be(42);
    }

    [Fact]
    public void RuleBuilder_WithoutName_ThrowsException()
    {
        var act = () => RuleBuilder.Create()
            .When(ConditionBuilder.Create().Field("x").Eq(1).Build())
            .ThenAllow("OK", "ok")
            .Build();

        act.Should().Throw<InvalidOperationException>()
            .WithMessage("*name*");
    }

    [Fact]
    public void RuleBuilder_WithoutCondition_ThrowsException()
    {
        var act = () => RuleBuilder.Create()
            .Name("Test Rule")
            .ThenAllow("OK", "ok")
            .Build();

        act.Should().Throw<InvalidOperationException>()
            .WithMessage("*condition*");
    }

    [Fact]
    public void RuleBuilder_WithoutConsequence_ThrowsException()
    {
        var act = () => RuleBuilder.Create()
            .Name("Test Rule")
            .When(ConditionBuilder.Create().Field("x").Eq(1).Build())
            .Build();

        act.Should().Throw<InvalidOperationException>()
            .WithMessage("*consequence*");
    }

    [Fact]
    public void RuleBuilder_FullyConfiguredRule_HasAllProperties()
    {
        var rule = RuleBuilder.Create()
            .Id("rule-full-config")
            .Name("Full Rule")
            .Description("A fully configured rule")
            .Jurisdiction("EU")
            .WithSeverity(Severity.High)
            .When(ConditionBuilder.And(
                ConditionBuilder.Create().Field("amount").Gt(10000).Build(),
                ConditionBuilder.Create().Field("currency").In("EUR", "GBP").Build()
            ))
            .ThenFlag("HIGH_VALUE_EU", "High value EU transaction")
            .Tags("compliance", "eu", "high-value")
            .EffectiveFrom(DateTimeOffset.Parse("2024-01-01T00:00:00Z"))
            .Metadata("created_by", "test")
            .Build();

        rule.Id.Should().Be("rule-full-config");
        rule.Name.Should().Be("Full Rule");
        rule.Description.Should().Be("A fully configured rule");
        rule.Jurisdiction.Should().Be("EU");
        rule.Severity.Should().Be(Severity.High);
        rule.Condition.Should().BeOfType<AndCondition>();
        rule.Consequence.Decision.Should().Be(Decision.Flag);
        rule.Tags.Should().HaveCount(3);
        rule.EffectiveFrom.Should().NotBeNull();
        rule.Metadata.Should().ContainKey("created_by");
    }

    #endregion
}
