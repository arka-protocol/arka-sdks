package com.arka.sdk.client;

import com.arka.sdk.types.*;
import com.fasterxml.jackson.databind.ObjectMapper;
import com.fasterxml.jackson.datatype.jsr310.JavaTimeModule;
import okhttp3.mockwebserver.MockResponse;
import okhttp3.mockwebserver.MockWebServer;
import okhttp3.mockwebserver.RecordedRequest;
import org.junit.jupiter.api.*;

import java.io.IOException;
import java.time.Duration;
import java.time.Instant;
import java.util.List;
import java.util.Map;

import static org.assertj.core.api.Assertions.*;

/**
 * Comprehensive tests for PactClient.
 */
@DisplayName("PactClient")
class PactClientTest {

    private MockWebServer mockServer;
    private PactClient client;
    private ObjectMapper mapper;

    @BeforeEach
    void setUp() throws IOException {
        mockServer = new MockWebServer();
        mockServer.start();

        String baseUrl = mockServer.url("/").toString();
        client = new PactClient(baseUrl, "test-api-key");

        mapper = new ObjectMapper()
            .registerModule(new JavaTimeModule());
    }

    @AfterEach
    void tearDown() throws IOException {
        client.close();
        mockServer.shutdown();
    }

    @Nested
    @DisplayName("Client Initialization")
    class ClientInitialization {

        @Test
        @DisplayName("should initialize with base URL only")
        void initializeWithBaseUrlOnly() {
            try (PactClient simpleClient = new PactClient("http://localhost:8080")) {
                assertThat(simpleClient).isNotNull();
            }
        }

        @Test
        @DisplayName("should initialize with base URL and API key")
        void initializeWithApiKey() {
            try (PactClient keyClient = new PactClient("http://localhost:8080", "api-key")) {
                assertThat(keyClient).isNotNull();
            }
        }

        @Test
        @DisplayName("should initialize with custom timeout")
        void initializeWithCustomTimeout() {
            try (PactClient timeoutClient = new PactClient(
                "http://localhost:8080",
                "api-key",
                Duration.ofSeconds(60)
            )) {
                assertThat(timeoutClient).isNotNull();
            }
        }

        @Test
        @DisplayName("should strip trailing slash from base URL")
        void stripTrailingSlash() throws Exception {
            mockServer.enqueue(new MockResponse()
                .setResponseCode(200)
                .setBody("{\"valid\": true, \"errors\": [], \"warnings\": []}"));

            try (PactClient slashClient = new PactClient(
                mockServer.url("/").toString() + "/",
                "test-key"
            )) {
                slashClient.validate("User", Map.of("name", "test"));

                RecordedRequest request = mockServer.takeRequest();
                assertThat(request.getPath()).isEqualTo("/api/v1/validate");
            }
        }
    }

    @Nested
    @DisplayName("Event Operations")
    class EventOperations {

        @Test
        @DisplayName("should submit event and receive decision")
        void submitEvent() throws Exception {
            Instant now = Instant.now();
            PactDecision expectedDecision = PactDecision.builder()
                .id("dec_123")
                .eventId("evt_456")
                .status(DecisionStatus.ALLOW)
                .triggeredRules(List.of())
                .decidedAt(now)
                .executionTimeMs(15L)
                .build();

            mockServer.enqueue(new MockResponse()
                .setResponseCode(200)
                .setHeader("Content-Type", "application/json")
                .setBody(mapper.writeValueAsString(expectedDecision)));

            PactEvent event = PactEvent.builder()
                .id("evt_456")
                .source("test-plugin")
                .type("user_created")
                .entityId("user_123")
                .entityType("User")
                .jurisdiction("US")
                .payload(Map.of("email", "test@example.com"))
                .occurredAt(now)
                .receivedAt(now)
                .build();

            PactDecision decision = client.submitEvent(event);

            assertThat(decision.id()).isEqualTo("dec_123");
            assertThat(decision.status()).isEqualTo(DecisionStatus.ALLOW);
            assertThat(decision.isAllowed()).isTrue();
            assertThat(decision.isDenied()).isFalse();

            RecordedRequest request = mockServer.takeRequest();
            assertThat(request.getMethod()).isEqualTo("POST");
            assertThat(request.getPath()).isEqualTo("/api/v1/events");
            assertThat(request.getHeader("Authorization")).isEqualTo("Bearer test-api-key");
            assertThat(request.getHeader("Content-Type")).isEqualTo("application/json");
        }

