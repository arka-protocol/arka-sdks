package com.arka.sdk.types;

import com.fasterxml.jackson.annotation.JsonProperty;
import java.time.Instant;
import java.util.List;
import java.util.Map;

/**
 * PACT rule definition.
 */
public record PactRule(
    String id,
    String name,
    String description,
    String jurisdiction,
    Severity severity,
    Condition condition,
    Consequence consequence,
    List<String> tags,
    @JsonProperty("effective_from") Instant effectiveFrom,
    @JsonProperty("effective_to") Instant effectiveTo,
    Map<String, Object> metadata
) {
    public PactRule {
        if (severity == null) severity = Severity.MEDIUM;
        if (tags == null) tags = List.of();
        if (metadata == null) metadata = Map.of();
    }

    public static Builder builder() {
        return new Builder();
    }

    public static class Builder {
        private String id;
        private String name;
        private String description;
        private String jurisdiction;
        private Severity severity = Severity.MEDIUM;
        private Condition condition;
        private Consequence consequence;
        private List<String> tags = List.of();
        private Instant effectiveFrom;
        private Instant effectiveTo;
        private Map<String, Object> metadata = Map.of();

        public Builder id(String id) { this.id = id; return this; }
        public Builder name(String name) { this.name = name; return this; }
        public Builder description(String description) { this.description = description; return this; }
        public Builder jurisdiction(String jurisdiction) { this.jurisdiction = jurisdiction; return this; }
        public Builder severity(Severity severity) { this.severity = severity; return this; }
        public Builder condition(Condition condition) { this.condition = condition; return this; }
        public Builder consequence(Consequence consequence) { this.consequence = consequence; return this; }
        public Builder tags(List<String> tags) { this.tags = tags; return this; }
        public Builder effectiveFrom(Instant effectiveFrom) { this.effectiveFrom = effectiveFrom; return this; }
        public Builder effectiveTo(Instant effectiveTo) { this.effectiveTo = effectiveTo; return this; }
        public Builder metadata(Map<String, Object> metadata) { this.metadata = metadata; return this; }

        public PactRule build() {
            if (description == null) description = name;
            return new PactRule(id, name, description, jurisdiction, severity, condition, consequence, tags, effectiveFrom, effectiveTo, metadata);
        }
    }
}
