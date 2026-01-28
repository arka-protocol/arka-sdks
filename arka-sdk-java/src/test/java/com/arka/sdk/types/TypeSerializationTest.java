package com.arka.sdk.types;

import com.fasterxml.jackson.databind.ObjectMapper;
import com.fasterxml.jackson.datatype.jsr310.JavaTimeModule;
import org.junit.jupiter.api.*;

import java.time.Instant;
import java.util.List;
import java.util.Map;

import static org.assertj.core.api.Assertions.*;

/**
 * Tests for type serialization and deserialization with Jackson.
 */
@DisplayName("Type Serialization")
class TypeSerializationTest {

    private ObjectMapper mapper;

    @BeforeEach
    void setUp() {
        mapper = new ObjectMapper()
            .registerModule(new JavaTimeModule());
    }

    @Nested
    @DisplayName("PactEvent")
    class PactEventTests {

        @Test
        @DisplayName("should serialize PactEvent to JSON")
        void serializeEvent() throws Exception {
            Instant now = Instant.parse("2024-01-15T10:30:00Z");
            PactEvent event = PactEvent.builder()
                .id("evt_123")
                .source("test-plugin")
                .type("user_created")
                .entityId("user_456")
                .entityType("User")
                .jurisdiction("US")
                .payload(Map.of("email", "test@example.com"))
                .occurredAt(now)
                .receivedAt(now)
                .metadata(Map.of("version", "1.0"))
                .build();

            String json = mapper.writeValueAsString(event);

            assertThat(json).contains("\"id\":\"evt_123\"");
            assertThat(json).contains("\"entity_id\":\"user_456\"");
            assertThat(json).contains("\"entity_type\":\"User\"");
            assertThat(json).contains("\"occurred_at\"");
            assertThat(json).contains("\"received_at\"");
        }

        @Test
        @DisplayName("should deserialize JSON to PactEvent")
        void deserializeEvent() throws Exception {
            String json = """
                {
                    "id": "evt_789",
                    "source": "finance-plugin",
                    "type": "payment_processed",
                    "entity_id": "payment_123",
                    "entity_type": "Payment",
                    "jurisdiction": "EU",
                    "payload": {"amount": 500.0, "currency": "EUR"},
                    "occurred_at": "2024-01-15T12:00:00Z",
                    "received_at": "2024-01-15T12:00:01Z",
                    "metadata": {}
                }
                """;

            PactEvent event = mapper.readValue(json, PactEvent.class);

            assertThat(event.id()).isEqualTo("evt_789");
            assertThat(event.source()).isEqualTo("finance-plugin");
            assertThat(event.type()).isEqualTo("payment_processed");
            assertThat(event.entityId()).isEqualTo("payment_123");
            assertThat(event.entityType()).isEqualTo("Payment");
            assertThat(event.jurisdiction()).isEqualTo("EU");
            assertThat(event.payload()).containsEntry("amount", 500.0);
        }

        @Test
        @DisplayName("should handle null payload and metadata")
        void nullPayloadAndMetadata() throws Exception {
            String json = """
                {
                    "id": "evt_null",
                    "source": "test",
                    "type": "test_event"
                }
                """;

            PactEvent event = mapper.readValue(json, PactEvent.class);

            assertThat(event.payload()).isNotNull().isEmpty();
            assertThat(event.metadata()).isNotNull().isEmpty();
        }

        @Test
        @DisplayName("should roundtrip PactEvent")
        void roundtripEvent() throws Exception {
            Instant now = Instant.now();
            PactEvent original = PactEvent.builder()
                .id("evt_rt")
                .source("roundtrip-test")
                .type("test_event")
                .entityId("ent_rt")
                .entityType("TestEntity")
                .jurisdiction("UK")
                .payload(Map.of("key", "value", "nested", Map.of("a", 1)))
                .occurredAt(now)
                .receivedAt(now)
                .metadata(Map.of("trace_id", "abc123"))
                .build();

            String json = mapper.writeValueAsString(original);
            PactEvent deserialized = mapper.readValue(json, PactEvent.class);

            assertThat(deserialized.id()).isEqualTo(original.id());
            assertThat(deserialized.source()).isEqualTo(original.source());
            assertThat(deserialized.type()).isEqualTo(original.type());
            assertThat(deserialized.entityId()).isEqualTo(original.entityId());
            assertThat(deserialized.entityType()).isEqualTo(original.entityType());
        }
    }