        @Test
        @DisplayName("should get event by ID")
        void getEvent() throws Exception {
            Instant now = Instant.now();
            PactEvent expectedEvent = PactEvent.builder()
                .id("evt_789")
                .source("test-plugin")
                .type("payment_processed")
                .entityId("payment_456")
                .entityType("Payment")
                .occurredAt(now)
                .receivedAt(now)
                .build();

            mockServer.enqueue(new MockResponse()
                .setResponseCode(200)
                .setHeader("Content-Type", "application/json")
                .setBody(mapper.writeValueAsString(expectedEvent)));

            PactEvent event = client.getEvent("evt_789");

            assertThat(event.id()).isEqualTo("evt_789");
            assertThat(event.type()).isEqualTo("payment_processed");

            RecordedRequest request = mockServer.takeRequest();
            assertThat(request.getMethod()).isEqualTo("GET");
            assertThat(request.getPath()).isEqualTo("/api/v1/events/evt_789");
        }

        @Test
        @DisplayName("should submit event asynchronously")
        void submitEventAsync() throws Exception {
            Instant now = Instant.now();
            PactDecision expectedDecision = PactDecision.builder()
                .id("dec_async")
                .status(DecisionStatus.ALLOW_WITH_FLAGS)
                .triggeredRules(List.of(
                    new PactDecision.TriggeredRule(
                        "rule_1", "High Value Check", Decision.FLAG,
                        "HIGH_VALUE", "Transaction exceeds threshold", Severity.MEDIUM
                    )
                ))
                .decidedAt(now)
                .build();

            mockServer.enqueue(new MockResponse()
                .setResponseCode(200)
                .setHeader("Content-Type", "application/json")
                .setBody(mapper.writeValueAsString(expectedDecision)));

            PactEvent event = PactEvent.builder()
                .id("evt_async")
                .source("test")
                .type("transaction_submitted")
                .build();

            PactDecision decision = client.submitEventAsync(event).join();

            assertThat(decision.status()).isEqualTo(DecisionStatus.ALLOW_WITH_FLAGS);
            assertThat(decision.isAllowed()).isTrue();
            assertThat(decision.hasFlagged()).isTrue();
        }
    }

    @Nested
    @DisplayName("Entity Operations")
    class EntityOperations {

        @Test
        @DisplayName("should create entity")
        void createEntity() throws Exception {
            Instant now = Instant.now();
            PactEntity expectedEntity = PactEntity.builder()
                .id("ent_123")
                .type("User")
                .data(Map.of("name", "John Doe", "email", "john@example.com"))
                .createdAt(now)
                .updatedAt(now)
                .jurisdiction("US")
                .build();

            mockServer.enqueue(new MockResponse()
                .setResponseCode(200)
                .setHeader("Content-Type", "application/json")
                .setBody(mapper.writeValueAsString(expectedEntity)));

            PactEntity entity = PactEntity.builder()
                .type("User")
                .data(Map.of("name", "John Doe", "email", "john@example.com"))
                .jurisdiction("US")
                .build();

            PactEntity created = client.createEntity(entity);

            assertThat(created.id()).isEqualTo("ent_123");
            assertThat(created.type()).isEqualTo("User");
            assertThat(created.data()).containsEntry("name", "John Doe");

            RecordedRequest request = mockServer.takeRequest();
            assertThat(request.getMethod()).isEqualTo("POST");
            assertThat(request.getPath()).isEqualTo("/api/v1/entities");
        }

        @Test
        @DisplayName("should get entity by ID")
        void getEntity() throws Exception {
            PactEntity expectedEntity = PactEntity.builder()
                .id("ent_456")
                .type("Account")
                .data(Map.of("balance", 1000.0))
                .build();

            mockServer.enqueue(new MockResponse()
                .setResponseCode(200)
                .setHeader("Content-Type", "application/json")
                .setBody(mapper.writeValueAsString(expectedEntity)));

            PactEntity entity = client.getEntity("ent_456");

            assertThat(entity.id()).isEqualTo("ent_456");
            assertThat(entity.type()).isEqualTo("Account");
        }

