package com.arka.sdk.plugin;

import com.arka.sdk.types.*;
import org.junit.jupiter.api.*;

import java.time.Instant;
import java.util.List;
import java.util.Map;
import java.util.concurrent.atomic.AtomicBoolean;
import java.util.concurrent.atomic.AtomicReference;

import static org.assertj.core.api.Assertions.*;

/**
 * Tests for plugin classes.
 */
@DisplayName("Plugin")
class PluginTest {

    @Nested
    @DisplayName("BasePlugin")
    class BasePluginTests {

        private TestPlugin plugin;

        @BeforeEach
        void setUp() {
            plugin = new TestPlugin();
        }

        @Test
        @DisplayName("should return manifest")
        void getManifest() {
            PluginManifest manifest = plugin.getManifest();

            assertThat(manifest.id()).isEqualTo("test-plugin");
            assertThat(manifest.name()).isEqualTo("Test Plugin");
            assertThat(manifest.version()).isEqualTo("1.0.0");
        }

        @Test
        @DisplayName("should return entity types")
        void getEntityTypes() {
            List<PactEntityType> types = plugin.getEntityTypes();

            assertThat(types).hasSize(2);
            assertThat(types.get(0).name()).isEqualTo("User");
            assertThat(types.get(1).name()).isEqualTo("Transaction");
        }

        @Test
        @DisplayName("should return default rules")
        void getDefaultRules() {
            List<PactRule> rules = plugin.getDefaultRules();

            assertThat(rules).hasSize(1);
            assertThat(rules.get(0).name()).isEqualTo("Default Rule");
        }

        @Test
        @DisplayName("should map domain event to canonical event")
        void mapToCanonicalEvent() {
            Instant occurredAt = Instant.now();
            DomainEvent domainEvent = DomainEvent.builder()
                .type("user_created")
                .entityId("user_123")
                .jurisdiction("US")
                .payload(Map.of("email", "test@example.com"))
                .occurredAt(occurredAt)
                .metadata(Map.of("source", "api"))
                .build();

            PactEvent pactEvent = plugin.mapToCanonicalEvent(domainEvent);

            assertThat(pactEvent.id()).startsWith("evt_");
            assertThat(pactEvent.source()).isEqualTo("test-plugin");
            assertThat(pactEvent.type()).isEqualTo("user_created");
            assertThat(pactEvent.entityId()).isEqualTo("user_123");
            assertThat(pactEvent.entityType()).isEqualTo("User");
            assertThat(pactEvent.jurisdiction()).isEqualTo("US");
            assertThat(pactEvent.payload()).containsEntry("email", "test@example.com");
            assertThat(pactEvent.occurredAt()).isEqualTo(occurredAt);
            assertThat(pactEvent.receivedAt()).isNotNull();
            assertThat(pactEvent.metadata()).containsEntry("source", "api");
        }

        @Test
        @DisplayName("should use current time if occurredAt is null")
        void mapEventWithNullOccurredAt() {
            DomainEvent domainEvent = DomainEvent.builder()
                .type("test_event")
                .build();

            Instant before = Instant.now();
            PactEvent pactEvent = plugin.mapToCanonicalEvent(domainEvent);
            Instant after = Instant.now();

            assertThat(pactEvent.occurredAt())
                .isAfterOrEqualTo(before)
                .isBeforeOrEqualTo(after);
        }

        @Test
        @DisplayName("should infer entity type from event type")
        void inferEntityType() {
            DomainEvent domainEvent = DomainEvent.builder()
                .type("transaction_processed")
                .build();

            PactEvent pactEvent = plugin.mapToCanonicalEvent(domainEvent);

            assertThat(pactEvent.entityType()).isEqualTo("Transaction");
        }

        @Test
        @DisplayName("should validate domain data successfully")
        void validateDomainDataSuccess() {
            ValidationResult result = plugin.validateDomainData("User", Map.of(
                "name", "John Doe",
                "email", "john@example.com"
            ));

            assertThat(result.valid()).isTrue();
            assertThat(result.errors()).isEmpty();
        }