    @Nested
    @DisplayName("PactEntity")
    class PactEntityTests {

        @Test
        @DisplayName("should serialize PactEntity to JSON")
        void serializeEntity() throws Exception {
            Instant now = Instant.parse("2024-02-01T09:00:00Z");
            PactEntity entity = PactEntity.builder()
                .id("ent_123")
                .type("User")
                .data(Map.of("name", "Alice", "age", 30))
                .createdAt(now)
                .updatedAt(now)
                .jurisdiction("CA")
                .metadata(Map.of())
                .build();

            String json = mapper.writeValueAsString(entity);

            assertThat(json).contains("\"id\":\"ent_123\"");
            assertThat(json).contains("\"type\":\"User\"");
            assertThat(json).contains("\"created_at\"");
            assertThat(json).contains("\"updated_at\"");
        }

        @Test
        @DisplayName("should deserialize JSON to PactEntity")
        void deserializeEntity() throws Exception {
            String json = """
                {
                    "id": "ent_456",
                    "type": "Account",
                    "data": {"balance": 1000.0, "currency": "USD"},
                    "created_at": "2024-02-01T10:00:00Z",
                    "updated_at": "2024-02-01T11:00:00Z",
                    "jurisdiction": "US",
                    "metadata": {"tier": "premium"}
                }
                """;

            PactEntity entity = mapper.readValue(json, PactEntity.class);

            assertThat(entity.id()).isEqualTo("ent_456");
            assertThat(entity.type()).isEqualTo("Account");
            assertThat(entity.data()).containsEntry("balance", 1000.0);
            assertThat(entity.jurisdiction()).isEqualTo("US");
            assertThat(entity.metadata()).containsEntry("tier", "premium");
        }
    }

    @Nested
    @DisplayName("PactRule")
    class PactRuleTests {

        @Test
        @DisplayName("should serialize PactRule with Compare condition")
        void serializeRuleWithCompare() throws Exception {
            PactRule rule = PactRule.builder()
                .id("rule_123")
                .name("Amount Limit")
                .description("Deny if amount exceeds limit")
                .severity(Severity.HIGH)
                .condition(new Condition.Compare("amount", "gt", 10000))
                .consequence(Consequence.deny("LIMIT_EXCEEDED", "Amount too high"))
                .tags(List.of("compliance"))
                .build();

            String json = mapper.writeValueAsString(rule);

            assertThat(json).contains("\"id\":\"rule_123\"");
            assertThat(json).contains("\"severity\":\"HIGH\"");
            assertThat(json).contains("\"type\":\"compare\"");
            assertThat(json).contains("\"field\":\"amount\"");
            assertThat(json).contains("\"operator\":\"gt\"");
        }

        @Test
        @DisplayName("should deserialize PactRule with Compare condition")
        void deserializeRuleWithCompare() throws Exception {
            String json = """
                {
                    "id": "rule_456",
                    "name": "Velocity Check",
                    "description": "Flag if velocity exceeds threshold",
                    "severity": "MEDIUM",
                    "condition": {
                        "type": "compare",
                        "field": "velocity",
                        "operator": "gte",
                        "value": 5
                    },
                    "consequence": {
                        "decision": "FLAG",
                        "code": "HIGH_VELOCITY",
                        "message": "High transaction velocity detected"
                    },
                    "tags": ["fraud", "velocity"]
                }
                """;

            PactRule rule = mapper.readValue(json, PactRule.class);

            assertThat(rule.id()).isEqualTo("rule_456");
            assertThat(rule.severity()).isEqualTo(Severity.MEDIUM);
            assertThat(rule.condition()).isInstanceOf(Condition.Compare.class);

            Condition.Compare compare = (Condition.Compare) rule.condition();
            assertThat(compare.field()).isEqualTo("velocity");
            assertThat(compare.operator()).isEqualTo("gte");
            assertThat(compare.value()).isEqualTo(5);

            assertThat(rule.consequence().decision()).isEqualTo(Decision.FLAG);
        }