        @Test
        @DisplayName("should update entity")
        void updateEntity() throws Exception {
            PactEntity expectedEntity = PactEntity.builder()
                .id("ent_789")
                .type("Account")
                .data(Map.of("balance", 2000.0))
                .build();

            mockServer.enqueue(new MockResponse()
                .setResponseCode(200)
                .setHeader("Content-Type", "application/json")
                .setBody(mapper.writeValueAsString(expectedEntity)));

            PactEntity updated = client.updateEntity("ent_789", Map.of("balance", 2000.0));

            assertThat(updated.data()).containsEntry("balance", 2000.0);

            RecordedRequest request = mockServer.takeRequest();
            assertThat(request.getMethod()).isEqualTo("PATCH");
            assertThat(request.getPath()).isEqualTo("/api/v1/entities/ent_789");
        }

        @Test
        @DisplayName("should delete entity")
        void deleteEntity() throws Exception {
            mockServer.enqueue(new MockResponse().setResponseCode(204));

            client.deleteEntity("ent_to_delete");

            RecordedRequest request = mockServer.takeRequest();
            assertThat(request.getMethod()).isEqualTo("DELETE");
            assertThat(request.getPath()).isEqualTo("/api/v1/entities/ent_to_delete");
        }
    }

    @Nested
    @DisplayName("Rule Operations")
    class RuleOperations {

        @Test
        @DisplayName("should create rule")
        void createRule() throws Exception {
            Instant now = Instant.now();
            PactRule expectedRule = PactRule.builder()
                .id("rule_123")
                .name("High Value Transaction")
                .description("Flag transactions over $10,000")
                .severity(Severity.HIGH)
                .condition(new Condition.Compare("amount", "gt", 10000))
                .consequence(Consequence.flag("HIGH_VALUE", "Amount exceeds $10,000"))
                .tags(List.of("compliance", "aml"))
                .effectiveFrom(now)
                .build();

            mockServer.enqueue(new MockResponse()
                .setResponseCode(200)
                .setHeader("Content-Type", "application/json")
                .setBody(mapper.writeValueAsString(expectedRule)));

            PactRule rule = PactRule.builder()
                .name("High Value Transaction")
                .severity(Severity.HIGH)
                .condition(new Condition.Compare("amount", "gt", 10000))
                .consequence(Consequence.flag("HIGH_VALUE", "Amount exceeds $10,000"))
                .build();

            PactRule created = client.createRule(rule);

            assertThat(created.id()).isEqualTo("rule_123");
            assertThat(created.name()).isEqualTo("High Value Transaction");
            assertThat(created.severity()).isEqualTo(Severity.HIGH);

            RecordedRequest request = mockServer.takeRequest();
            assertThat(request.getMethod()).isEqualTo("POST");
            assertThat(request.getPath()).isEqualTo("/api/v1/rules");
        }

        @Test
        @DisplayName("should get rule by ID")
        void getRule() throws Exception {
            PactRule expectedRule = PactRule.builder()
                .id("rule_456")
                .name("Blocked Country")
                .condition(new Condition.In("country", List.of("XX", "YY")))
                .consequence(Consequence.deny("BLOCKED_COUNTRY", "Country is blocked"))
                .build();

            mockServer.enqueue(new MockResponse()
                .setResponseCode(200)
                .setHeader("Content-Type", "application/json")
                .setBody(mapper.writeValueAsString(expectedRule)));

            PactRule rule = client.getRule("rule_456");

            assertThat(rule.id()).isEqualTo("rule_456");
            assertThat(rule.name()).isEqualTo("Blocked Country");
        }

        @Test
        @DisplayName("should update rule")
        void updateRule() throws Exception {
            PactRule expectedRule = PactRule.builder()
                .id("rule_789")
                .name("Updated Rule")
                .severity(Severity.CRITICAL)
                .condition(new Condition.Compare("amount", "gt", 50000))
                .consequence(Consequence.deny("LIMIT_EXCEEDED", "Limit exceeded"))
                .build();

            mockServer.enqueue(new MockResponse()
                .setResponseCode(200)
                .setHeader("Content-Type", "application/json")
                .setBody(mapper.writeValueAsString(expectedRule)));

            PactRule rule = PactRule.builder()
                .id("rule_789")
                .name("Updated Rule")
                .severity(Severity.CRITICAL)
                .condition(new Condition.Compare("amount", "gt", 50000))
                .consequence(Consequence.deny("LIMIT_EXCEEDED", "Limit exceeded"))
                .build();

            PactRule updated = client.updateRule("rule_789", rule);

            assertThat(updated.severity()).isEqualTo(Severity.CRITICAL);

            RecordedRequest request = mockServer.takeRequest();
            assertThat(request.getMethod()).isEqualTo("PUT");
        }