        @Test
        @DisplayName("should validate domain data with missing required fields")
        void validateDomainDataMissingFields() {
            ValidationResult result = plugin.validateDomainData("User", Map.of(
                "name", "John Doe"
                // missing email
            ));

            assertThat(result.valid()).isFalse();
            assertThat(result.errors()).hasSize(1);
            assertThat(result.errors().get(0).field()).isEqualTo("email");
            assertThat(result.errors().get(0).code()).isEqualTo("REQUIRED_FIELD_MISSING");
        }

        @Test
        @DisplayName("should return failure for unknown entity type")
        void validateUnknownEntityType() {
            ValidationResult result = plugin.validateDomainData("Unknown", Map.of());

            assertThat(result.valid()).isFalse();
            assertThat(result.errors().get(0).code()).isEqualTo("UNKNOWN_ENTITY_TYPE");
        }

        @Test
        @DisplayName("should validate null values as missing")
        void validateNullValues() {
            ValidationResult result = plugin.validateDomainData("User", Map.of(
                "name", "John",
                "email", "" // This won't be considered null, but let's test actual null
            ));

            // Need to use HashMap to allow null values
            java.util.HashMap<String, Object> data = new java.util.HashMap<>();
            data.put("name", "John");
            data.put("email", null);

            result = plugin.validateDomainData("User", data);

            assertThat(result.valid()).isFalse();
            assertThat(result.errors().get(0).field()).isEqualTo("email");
        }

        @Test
        @DisplayName("should create rule ID")
        void createRuleId() {
            String ruleId = plugin.createRuleId();

            assertThat(ruleId).startsWith("rule_");
            assertThat(ruleId).hasSize(17); // "rule_" + 12 chars
        }

        @Test
        @DisplayName("should create rule with metadata")
        void createRule() {
            Condition condition = new Condition.Compare("amount", "gt", 1000);
            Consequence consequence = Consequence.flag("HIGH_VALUE", "High value");

            PactRule rule = plugin.createRule("Test Rule", condition, consequence);

            assertThat(rule.id()).startsWith("rule_");
            assertThat(rule.name()).isEqualTo("Test Rule");
            assertThat(rule.severity()).isEqualTo(Severity.MEDIUM);
            assertThat(rule.tags()).contains("test-plugin");
            assertThat(rule.metadata()).containsEntry("plugin_id", "test-plugin");
            assertThat(rule.metadata()).containsEntry("plugin_version", "1.0.0");
        }

        @Test
        @DisplayName("should set and get hooks")
        void setAndGetHooks() {
            PluginHooks hooks = new PluginHooks();
            plugin.setHooks(hooks);

            assertThat(plugin.getHooks()).isEqualTo(hooks);
        }

        @Test
        @DisplayName("should return default evaluation context")
        void getEvaluationContext() {
            PactEvent event = PactEvent.builder().build();
            PactEntity entity = PactEntity.builder().build();

            Map<String, Object> context = plugin.getEvaluationContext(event, entity);

            assertThat(context).isEmpty();
        }
    }

    @Nested
    @DisplayName("PluginHooks")
    class PluginHooksTests {

        @Test
        @DisplayName("should trigger onLoad hook")
        void triggerOnLoad() {
            AtomicBoolean called = new AtomicBoolean(false);

            PluginHooks hooks = new PluginHooks()
                .onLoad(() -> called.set(true));

            hooks.triggerOnLoad();

            assertThat(called.get()).isTrue();
        }

        @Test
        @DisplayName("should trigger onUnload hook")
        void triggerOnUnload() {
            AtomicBoolean called = new AtomicBoolean(false);

            PluginHooks hooks = new PluginHooks()
                .onUnload(() -> called.set(true));

            hooks.triggerOnUnload();

            assertThat(called.get()).isTrue();
        }

