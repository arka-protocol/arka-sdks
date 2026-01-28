package com.arka.sdk.types;

import com.fasterxml.jackson.annotation.JsonProperty;
import java.time.Instant;
import java.util.Map;

/**
 * Canonical PACT event format.
 */
public record PactEvent(
    String id,
    String source,
    String type,
    @JsonProperty("entity_id") String entityId,
    @JsonProperty("entity_type") String entityType,
    String jurisdiction,
    Map<String, Object> payload,
    @JsonProperty("occurred_at") Instant occurredAt,
    @JsonProperty("received_at") Instant receivedAt,
    Map<String, Object> metadata
) {
    public PactEvent {
        if (payload == null) payload = Map.of();
        if (metadata == null) metadata = Map.of();
    }

    public static Builder builder() {
        return new Builder();
    }

    public static class Builder {
        private String id;
        private String source;
        private String type;
        private String entityId;
        private String entityType;
        private String jurisdiction;
        private Map<String, Object> payload = Map.of();
        private Instant occurredAt;
        private Instant receivedAt;
        private Map<String, Object> metadata = Map.of();

        public Builder id(String id) { this.id = id; return this; }
        public Builder source(String source) { this.source = source; return this; }
        public Builder type(String type) { this.type = type; return this; }
        public Builder entityId(String entityId) { this.entityId = entityId; return this; }
        public Builder entityType(String entityType) { this.entityType = entityType; return this; }
        public Builder jurisdiction(String jurisdiction) { this.jurisdiction = jurisdiction; return this; }
        public Builder payload(Map<String, Object> payload) { this.payload = payload; return this; }
        public Builder occurredAt(Instant occurredAt) { this.occurredAt = occurredAt; return this; }
        public Builder receivedAt(Instant receivedAt) { this.receivedAt = receivedAt; return this; }
        public Builder metadata(Map<String, Object> metadata) { this.metadata = metadata; return this; }

        public PactEvent build() {
            return new PactEvent(id, source, type, entityId, entityType, jurisdiction, payload, occurredAt, receivedAt, metadata);
        }
    }
}