        @Test
        @DisplayName("should delete rule")
        void deleteRule() throws Exception {
            mockServer.enqueue(new MockResponse().setResponseCode(204));

            client.deleteRule("rule_to_delete");

            RecordedRequest request = mockServer.takeRequest();
            assertThat(request.getMethod()).isEqualTo("DELETE");
            assertThat(request.getPath()).isEqualTo("/api/v1/rules/rule_to_delete");
        }
    }

    @Nested
    @DisplayName("Decision Operations")
    class DecisionOperations {

        @Test
        @DisplayName("should get decision by ID")
        void getDecision() throws Exception {
            PactDecision expectedDecision = PactDecision.builder()
                .id("dec_123")
                .eventId("evt_456")
                .status(DecisionStatus.DENY)
                .triggeredRules(List.of(
                    new PactDecision.TriggeredRule(
                        "rule_1", "Blocked Entity", Decision.DENY,
                        "ENTITY_BLOCKED", "Entity is on blocklist", Severity.CRITICAL
                    )
                ))
                .decidedAt(Instant.now())
                .executionTimeMs(5L)
                .build();

            mockServer.enqueue(new MockResponse()
                .setResponseCode(200)
                .setHeader("Content-Type", "application/json")
                .setBody(mapper.writeValueAsString(expectedDecision)));

            PactDecision decision = client.getDecision("dec_123");

            assertThat(decision.id()).isEqualTo("dec_123");
            assertThat(decision.status()).isEqualTo(DecisionStatus.DENY);
            assertThat(decision.isDenied()).isTrue();
            assertThat(decision.isAllowed()).isFalse();
            assertThat(decision.triggeredRules()).hasSize(1);
            assertThat(decision.triggeredRules().get(0).ruleName()).isEqualTo("Blocked Entity");

            RecordedRequest request = mockServer.takeRequest();
            assertThat(request.getMethod()).isEqualTo("GET");
            assertThat(request.getPath()).isEqualTo("/api/v1/decisions/dec_123");
        }
    }

    @Nested
    @DisplayName("Validation Operations")
    class ValidationOperations {

        @Test
        @DisplayName("should validate data successfully")
        void validateDataSuccess() throws Exception {
            ValidationResult expectedResult = ValidationResult.success();

            mockServer.enqueue(new MockResponse()
                .setResponseCode(200)
                .setHeader("Content-Type", "application/json")
                .setBody(mapper.writeValueAsString(expectedResult)));

            ValidationResult result = client.validate("User", Map.of(
                "name", "John Doe",
                "email", "john@example.com"
            ));

            assertThat(result.valid()).isTrue();
            assertThat(result.errors()).isEmpty();

            RecordedRequest request = mockServer.takeRequest();
            assertThat(request.getMethod()).isEqualTo("POST");
            assertThat(request.getPath()).isEqualTo("/api/v1/validate");
        }

        @Test
        @DisplayName("should return validation errors")
        void validateDataFailure() throws Exception {
            ValidationResult expectedResult = ValidationResult.failure(
                "email", "Invalid email format", "INVALID_EMAIL"
            );

            mockServer.enqueue(new MockResponse()
                .setResponseCode(200)
                .setHeader("Content-Type", "application/json")
                .setBody(mapper.writeValueAsString(expectedResult)));

            ValidationResult result = client.validate("User", Map.of(
                "name", "John",
                "email", "invalid-email"
            ));

            assertThat(result.valid()).isFalse();
            assertThat(result.errors()).hasSize(1);
            assertThat(result.errors().get(0).field()).isEqualTo("email");
            assertThat(result.errors().get(0).code()).isEqualTo("INVALID_EMAIL");
        }
    }

    @Nested
    @DisplayName("Health Operations")
    class HealthOperations {

        @Test
        @DisplayName("should return healthy when server responds 200")
        void isHealthySuccess() {
            mockServer.enqueue(new MockResponse().setResponseCode(200));

            boolean healthy = client.isHealthy();

            assertThat(healthy).isTrue();
        }

        @Test
        @DisplayName("should return unhealthy when server responds with error")
        void isHealthyFailure() {
            mockServer.enqueue(new MockResponse().setResponseCode(503));

            boolean healthy = client.isHealthy();

            assertThat(healthy).isFalse();
        }

        @Test
        @DisplayName("should return ready when server responds 200")
        void isReadySuccess() {
            mockServer.enqueue(new MockResponse().setResponseCode(200));

            boolean ready = client.isReady();

            assertThat(ready).isTrue();
        }