        @Test
        @DisplayName("should trigger beforeEventProcess hook")
        void triggerBeforeEventProcess() {
            AtomicReference<String> captured = new AtomicReference<>();

            PluginHooks hooks = new PluginHooks()
                .beforeEventProcess(event -> {
                    captured.set(event.id());
                    return PactEvent.builder()
                        .id(event.id() + "_modified")
                        .source(event.source())
                        .type(event.type())
                        .build();
                });

            PactEvent event = PactEvent.builder()
                .id("evt_123")
                .source("test")
                .type("test_event")
                .build();

            PactEvent result = hooks.triggerBeforeEventProcess(event);

            assertThat(captured.get()).isEqualTo("evt_123");
            assertThat(result.id()).isEqualTo("evt_123_modified");
        }

        @Test
        @DisplayName("should return original event if no beforeEventProcess hook")
        void noBeforeEventProcessHook() {
            PluginHooks hooks = new PluginHooks();

            PactEvent event = PactEvent.builder()
                .id("evt_original")
                .build();

            PactEvent result = hooks.triggerBeforeEventProcess(event);

            assertThat(result).isEqualTo(event);
        }

        @Test
        @DisplayName("should trigger afterDecision hook")
        void triggerAfterDecision() {
            AtomicReference<String> capturedEventId = new AtomicReference<>();

            PluginHooks hooks = new PluginHooks()
                .afterDecision(event -> capturedEventId.set(event.id()));

            PactEvent event = PactEvent.builder()
                .id("evt_after")
                .build();

            hooks.triggerAfterDecision(event);

            assertThat(capturedEventId.get()).isEqualTo("evt_after");
        }

        @Test
        @DisplayName("should trigger onRulesUpdated hook")
        void triggerOnRulesUpdated() {
            AtomicReference<Integer> capturedSize = new AtomicReference<>();

            PluginHooks hooks = new PluginHooks()
                .onRulesUpdated(rules -> capturedSize.set(rules.size()));

            List<PactRule> rules = List.of(
                PactRule.builder()
                    .name("Rule 1")
                    .condition(new Condition.Compare("a", "eq", 1))
                    .consequence(Consequence.allow("OK", "OK"))
                    .build(),
                PactRule.builder()
                    .name("Rule 2")
                    .condition(new Condition.Compare("b", "eq", 2))
                    .consequence(Consequence.allow("OK", "OK"))
                    .build()
            );

            hooks.triggerOnRulesUpdated(rules);

            assertThat(capturedSize.get()).isEqualTo(2);
        }

        @Test
        @DisplayName("should handle null hooks gracefully")
        void handleNullHooks() {
            PluginHooks hooks = new PluginHooks();

            // These should not throw
            assertThatCode(() -> hooks.triggerOnLoad()).doesNotThrowAnyException();
            assertThatCode(() -> hooks.triggerOnUnload()).doesNotThrowAnyException();
            assertThatCode(() -> hooks.triggerAfterDecision(PactEvent.builder().build()))
                .doesNotThrowAnyException();
            assertThatCode(() -> hooks.triggerOnRulesUpdated(List.of()))
                .doesNotThrowAnyException();
        }

        @Test
        @DisplayName("should support method chaining")
        void methodChaining() {
            AtomicBoolean loadCalled = new AtomicBoolean(false);
            AtomicBoolean unloadCalled = new AtomicBoolean(false);

            PluginHooks hooks = new PluginHooks()
                .onLoad(() -> loadCalled.set(true))
                .onUnload(() -> unloadCalled.set(true))
                .beforeEventProcess(e -> e)
                .afterDecision(e -> {})
                .onRulesUpdated(r -> {});

            hooks.triggerOnLoad();
            hooks.triggerOnUnload();

            assertThat(loadCalled.get()).isTrue();
            assertThat(unloadCalled.get()).isTrue();
        }
    }

