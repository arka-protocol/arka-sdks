package com.arka.sdk.types;

import com.fasterxml.jackson.annotation.JsonProperty;
import java.time.Instant;
import java.util.Map;

/**
 * PACT entity representation.
 */
public record PactEntity(
    String id,
    String type,
    Map<String, Object> data,
    @JsonProperty("created_at") Instant createdAt,
    @JsonProperty("updated_at") Instant updatedAt,
    String jurisdiction,
    Map<String, Object> metadata
) {
    public PactEntity {
        if (data == null) data = Map.of();
        if (metadata == null) metadata = Map.of();
    }

    public static Builder builder() {
        return new Builder();
    }

    public static class Builder {
        private String id;
        private String type;
        private Map<String, Object> data = Map.of();
        private Instant createdAt;
        private Instant updatedAt;
        private String jurisdiction;
        private Map<String, Object> metadata = Map.of();

        public Builder id(String id) { this.id = id; return this; }
        public Builder type(String type) { this.type = type; return this; }
        public Builder data(Map<String, Object> data) { this.data = data; return this; }
        public Builder createdAt(Instant createdAt) { this.createdAt = createdAt; return this; }
        public Builder updatedAt(Instant updatedAt) { this.updatedAt = updatedAt; return this; }
        public Builder jurisdiction(String jurisdiction) { this.jurisdiction = jurisdiction; return this; }
        public Builder metadata(Map<String, Object> metadata) { this.metadata = metadata; return this; }

        public PactEntity build() {
            return new PactEntity(id, type, data, createdAt, updatedAt, jurisdiction, metadata);
        }
    }
}