        @Test
        @DisplayName("should serialize PactRule with And condition")
        void serializeRuleWithAnd() throws Exception {
            Condition.And andCondition = new Condition.And(List.of(
                new Condition.Compare("amount", "gt", 1000),
                new Condition.Compare("country", "eq", "XX")
            ));

            PactRule rule = PactRule.builder()
                .id("rule_and")
                .name("Combined Rule")
                .condition(andCondition)
                .consequence(Consequence.deny("COMBINED_DENY", "Multiple conditions met"))
                .build();

            String json = mapper.writeValueAsString(rule);

            assertThat(json).contains("\"type\":\"and\"");
            assertThat(json).contains("\"conditions\"");
        }

        @Test
        @DisplayName("should deserialize PactRule with And condition")
        void deserializeRuleWithAnd() throws Exception {
            String json = """
                {
                    "id": "rule_and",
                    "name": "And Rule",
                    "condition": {
                        "type": "and",
                        "conditions": [
                            {"type": "compare", "field": "amount", "operator": "gt", "value": 500},
                            {"type": "exists", "field": "riskScore"}
                        ]
                    },
                    "consequence": {
                        "decision": "FLAG",
                        "code": "AND_MATCH",
                        "message": "And condition matched"
                    }
                }
                """;

            PactRule rule = mapper.readValue(json, PactRule.class);

            assertThat(rule.condition()).isInstanceOf(Condition.And.class);
            Condition.And and = (Condition.And) rule.condition();
            assertThat(and.conditions()).hasSize(2);
        }

        @Test
        @DisplayName("should serialize PactRule with Or condition")
        void serializeRuleWithOr() throws Exception {
            Condition.Or orCondition = new Condition.Or(List.of(
                new Condition.Compare("risk_level", "eq", "HIGH"),
                new Condition.Compare("blocked", "eq", true)
            ));

            PactRule rule = PactRule.builder()
                .id("rule_or")
                .name("Or Rule")
                .condition(orCondition)
                .consequence(Consequence.flag("OR_FLAG", "One condition met"))
                .build();

            String json = mapper.writeValueAsString(rule);

            assertThat(json).contains("\"type\":\"or\"");
        }

        @Test
        @DisplayName("should serialize PactRule with Not condition")
        void serializeRuleWithNot() throws Exception {
            Condition.Not notCondition = new Condition.Not(
                new Condition.Compare("verified", "eq", true)
            );

            PactRule rule = PactRule.builder()
                .id("rule_not")
                .name("Not Verified")
                .condition(notCondition)
                .consequence(Consequence.deny("NOT_VERIFIED", "Entity not verified"))
                .build();

            String json = mapper.writeValueAsString(rule);

            assertThat(json).contains("\"type\":\"not\"");
        }

        @Test
        @DisplayName("should serialize PactRule with In condition")
        void serializeRuleWithIn() throws Exception {
            Condition.In inCondition = new Condition.In(
                "country",
                List.of("XX", "YY", "ZZ")
            );

            PactRule rule = PactRule.builder()
                .id("rule_in")
                .name("Blocked Countries")
                .condition(inCondition)
                .consequence(Consequence.deny("BLOCKED_COUNTRY", "Country is blocked"))
                .build();

            String json = mapper.writeValueAsString(rule);

            assertThat(json).contains("\"type\":\"in\"");
            assertThat(json).contains("\"values\"");
        }

        @Test
        @DisplayName("should serialize PactRule with Range condition")
        void serializeRuleWithRange() throws Exception {
            Condition.Range rangeCondition = new Condition.Range(
                "age", 18.0, 65.0, true, true
            );

            PactRule rule = PactRule.builder()
                .id("rule_range")
                .name("Age Range")
                .condition(rangeCondition)
                .consequence(Consequence.allow("AGE_OK", "Age within range"))
                .build();

            String json = mapper.writeValueAsString(rule);

            assertThat(json).contains("\"type\":\"range\"");
            assertThat(json).contains("\"min\":18.0");
            assertThat(json).contains("\"max\":65.0");
        }