    @Nested
    @DisplayName("CanonicalJson")
    class CanonicalJsonTests {

        @Test
        @DisplayName("should serialize object to bytes")
        void serializeToBytes() {
            Map<String, Object> data = Map.of("name", "test", "value", 42);

            byte[] bytes = CanonicalJson.serialize(data);

            assertThat(bytes).isNotEmpty();
            String json = new String(bytes);
            assertThat(json).contains("name");
            assertThat(json).contains("test");
        }

        @Test
        @DisplayName("should deserialize bytes to object")
        void deserializeFromBytes() {
            String json = "{\"name\":\"test\",\"count\":10}";
            byte[] bytes = json.getBytes();

            Object result = CanonicalJson.deserialize(bytes);

            assertThat(result).isInstanceOf(Map.class);
            @SuppressWarnings("unchecked")
            Map<String, Object> map = (Map<String, Object>) result;
            assertThat(map).containsEntry("name", "test");
            assertThat(map).containsEntry("count", 10);
        }

        @Test
        @DisplayName("should convert object to JSON string")
        void toJson() {
            Map<String, Object> data = Map.of("key", "value");

            String json = CanonicalJson.toJson(data);

            assertThat(json).contains("\"key\"");
            assertThat(json).contains("\"value\"");
        }

        @Test
        @DisplayName("should convert JSON string to typed object")
        void fromJson() {
            String json = "{\"id\":\"evt_123\",\"source\":\"test\",\"type\":\"test_type\"}";

            PactEvent event = CanonicalJson.fromJson(json, PactEvent.class);

            assertThat(event.id()).isEqualTo("evt_123");
            assertThat(event.source()).isEqualTo("test");
        }

        @Test
        @DisplayName("should throw on invalid JSON for serialize")
        void serializeInvalid() {
            Object circular = new Object() {
                public Object self = this; // This will fail due to circular reference detection
            };

            // This test verifies error handling - Jackson will throw on circular references
            assertThatThrownBy(() -> CanonicalJson.serialize(circular))
                .isInstanceOf(RuntimeException.class);
        }

        @Test
        @DisplayName("should throw on invalid JSON for deserialize")
        void deserializeInvalid() {
            byte[] invalidJson = "{ invalid json }".getBytes();

            assertThatThrownBy(() -> CanonicalJson.deserialize(invalidJson))
                .isInstanceOf(RuntimeException.class)
                .hasMessageContaining("Failed to deserialize");
        }

        @Test
        @DisplayName("should throw on invalid JSON string for fromJson")
        void fromJsonInvalid() {
            String invalidJson = "not json at all";

            assertThatThrownBy(() -> CanonicalJson.fromJson(invalidJson, Map.class))
                .isInstanceOf(RuntimeException.class)
                .hasMessageContaining("Failed to deserialize");
        }

        @Test
        @DisplayName("should serialize with sorted keys")
        void sortedKeys() {
            // Use LinkedHashMap to ensure we have multiple keys
            java.util.LinkedHashMap<String, Object> data = new java.util.LinkedHashMap<>();
            data.put("zebra", 1);
            data.put("apple", 2);
            data.put("mango", 3);

            String json = CanonicalJson.toJson(data);

            // Keys should be sorted alphabetically
            int appleIndex = json.indexOf("apple");
            int mangoIndex = json.indexOf("mango");
            int zebraIndex = json.indexOf("zebra");

            assertThat(appleIndex).isLessThan(mangoIndex);
            assertThat(mangoIndex).isLessThan(zebraIndex);
        }

