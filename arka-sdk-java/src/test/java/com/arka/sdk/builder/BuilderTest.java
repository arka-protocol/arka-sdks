package com.arka.sdk.builder;

import com.arka.sdk.types.*;
import org.junit.jupiter.api.*;

import java.time.Instant;
import java.util.List;

import static org.assertj.core.api.Assertions.*;

/**
 * Tests for RuleBuilder and ConditionBuilder.
 */
@DisplayName("Builders")
class BuilderTest {

    @Nested
    @DisplayName("RuleBuilder")
    class RuleBuilderTests {

        @Test
        @DisplayName("should build rule with all fields")
        void buildCompleteRule() {
            Instant effectiveFrom = Instant.parse("2024-01-01T00:00:00Z");
            Instant effectiveTo = Instant.parse("2024-12-31T23:59:59Z");

            PactRule rule = RuleBuilder.rule()
                .id("rule_custom")
                .name("High Value Transaction")
                .description("Flag transactions exceeding $10,000")
                .jurisdiction("US")
                .severity(Severity.HIGH)
                .whenField("amount", "gt", 10000)
                .thenFlag("HIGH_VALUE", "Transaction exceeds $10,000")
                .tags("compliance", "aml", "finance")
                .effectiveFrom(effectiveFrom)
                .effectiveTo(effectiveTo)
                .metadata("author", "compliance-team")
                .metadata("version", "1.0")
                .build();

            assertThat(rule.id()).isEqualTo("rule_custom");
            assertThat(rule.name()).isEqualTo("High Value Transaction");
            assertThat(rule.description()).isEqualTo("Flag transactions exceeding $10,000");
            assertThat(rule.jurisdiction()).isEqualTo("US");
            assertThat(rule.severity()).isEqualTo(Severity.HIGH);
            assertThat(rule.condition()).isInstanceOf(Condition.Compare.class);
            assertThat(rule.consequence().decision()).isEqualTo(Decision.FLAG);
            assertThat(rule.tags()).containsExactly("compliance", "aml", "finance");
            assertThat(rule.effectiveFrom()).isEqualTo(effectiveFrom);
            assertThat(rule.effectiveTo()).isEqualTo(effectiveTo);
            assertThat(rule.metadata()).containsEntry("author", "compliance-team");
        }

        @Test
        @DisplayName("should generate rule ID if not provided")
        void generateRuleId() {
            PactRule rule = RuleBuilder.rule()
                .name("Auto ID Rule")
                .whenField("status", "eq", "BLOCKED")
                .thenDeny("BLOCKED", "Status is blocked")
                .build();

            assertThat(rule.id()).startsWith("rule_");
            assertThat(rule.id()).hasSize(17); // "rule_" + 12 chars
        }

        @Test
        @DisplayName("should use name as description if not provided")
        void useNameAsDescription() {
            PactRule rule = RuleBuilder.rule()
                .name("Simple Rule")
                .whenField("active", "eq", false)
                .thenDeny("INACTIVE", "Entity is inactive")
                .build();

            assertThat(rule.description()).isEqualTo("Simple Rule");
        }

        @Test
        @DisplayName("should default severity to MEDIUM")
        void defaultSeverity() {
            PactRule rule = RuleBuilder.rule()
                .name("Default Severity Rule")
                .whenField("test", "eq", true)
                .thenAllow("TEST_OK", "Test passed")
                .build();

            assertThat(rule.severity()).isEqualTo(Severity.MEDIUM);
        }

        @Test
        @DisplayName("should build rule with custom condition")
        void buildWithCustomCondition() {
            Condition.And andCondition = new Condition.And(List.of(
                new Condition.Compare("amount", "gt", 5000),
                new Condition.Compare("currency", "eq", "USD")
            ));

            PactRule rule = RuleBuilder.rule()
                .name("Combined Condition Rule")
                .when(andCondition)
                .thenFlag("COMBINED", "Both conditions met")
                .build();

            assertThat(rule.condition()).isInstanceOf(Condition.And.class);
        }