        @Test
        @DisplayName("should serialize PactRule with Expression condition")
        void serializeRuleWithExpression() throws Exception {
            Condition.Expression exprCondition = new Condition.Expression(
                "event.payload.amount > 1000 && event.payload.currency == 'USD'",
                "cel"
            );

            PactRule rule = PactRule.builder()
                .id("rule_expr")
                .name("CEL Expression Rule")
                .condition(exprCondition)
                .consequence(Consequence.flag("EXPR_MATCH", "Expression matched"))
                .build();

            String json = mapper.writeValueAsString(rule);

            assertThat(json).contains("\"type\":\"expression\"");
            assertThat(json).contains("\"language\":\"cel\"");
        }

        @Test
        @DisplayName("should deserialize PactRule with Expression condition")
        void deserializeRuleWithExpression() throws Exception {
            String json = """
                {
                    "id": "rule_expr",
                    "name": "Expression Rule",
                    "condition": {
                        "type": "expression",
                        "expression": "payload.risk_score >= 80",
                        "language": "cel"
                    },
                    "consequence": {
                        "decision": "DENY",
                        "code": "HIGH_RISK",
                        "message": "Risk score too high"
                    }
                }
                """;

            PactRule rule = mapper.readValue(json, PactRule.class);

            assertThat(rule.condition()).isInstanceOf(Condition.Expression.class);
            Condition.Expression expr = (Condition.Expression) rule.condition();
            assertThat(expr.expression()).isEqualTo("payload.risk_score >= 80");
            assertThat(expr.language()).isEqualTo("cel");
        }
    }

    @Nested
    @DisplayName("PactDecision")
    class PactDecisionTests {

        @Test
        @DisplayName("should serialize PactDecision to JSON")
        void serializeDecision() throws Exception {
            Instant now = Instant.parse("2024-03-01T14:00:00Z");
            PactDecision decision = PactDecision.builder()
                .id("dec_123")
                .eventId("evt_456")
                .status(DecisionStatus.DENY)
                .triggeredRules(List.of(
                    new PactDecision.TriggeredRule(
                        "rule_1", "High Risk",
                        Decision.DENY, "HIGH_RISK", "Risk exceeds threshold", Severity.CRITICAL
                    )
                ))
                .decidedAt(now)
                .executionTimeMs(10L)
                .metadata(Map.of("trace_id", "xyz789"))
                .build();

            String json = mapper.writeValueAsString(decision);

            assertThat(json).contains("\"id\":\"dec_123\"");
            assertThat(json).contains("\"event_id\":\"evt_456\"");
            assertThat(json).contains("\"status\":\"DENY\"");
            assertThat(json).contains("\"triggered_rules\"");
            assertThat(json).contains("\"execution_time_ms\":10");
        }

        @Test
        @DisplayName("should deserialize JSON to PactDecision")
        void deserializeDecision() throws Exception {
            String json = """
                {
                    "id": "dec_789",
                    "event_id": "evt_abc",
                    "status": "ALLOW_WITH_FLAGS",
                    "triggered_rules": [
                        {
                            "rule_id": "rule_1",
                            "rule_name": "Velocity Warning",
                            "decision": "FLAG",
                            "code": "HIGH_VELOCITY",
                            "message": "Transaction velocity is elevated",
                            "severity": "MEDIUM"
                        }
                    ],
                    "decided_at": "2024-03-01T15:00:00Z",
                    "execution_time_ms": 5,
                    "metadata": {}
                }
                """;

            PactDecision decision = mapper.readValue(json, PactDecision.class);

            assertThat(decision.id()).isEqualTo("dec_789");
            assertThat(decision.eventId()).isEqualTo("evt_abc");
            assertThat(decision.status()).isEqualTo(DecisionStatus.ALLOW_WITH_FLAGS);
            assertThat(decision.isAllowed()).isTrue();
            assertThat(decision.hasFlagged()).isTrue();
            assertThat(decision.triggeredRules()).hasSize(1);
            assertThat(decision.triggeredRules().get(0).ruleName()).isEqualTo("Velocity Warning");
        }
    }

    @Nested
    @DisplayName("ValidationResult")
    class ValidationResultTests {

        @Test
        @DisplayName("should serialize success result")
        void serializeSuccess() throws Exception {
            ValidationResult result = ValidationResult.success();

            String json = mapper.writeValueAsString(result);

            assertThat(json).contains("\"valid\":true");
            assertThat(json).contains("\"errors\":[]");
            assertThat(json).contains("\"warnings\":[]");
        }