        @Test
        @DisplayName("should return not ready when server responds with error")
        void isReadyFailure() {
            mockServer.enqueue(new MockResponse().setResponseCode(503));

            boolean ready = client.isReady();

            assertThat(ready).isFalse();
        }
    }

    @Nested
    @DisplayName("Error Handling")
    class ErrorHandling {

        @Test
        @DisplayName("should throw IOException on HTTP error")
        void httpError() {
            mockServer.enqueue(new MockResponse()
                .setResponseCode(500)
                .setBody("{\"error\": \"Internal Server Error\"}"));

            assertThatThrownBy(() -> client.getEvent("evt_123"))
                .isInstanceOf(IOException.class)
                .hasMessageContaining("HTTP 500");
        }

        @Test
        @DisplayName("should throw IOException on 404 Not Found")
        void notFoundError() {
            mockServer.enqueue(new MockResponse()
                .setResponseCode(404)
                .setBody("{\"error\": \"Event not found\"}"));

            assertThatThrownBy(() -> client.getEvent("nonexistent"))
                .isInstanceOf(IOException.class)
                .hasMessageContaining("HTTP 404");
        }

        @Test
        @DisplayName("should throw IOException on 401 Unauthorized")
        void unauthorizedError() {
            mockServer.enqueue(new MockResponse()
                .setResponseCode(401)
                .setBody("{\"error\": \"Invalid API key\"}"));

            assertThatThrownBy(() -> client.getEntity("ent_123"))
                .isInstanceOf(IOException.class)
                .hasMessageContaining("HTTP 401");
        }

        @Test
        @DisplayName("should throw IOException on 400 Bad Request")
        void badRequestError() {
            mockServer.enqueue(new MockResponse()
                .setResponseCode(400)
                .setBody("{\"error\": \"Invalid request body\"}"));

            PactEntity invalidEntity = PactEntity.builder().build();

            assertThatThrownBy(() -> client.createEntity(invalidEntity))
                .isInstanceOf(IOException.class)
                .hasMessageContaining("HTTP 400");
        }

        @Test
        @DisplayName("should handle async errors")
        void asyncError() {
            mockServer.enqueue(new MockResponse()
                .setResponseCode(500)
                .setBody("{\"error\": \"Server error\"}"));

            PactEvent event = PactEvent.builder()
                .id("evt_fail")
                .source("test")
                .type("test_event")
                .build();

            assertThatThrownBy(() -> client.submitEventAsync(event).join())
                .hasCauseInstanceOf(IOException.class);
        }
    }

    @Nested
    @DisplayName("Request Headers")
    class RequestHeaders {

        @Test
        @DisplayName("should include authorization header when API key is set")
        void authorizationHeader() throws Exception {
            mockServer.enqueue(new MockResponse()
                .setResponseCode(200)
                .setBody("{\"valid\": true, \"errors\": [], \"warnings\": []}"));

            client.validate("Test", Map.of());

            RecordedRequest request = mockServer.takeRequest();
            assertThat(request.getHeader("Authorization")).isEqualTo("Bearer test-api-key");
        }

        @Test
        @DisplayName("should not include authorization header when API key is null")
        void noAuthorizationHeader() throws Exception {
            try (PactClient noAuthClient = new PactClient(mockServer.url("/").toString())) {
                mockServer.enqueue(new MockResponse()
                    .setResponseCode(200)
                    .setBody("{\"valid\": true, \"errors\": [], \"warnings\": []}"));

                noAuthClient.validate("Test", Map.of());

                RecordedRequest request = mockServer.takeRequest();
                assertThat(request.getHeader("Authorization")).isNull();
            }
        }

        @Test
        @DisplayName("should include content-type header")
        void contentTypeHeader() throws Exception {
            mockServer.enqueue(new MockResponse()
                .setResponseCode(200)
                .setBody("{\"valid\": true, \"errors\": [], \"warnings\": []}"));

            client.validate("Test", Map.of());

            RecordedRequest request = mockServer.takeRequest();
            assertThat(request.getHeader("Content-Type")).isEqualTo("application/json");
        }

        @Test
        @DisplayName("should include accept header")
        void acceptHeader() throws Exception {
            mockServer.enqueue(new MockResponse()
                .setResponseCode(200)
                .setBody("{\"valid\": true, \"errors\": [], \"warnings\": []}"));

            client.validate("Test", Map.of());

            RecordedRequest request = mockServer.takeRequest();
            assertThat(request.getHeader("Accept")).isEqualTo("application/json");
        }
    }
}
