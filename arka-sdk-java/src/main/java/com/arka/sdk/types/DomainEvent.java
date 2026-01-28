package com.arka.sdk.types;

import com.fasterxml.jackson.annotation.JsonProperty;
import java.time.Instant;
import java.util.Map;

/**
 * Domain-specific event before conversion to canonical format.
 */
public record DomainEvent(
    String type,
    Map<String, Object> payload,
    @JsonProperty("entity_id") String entityId,
    String jurisdiction,
    @JsonProperty("occurred_at") Instant occurredAt,
    Map<String, Object> metadata
) {
    public DomainEvent {
        if (payload == null) payload = Map.of();
        if (metadata == null) metadata = Map.of();
    }

    public static Builder builder() {
        return new Builder();
    }

    public static class Builder {
        private String type;
        private Map<String, Object> payload = Map.of();
        private String entityId;
        private String jurisdiction;
        private Instant occurredAt;
        private Map<String, Object> metadata = Map.of();

        public Builder type(String type) { this.type = type; return this; }
        public Builder payload(Map<String, Object> payload) { this.payload = payload; return this; }
        public Builder entityId(String entityId) { this.entityId = entityId; return this; }
        public Builder jurisdiction(String jurisdiction) { this.jurisdiction = jurisdiction; return this; }
        public Builder occurredAt(Instant occurredAt) { this.occurredAt = occurredAt; return this; }
        public Builder metadata(Map<String, Object> metadata) { this.metadata = metadata; return this; }

        public DomainEvent build() {
            return new DomainEvent(type, payload, entityId, jurisdiction, occurredAt, metadata);
        }
    }
}