        @Test
        @DisplayName("should build rule with thenDeny")
        void buildWithDeny() {
            PactRule rule = RuleBuilder.rule()
                .name("Deny Rule")
                .whenField("blocked", "eq", true)
                .thenDeny("ENTITY_BLOCKED", "Entity is on blocklist")
                .build();

            assertThat(rule.consequence().decision()).isEqualTo(Decision.DENY);
            assertThat(rule.consequence().code()).isEqualTo("ENTITY_BLOCKED");
        }

        @Test
        @DisplayName("should build rule with thenFlag")
        void buildWithFlag() {
            PactRule rule = RuleBuilder.rule()
                .name("Flag Rule")
                .whenField("risk_score", "gt", 70)
                .thenFlag("HIGH_RISK", "High risk score detected")
                .build();

            assertThat(rule.consequence().decision()).isEqualTo(Decision.FLAG);
        }

        @Test
        @DisplayName("should build rule with thenAllow")
        void buildWithAllow() {
            PactRule rule = RuleBuilder.rule()
                .name("Allow Rule")
                .whenField("verified", "eq", true)
                .thenAllow("VERIFIED", "Entity is verified")
                .build();

            assertThat(rule.consequence().decision()).isEqualTo(Decision.ALLOW);
        }

        @Test
        @DisplayName("should build rule with custom consequence")
        void buildWithCustomConsequence() {
            Consequence consequence = new Consequence(
                Decision.FLAG,
                "CUSTOM_FLAG",
                "Custom flag message",
                java.util.Map.of("extra", "data")
            );

            PactRule rule = RuleBuilder.rule()
                .name("Custom Consequence Rule")
                .whenField("custom", "eq", true)
                .consequence(consequence)
                .build();

            assertThat(rule.consequence().metadata()).containsEntry("extra", "data");
        }

        @Test
        @DisplayName("should throw if name is missing")
        void throwOnMissingName() {
            RuleBuilder builder = RuleBuilder.rule()
                .whenField("test", "eq", true)
                .thenAllow("OK", "OK");

            assertThatThrownBy(builder::build)
                .isInstanceOf(IllegalArgumentException.class)
                .hasMessageContaining("name is required");
        }

        @Test
        @DisplayName("should throw if condition is missing")
        void throwOnMissingCondition() {
            RuleBuilder builder = RuleBuilder.rule()
                .name("Missing Condition")
                .thenAllow("OK", "OK");

            assertThatThrownBy(builder::build)
                .isInstanceOf(IllegalArgumentException.class)
                .hasMessageContaining("condition is required");
        }

        @Test
        @DisplayName("should throw if consequence is missing")
        void throwOnMissingConsequence() {
            RuleBuilder builder = RuleBuilder.rule()
                .name("Missing Consequence")
                .whenField("test", "eq", true);

            assertThatThrownBy(builder::build)
                .isInstanceOf(IllegalArgumentException.class)
                .hasMessageContaining("consequence is required");
        }
    }

    @Nested
    @DisplayName("ConditionBuilder")
    class ConditionBuilderTests {

        @Test
        @DisplayName("should build eq condition")
        void buildEqCondition() {
            Condition condition = ConditionBuilder.condition()
                .field("status")
                .eq("ACTIVE")
                .build();

            assertThat(condition).isInstanceOf(Condition.Compare.class);
            Condition.Compare compare = (Condition.Compare) condition;
            assertThat(compare.field()).isEqualTo("status");
            assertThat(compare.operator()).isEqualTo("eq");
            assertThat(compare.value()).isEqualTo("ACTIVE");
        }

        @Test
        @DisplayName("should build ne condition")
        void buildNeCondition() {
            Condition condition = ConditionBuilder.condition()
                .field("type")
                .ne("BLOCKED")
                .build();

            Condition.Compare compare = (Condition.Compare) condition;
            assertThat(compare.operator()).isEqualTo("ne");
        }

        @Test
        @DisplayName("should build gt condition")
        void buildGtCondition() {
            Condition condition = ConditionBuilder.condition()
                .field("amount")
                .gt(1000)
                .build();

            Condition.Compare compare = (Condition.Compare) condition;
            assertThat(compare.operator()).isEqualTo("gt");
            assertThat(compare.value()).isEqualTo(1000);
        }

