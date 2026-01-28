package com.arka.sdk.client;

import com.fasterxml.jackson.databind.ObjectMapper;
import com.fasterxml.jackson.datatype.jsr310.JavaTimeModule;
import com.arka.sdk.types.*;
import okhttp3.*;

import java.io.IOException;
import java.time.Duration;
import java.util.Map;
import java.util.concurrent.CompletableFuture;

/**
 * HTTP client for communicating with PACT Core.
 */
public class PactClient implements AutoCloseable {
    private static final MediaType JSON = MediaType.get("application/json");

    private final String baseUrl;
    private final String apiKey;
    private final OkHttpClient httpClient;
    private final ObjectMapper mapper;

    public PactClient(String baseUrl) {
        this(baseUrl, null, Duration.ofSeconds(30));
    }

    public PactClient(String baseUrl, String apiKey) {
        this(baseUrl, apiKey, Duration.ofSeconds(30));
    }

    public PactClient(String baseUrl, String apiKey, Duration timeout) {
        this.baseUrl = baseUrl.replaceAll("/$", "");
        this.apiKey = apiKey;
        this.httpClient = new OkHttpClient.Builder()
            .connectTimeout(timeout)
            .readTimeout(timeout)
            .writeTimeout(timeout)
            .build();
        this.mapper = new ObjectMapper()
            .registerModule(new JavaTimeModule());
    }

    // Event Operations

    public PactDecision submitEvent(PactEvent event) throws IOException {
        String json = mapper.writeValueAsString(event);
        RequestBody body = RequestBody.create(json, JSON);
        Request request = buildRequest("/api/v1/events", "POST", body);
        return execute(request, PactDecision.class);
    }

    public PactEvent getEvent(String eventId) throws IOException {
        Request request = buildRequest("/api/v1/events/" + eventId, "GET", null);
        return execute(request, PactEvent.class);
    }

    // Entity Operations

    public PactEntity createEntity(PactEntity entity) throws IOException {
        String json = mapper.writeValueAsString(entity);
        RequestBody body = RequestBody.create(json, JSON);
        Request request = buildRequest("/api/v1/entities", "POST", body);
        return execute(request, PactEntity.class);
    }

    public PactEntity getEntity(String entityId) throws IOException {
        Request request = buildRequest("/api/v1/entities/" + entityId, "GET", null);
        return execute(request, PactEntity.class);
    }

    public PactEntity updateEntity(String entityId, Map<String, Object> data) throws IOException {
        String json = mapper.writeValueAsString(Map.of("data", data));
        RequestBody body = RequestBody.create(json, JSON);
        Request request = buildRequest("/api/v1/entities/" + entityId, "PATCH", body);
        return execute(request, PactEntity.class);
    }

    public void deleteEntity(String entityId) throws IOException {
        Request request = buildRequest("/api/v1/entities/" + entityId, "DELETE", null);
        executeVoid(request);
    }

    // Rule Operations

    public PactRule createRule(PactRule rule) throws IOException {
        String json = mapper.writeValueAsString(rule);
        RequestBody body = RequestBody.create(json, JSON);
        Request request = buildRequest("/api/v1/rules", "POST", body);
        return execute(request, PactRule.class);
    }

    public PactRule getRule(String ruleId) throws IOException {
        Request request = buildRequest("/api/v1/rules/" + ruleId, "GET", null);
        return execute(request, PactRule.class);
    }

    public PactRule updateRule(String ruleId, PactRule rule) throws IOException {
        String json = mapper.writeValueAsString(rule);
        RequestBody body = RequestBody.create(json, JSON);
        Request request = buildRequest("/api/v1/rules/" + ruleId, "PUT", body);
        return execute(request, PactRule.class);
    }

    public void deleteRule(String ruleId) throws IOException {
        Request request = buildRequest("/api/v1/rules/" + ruleId, "DELETE", null);
        executeVoid(request);
    }

    // Decision Operations

    public PactDecision getDecision(String decisionId) throws IOException {
        Request request = buildRequest("/api/v1/decisions/" + decisionId, "GET", null);
        return execute(request, PactDecision.class);
    }

    // Validation Operations

    public ValidationResult validate(String entityType, Map<String, Object> data) throws IOException {
        String json = mapper.writeValueAsString(Map.of(
            "entity_type", entityType,
            "data", data
        ));
        RequestBody body = RequestBody.create(json, JSON);
        Request request = buildRequest("/api/v1/validate", "POST", body);
        return execute(request, ValidationResult.class);
    }

    // Health Operations

    public boolean isHealthy() {
        try {
            Request request = buildRequest("/health", "GET", null);
            try (Response response = httpClient.newCall(request).execute()) {
                return response.isSuccessful();
            }
        } catch (Exception e) {
            return false;
        }
    }

    public boolean isReady() {
        try {
            Request request = buildRequest("/ready", "GET", null);
            try (Response response = httpClient.newCall(request).execute()) {
                return response.isSuccessful();
            }
        } catch (Exception e) {
            return false;
        }
    }

    // Async variants

    public CompletableFuture<PactDecision> submitEventAsync(PactEvent event) {
        return CompletableFuture.supplyAsync(() -> {
            try {
                return submitEvent(event);
            } catch (IOException e) {
                throw new RuntimeException(e);
            }
        });
    }

    public CompletableFuture<PactEvent> getEventAsync(String eventId) {
        return CompletableFuture.supplyAsync(() -> {
            try {
                return getEvent(eventId);
            } catch (IOException e) {
                throw new RuntimeException(e);
            }
        });
    }

    // Helper methods

    private Request buildRequest(String path, String method, RequestBody body) {
        Request.Builder builder = new Request.Builder()
            .url(baseUrl + path)
            .header("Content-Type", "application/json")
            .header("Accept", "application/json");

        if (apiKey != null) {
            builder.header("Authorization", "Bearer " + apiKey);
        }

        return builder.method(method, body).build();
    }

    private <T> T execute(Request request, Class<T> type) throws IOException {
        try (Response response = httpClient.newCall(request).execute()) {
            if (!response.isSuccessful()) {
                String errorBody = response.body() != null ? response.body().string() : "";
                throw new IOException("HTTP " + response.code() + ": " + errorBody);
            }

            if (response.body() == null) {
                throw new IOException("Empty response body");
            }

            return mapper.readValue(response.body().string(), type);
        }
    }

    private void executeVoid(Request request) throws IOException {
        try (Response response = httpClient.newCall(request).execute()) {
            if (!response.isSuccessful()) {
                String errorBody = response.body() != null ? response.body().string() : "";
                throw new IOException("HTTP " + response.code() + ": " + errorBody);
            }
        }
    }

    @Override
    public void close() {
        httpClient.dispatcher().executorService().shutdown();
        httpClient.connectionPool().evictAll();
    }
}