        @Test
        @DisplayName("should serialize failure result")
        void serializeFailure() throws Exception {
            ValidationResult result = ValidationResult.failure(List.of(
                new ValidationError("email", "Invalid email format", "INVALID_EMAIL"),
                new ValidationError("name", "Name is required", "REQUIRED")
            ));

            String json = mapper.writeValueAsString(result);

            assertThat(json).contains("\"valid\":false");
            assertThat(json).contains("INVALID_EMAIL");
            assertThat(json).contains("REQUIRED");
        }

        @Test
        @DisplayName("should deserialize validation result")
        void deserializeResult() throws Exception {
            String json = """
                {
                    "valid": false,
                    "errors": [
                        {"field": "amount", "message": "Must be positive", "code": "POSITIVE_REQUIRED"}
                    ],
                    "warnings": [
                        {"field": "description", "message": "Description is recommended"}
                    ]
                }
                """;

            ValidationResult result = mapper.readValue(json, ValidationResult.class);

            assertThat(result.valid()).isFalse();
            assertThat(result.errors()).hasSize(1);
            assertThat(result.errors().get(0).code()).isEqualTo("POSITIVE_REQUIRED");
            assertThat(result.warnings()).hasSize(1);
        }
    }

    @Nested
    @DisplayName("DomainEvent")
    class DomainEventTests {

        @Test
        @DisplayName("should serialize DomainEvent")
        void serializeDomainEvent() throws Exception {
            Instant now = Instant.parse("2024-04-01T08:00:00Z");
            DomainEvent event = DomainEvent.builder()
                .type("order_placed")
                .payload(Map.of("order_id", "ord_123", "total", 99.99))
                .entityId("order_123")
                .jurisdiction("US")
                .occurredAt(now)
                .metadata(Map.of("source", "mobile_app"))
                .build();

            String json = mapper.writeValueAsString(event);

            assertThat(json).contains("\"type\":\"order_placed\"");
            assertThat(json).contains("\"entity_id\":\"order_123\"");
            assertThat(json).contains("\"occurred_at\"");
        }

        @Test
        @DisplayName("should deserialize DomainEvent")
        void deserializeDomainEvent() throws Exception {
            String json = """
                {
                    "type": "inventory_updated",
                    "payload": {"sku": "ABC123", "quantity": 50},
                    "entity_id": "inv_456",
                    "jurisdiction": "EU",
                    "occurred_at": "2024-04-01T09:00:00Z",
                    "metadata": {}
                }
                """;

            DomainEvent event = mapper.readValue(json, DomainEvent.class);

            assertThat(event.type()).isEqualTo("inventory_updated");
            assertThat(event.entityId()).isEqualTo("inv_456");
            assertThat(event.jurisdiction()).isEqualTo("EU");
        }
    }

    @Nested
    @DisplayName("PluginManifest")
    class PluginManifestTests {

        @Test
        @DisplayName("should serialize PluginManifest")
        void serializeManifest() throws Exception {
            PluginManifest manifest = PluginManifest.builder()
                .id("finance-plugin")
                .name("Finance Domain Plugin")
                .version("1.0.0")
                .author("PACT Team")
                .description("Plugin for financial compliance")
                .entityTypes(List.of("Transaction", "Account", "Payment"))
                .eventTypes(List.of("transaction_created", "payment_processed"))
                .dependencies(List.of("core-plugin"))
                .pactCoreVersion("^1.0.0")
                .build();

            String json = mapper.writeValueAsString(manifest);

            assertThat(json).contains("\"id\":\"finance-plugin\"");
            assertThat(json).contains("\"entity_types\"");
            assertThat(json).contains("\"event_types\"");
            assertThat(json).contains("\"pact_core_version\"");
        }