        @Test
        @DisplayName("should build gte condition")
        void buildGteCondition() {
            Condition condition = ConditionBuilder.condition()
                .field("age")
                .gte(18)
                .build();

            Condition.Compare compare = (Condition.Compare) condition;
            assertThat(compare.operator()).isEqualTo("gte");
        }

        @Test
        @DisplayName("should build lt condition")
        void buildLtCondition() {
            Condition condition = ConditionBuilder.condition()
                .field("balance")
                .lt(0)
                .build();

            Condition.Compare compare = (Condition.Compare) condition;
            assertThat(compare.operator()).isEqualTo("lt");
        }

        @Test
        @DisplayName("should build lte condition")
        void buildLteCondition() {
            Condition condition = ConditionBuilder.condition()
                .field("quantity")
                .lte(100)
                .build();

            Condition.Compare compare = (Condition.Compare) condition;
            assertThat(compare.operator()).isEqualTo("lte");
        }

        @Test
        @DisplayName("should build contains condition")
        void buildContainsCondition() {
            Condition condition = ConditionBuilder.condition()
                .field("email")
                .contains("@example.com")
                .build();

            Condition.Compare compare = (Condition.Compare) condition;
            assertThat(compare.operator()).isEqualTo("contains");
        }

        @Test
        @DisplayName("should build matches (regex) condition")
        void buildMatchesCondition() {
            Condition condition = ConditionBuilder.condition()
                .field("phone")
                .matches("^\\+1\\d{10}$")
                .build();

            Condition.Compare compare = (Condition.Compare) condition;
            assertThat(compare.operator()).isEqualTo("regex");
        }

        @Test
        @DisplayName("should build exists condition")
        void buildExistsCondition() {
            Condition condition = ConditionBuilder.condition()
                .field("optionalField")
                .exists()
                .build();

            assertThat(condition).isInstanceOf(Condition.Exists.class);
            Condition.Exists exists = (Condition.Exists) condition;
            assertThat(exists.field()).isEqualTo("optionalField");
        }

        @Test
        @DisplayName("should build in condition")
        void buildInCondition() {
            Condition condition = ConditionBuilder.condition()
                .field("country")
                .in(List.of("US", "CA", "UK"))
                .build();

            assertThat(condition).isInstanceOf(Condition.In.class);
            Condition.In in = (Condition.In) condition;
            assertThat(in.field()).isEqualTo("country");
            assertThat(in.values()).containsExactly("US", "CA", "UK");
        }

        @Test
        @DisplayName("should build between (range) condition")
        void buildBetweenCondition() {
            Condition condition = ConditionBuilder.condition()
                .field("price")
                .between(10.0, 100.0)
                .build();

            assertThat(condition).isInstanceOf(Condition.Range.class);
            Condition.Range range = (Condition.Range) condition;
            assertThat(range.field()).isEqualTo("price");
            assertThat(range.min()).isEqualTo(10.0);
            assertThat(range.max()).isEqualTo(100.0);
            assertThat(range.minInclusive()).isTrue();
            assertThat(range.maxInclusive()).isTrue();
        }

        @Test
        @DisplayName("should build expression condition with default language")
        void buildExpressionConditionDefault() {
            Condition condition = ConditionBuilder.condition()
                .expression("payload.amount > 1000 && payload.currency == 'USD'", null)
                .build();

            assertThat(condition).isInstanceOf(Condition.Expression.class);
            Condition.Expression expr = (Condition.Expression) condition;
            assertThat(expr.language()).isEqualTo("cel");
        }

        @Test
        @DisplayName("should build expression condition with custom language")
        void buildExpressionConditionCustomLanguage() {
            Condition condition = ConditionBuilder.condition()
                .expression("return context.amount > 1000", "javascript")
                .build();

            Condition.Expression expr = (Condition.Expression) condition;
            assertThat(expr.language()).isEqualTo("javascript");
        }

