package com.arka.sdk.types;

import com.fasterxml.jackson.annotation.JsonProperty;
import java.util.List;
import java.util.Map;

/**
 * Entity type definition with schema.
 */
public record PactEntityType(
    String name,
    String description,
    Map<String, Object> schema,
    @JsonProperty("required_fields") List<String> requiredFields
) {
    public PactEntityType {
        if (description == null) description = "";
        if (requiredFields == null) requiredFields = List.of();
    }

    public static Builder builder() {
        return new Builder();
    }

    public static class Builder {
        private String name;
        private String description = "";
        private Map<String, Object> schema;
        private List<String> requiredFields = List.of();

        public Builder name(String name) { this.name = name; return this; }
        public Builder description(String description) { this.description = description; return this; }
        public Builder schema(Map<String, Object> schema) { this.schema = schema; return this; }
        public Builder requiredFields(List<String> requiredFields) { this.requiredFields = requiredFields; return this; }

        public PactEntityType build() {
            return new PactEntityType(name, description, schema, requiredFields);
        }
    }
}