        @Test
        @DisplayName("should deserialize PluginManifest")
        void deserializeManifest() throws Exception {
            String json = """
                {
                    "id": "healthcare-plugin",
                    "name": "Healthcare Domain Plugin",
                    "version": "2.0.0",
                    "author": "Health Tech Inc",
                    "description": "HIPAA compliance plugin",
                    "entity_types": ["Patient", "Provider", "Claim"],
                    "event_types": ["claim_submitted", "patient_enrolled"],
                    "dependencies": [],
                    "pact_core_version": ">=1.5.0"
                }
                """;

            PluginManifest manifest = mapper.readValue(json, PluginManifest.class);

            assertThat(manifest.id()).isEqualTo("healthcare-plugin");
            assertThat(manifest.version()).isEqualTo("2.0.0");
            assertThat(manifest.entityTypes()).containsExactly("Patient", "Provider", "Claim");
            assertThat(manifest.pactCoreVersion()).isEqualTo(">=1.5.0");
        }
    }

    @Nested
    @DisplayName("Enums")
    class EnumTests {

        @Test
        @DisplayName("should serialize Severity enum")
        void serializeSeverity() throws Exception {
            assertThat(mapper.writeValueAsString(Severity.LOW)).isEqualTo("\"LOW\"");
            assertThat(mapper.writeValueAsString(Severity.MEDIUM)).isEqualTo("\"MEDIUM\"");
            assertThat(mapper.writeValueAsString(Severity.HIGH)).isEqualTo("\"HIGH\"");
            assertThat(mapper.writeValueAsString(Severity.CRITICAL)).isEqualTo("\"CRITICAL\"");
        }

        @Test
        @DisplayName("should deserialize Severity enum")
        void deserializeSeverity() throws Exception {
            assertThat(mapper.readValue("\"LOW\"", Severity.class)).isEqualTo(Severity.LOW);
            assertThat(mapper.readValue("\"CRITICAL\"", Severity.class)).isEqualTo(Severity.CRITICAL);
        }

        @Test
        @DisplayName("should serialize Decision enum")
        void serializeDecision() throws Exception {
            assertThat(mapper.writeValueAsString(Decision.ALLOW)).isEqualTo("\"ALLOW\"");
            assertThat(mapper.writeValueAsString(Decision.DENY)).isEqualTo("\"DENY\"");
            assertThat(mapper.writeValueAsString(Decision.FLAG)).isEqualTo("\"FLAG\"");
        }

        @Test
        @DisplayName("should serialize DecisionStatus enum")
        void serializeDecisionStatus() throws Exception {
            assertThat(mapper.writeValueAsString(DecisionStatus.ALLOW)).isEqualTo("\"ALLOW\"");
            assertThat(mapper.writeValueAsString(DecisionStatus.ALLOW_WITH_FLAGS)).isEqualTo("\"ALLOW_WITH_FLAGS\"");
            assertThat(mapper.writeValueAsString(DecisionStatus.DENY)).isEqualTo("\"DENY\"");
        }
    }

    @Nested
    @DisplayName("Consequence")
    class ConsequenceTests {

        @Test
        @DisplayName("should serialize Consequence")
        void serializeConsequence() throws Exception {
            Consequence consequence = new Consequence(
                Decision.DENY,
                "BLOCKED",
                "Action is blocked",
                Map.of("reason", "policy violation")
            );

            String json = mapper.writeValueAsString(consequence);

            assertThat(json).contains("\"decision\":\"DENY\"");
            assertThat(json).contains("\"code\":\"BLOCKED\"");
            assertThat(json).contains("\"message\":\"Action is blocked\"");
        }

        @Test
        @DisplayName("should create deny consequence")
        void createDenyConsequence() {
            Consequence deny = Consequence.deny("ERROR_CODE", "Error message");

            assertThat(deny.decision()).isEqualTo(Decision.DENY);
            assertThat(deny.code()).isEqualTo("ERROR_CODE");
            assertThat(deny.message()).isEqualTo("Error message");
        }

        @Test
        @DisplayName("should create flag consequence")
        void createFlagConsequence() {
            Consequence flag = Consequence.flag("WARNING_CODE", "Warning message");

            assertThat(flag.decision()).isEqualTo(Decision.FLAG);
            assertThat(flag.code()).isEqualTo("WARNING_CODE");
        }

        @Test
        @DisplayName("should create allow consequence")
        void createAllowConsequence() {
            Consequence allow = Consequence.allow("OK_CODE", "All good");

            assertThat(allow.decision()).isEqualTo(Decision.ALLOW);
            assertThat(allow.code()).isEqualTo("OK_CODE");
        }
    }
}