        @Test
        @DisplayName("should build AND condition")
        void buildAndCondition() {
            Condition cond1 = ConditionBuilder.condition()
                .field("amount")
                .gt(1000)
                .build();

            Condition cond2 = ConditionBuilder.condition()
                .field("verified")
                .eq(true)
                .build();

            Condition and = ConditionBuilder.and(List.of(cond1, cond2));

            assertThat(and).isInstanceOf(Condition.And.class);
            Condition.And andCond = (Condition.And) and;
            assertThat(andCond.conditions()).hasSize(2);
        }

        @Test
        @DisplayName("should build OR condition")
        void buildOrCondition() {
            Condition cond1 = ConditionBuilder.condition()
                .field("risk")
                .eq("HIGH")
                .build();

            Condition cond2 = ConditionBuilder.condition()
                .field("blocked")
                .eq(true)
                .build();

            Condition or = ConditionBuilder.or(List.of(cond1, cond2));

            assertThat(or).isInstanceOf(Condition.Or.class);
            Condition.Or orCond = (Condition.Or) or;
            assertThat(orCond.conditions()).hasSize(2);
        }

        @Test
        @DisplayName("should build NOT condition")
        void buildNotCondition() {
            Condition inner = ConditionBuilder.condition()
                .field("enabled")
                .eq(true)
                .build();

            Condition not = ConditionBuilder.not(inner);

            assertThat(not).isInstanceOf(Condition.Not.class);
            Condition.Not notCond = (Condition.Not) not;
            assertThat(notCond.condition()).isEqualTo(inner);
        }

        @Test
        @DisplayName("should build nested conditions")
        void buildNestedConditions() {
            Condition amount = ConditionBuilder.condition()
                .field("amount")
                .gt(10000)
                .build();

            Condition country = ConditionBuilder.condition()
                .field("country")
                .in(List.of("XX", "YY"))
                .build();

            Condition verified = ConditionBuilder.condition()
                .field("verified")
                .eq(false)
                .build();

            // (amount > 10000 OR country in XX,YY) AND NOT verified
            Condition orPart = ConditionBuilder.or(List.of(amount, country));
            Condition notVerified = ConditionBuilder.not(verified);
            Condition combined = ConditionBuilder.and(List.of(orPart, notVerified));

            assertThat(combined).isInstanceOf(Condition.And.class);
            Condition.And andCond = (Condition.And) combined;
            assertThat(andCond.conditions()).hasSize(2);
            assertThat(andCond.conditions().get(0)).isInstanceOf(Condition.Or.class);
            assertThat(andCond.conditions().get(1)).isInstanceOf(Condition.Not.class);
        }
    }

    @Nested
    @DisplayName("Integration")
    class IntegrationTests {

        @Test
        @DisplayName("should build rule with complex condition using builders")
        void buildRuleWithComplexCondition() {
            // Rule: Flag if (amount > 5000 AND currency = USD) OR (country in blocked list)
            Condition highAmount = ConditionBuilder.condition()
                .field("amount")
                .gt(5000)
                .build();

            Condition usdCurrency = ConditionBuilder.condition()
                .field("currency")
                .eq("USD")
                .build();

            Condition blockedCountry = ConditionBuilder.condition()
                .field("country")
                .in(List.of("XX", "YY", "ZZ"))
                .build();

            Condition amountAndUsd = ConditionBuilder.and(List.of(highAmount, usdCurrency));
            Condition combined = ConditionBuilder.or(List.of(amountAndUsd, blockedCountry));

            PactRule rule = RuleBuilder.rule()
                .name("Complex AML Rule")
                .description("Flag high-value USD transactions or blocked countries")
                .severity(Severity.HIGH)
                .when(combined)
                .thenFlag("AML_FLAG", "Requires AML review")
                .tags("aml", "compliance", "high-value")
                .jurisdiction("US")
                .build();

            assertThat(rule.name()).isEqualTo("Complex AML Rule");
            assertThat(rule.condition()).isInstanceOf(Condition.Or.class);
            assertThat(rule.severity()).isEqualTo(Severity.HIGH);
        }
    }
}