        @Test
        @DisplayName("should roundtrip complex object")
        void roundtripComplex() {
            PactEvent original = PactEvent.builder()
                .id("evt_roundtrip")
                .source("test-source")
                .type("test_event")
                .entityId("ent_123")
                .entityType("TestEntity")
                .jurisdiction("US")
                .payload(Map.of("amount", 100, "currency", "USD"))
                .metadata(Map.of("trace_id", "abc123"))
                .build();

            String json = CanonicalJson.toJson(original);
            PactEvent restored = CanonicalJson.fromJson(json, PactEvent.class);

            assertThat(restored.id()).isEqualTo(original.id());
            assertThat(restored.source()).isEqualTo(original.source());
            assertThat(restored.type()).isEqualTo(original.type());
            assertThat(restored.entityId()).isEqualTo(original.entityId());
        }
    }

    @Nested
    @DisplayName("DomainPlugin Interface")
    class DomainPluginInterfaceTests {

        @Test
        @DisplayName("should use default serializeForChain implementation")
        void serializeForChain() {
            TestPlugin plugin = new TestPlugin();
            Map<String, Object> data = Map.of("key", "value");

            byte[] serialized = plugin.serializeForChain(data);

            assertThat(serialized).isNotEmpty();
            String json = new String(serialized);
            assertThat(json).contains("key");
        }

        @Test
        @DisplayName("should use default deserializeFromChain implementation")
        void deserializeFromChain() {
            TestPlugin plugin = new TestPlugin();
            String json = "{\"test\":\"data\"}";

            Object result = plugin.deserializeFromChain(json.getBytes());

            assertThat(result).isInstanceOf(Map.class);
            @SuppressWarnings("unchecked")
            Map<String, Object> map = (Map<String, Object>) result;
            assertThat(map).containsEntry("test", "data");
        }

        @Test
        @DisplayName("should use default getHooks returning null")
        void defaultGetHooks() {
            // Test that DomainPlugin interface default returns null
            DomainPlugin anonymousPlugin = new DomainPlugin() {
                @Override
                public PluginManifest getManifest() {
                    return PluginManifest.builder()
                        .id("anon")
                        .name("Anonymous")
                        .version("1.0.0")
                        .build();
                }

                @Override
                public List<PactEntityType> getEntityTypes() {
                    return List.of();
                }

                @Override
                public List<PactRule> getDefaultRules() {
                    return List.of();
                }

                @Override
                public PactEvent mapToCanonicalEvent(DomainEvent event) {
                    return null;
                }

                @Override
                public ValidationResult validateDomainData(String entityType, Map<String, Object> data) {
                    return ValidationResult.success();
                }
            };

            assertThat(anonymousPlugin.getHooks()).isNull();
        }

        @Test
        @DisplayName("should use default getEvaluationContext returning empty map")
        void defaultGetEvaluationContext() {
            TestPlugin plugin = new TestPlugin();
            PactEvent event = PactEvent.builder().build();
            PactEntity entity = PactEntity.builder().build();

            Map<String, Object> context = plugin.getEvaluationContext(event, entity);

            assertThat(context).isEmpty();
        }
    }

    /**
     * Test implementation of BasePlugin for testing purposes.
     */
    static class TestPlugin extends BasePlugin {

        public TestPlugin() {
            super(
                PluginManifest.builder()
                    .id("test-plugin")
                    .name("Test Plugin")
                    .version("1.0.0")
                    .author("Test Author")
                    .description("A test plugin")
                    .entityTypes(List.of("User", "Transaction"))
                    .eventTypes(List.of("user_created", "transaction_processed"))
                    .build(),
                List.of(
                    PactEntityType.builder()
                        .name("User")
                        .description("A user entity")
                        .requiredFields(List.of("name", "email"))
                        .build(),
                    PactEntityType.builder()
                        .name("Transaction")
                        .description("A transaction entity")
                        .requiredFields(List.of("amount", "currency"))
                        .build()
                ),
                List.of(
                    PactRule.builder()
                        .id("default_rule")
                        .name("Default Rule")
                        .condition(new Condition.Compare("active", "eq", true))
                        .consequence(Consequence.allow("OK", "Entity is active"))
                        .build()
                )
            );
        }
    }
}
