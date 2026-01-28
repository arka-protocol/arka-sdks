package com.arka.sdk.types;

import com.fasterxml.jackson.annotation.JsonProperty;
import java.time.Instant;
import java.util.List;
import java.util.Map;

/**
 * Decision result from PACT Core rule evaluation.
 */
public record PactDecision(
    String id,
    @JsonProperty("event_id") String eventId,
    DecisionStatus status,
    @JsonProperty("triggered_rules") List<TriggeredRule> triggeredRules,
    @JsonProperty("decided_at") Instant decidedAt,
    @JsonProperty("execution_time_ms") Long executionTimeMs,
    Map<String, Object> metadata
) {
    public PactDecision {
        if (triggeredRules == null) triggeredRules = List.of();
        if (metadata == null) metadata = Map.of();
    }

    public static Builder builder() {
        return new Builder();
    }

    public boolean isAllowed() {
        return status == DecisionStatus.ALLOW || status == DecisionStatus.ALLOW_WITH_FLAGS;
    }

    public boolean isDenied() {
        return status == DecisionStatus.DENY;
    }

    public boolean hasFlagged() {
        return status == DecisionStatus.ALLOW_WITH_FLAGS ||
               triggeredRules.stream().anyMatch(r -> r.decision() == Decision.FLAG);
    }

    /**
     * A rule that was triggered during evaluation.
     */
    public record TriggeredRule(
        @JsonProperty("rule_id") String ruleId,
        @JsonProperty("rule_name") String ruleName,
        Decision decision,
        String code,
        String message,
        Severity severity
    ) {}

    public static class Builder {
        private String id;
        private String eventId;
        private DecisionStatus status;
        private List<TriggeredRule> triggeredRules = List.of();
        private Instant decidedAt;
        private Long executionTimeMs;
        private Map<String, Object> metadata = Map.of();

        public Builder id(String id) { this.id = id; return this; }
        public Builder eventId(String eventId) { this.eventId = eventId; return this; }
        public Builder status(DecisionStatus status) { this.status = status; return this; }
        public Builder triggeredRules(List<TriggeredRule> triggeredRules) { this.triggeredRules = triggeredRules; return this; }
        public Builder decidedAt(Instant decidedAt) { this.decidedAt = decidedAt; return this; }
        public Builder executionTimeMs(Long executionTimeMs) { this.executionTimeMs = executionTimeMs; return this; }
        public Builder metadata(Map<String, Object> metadata) { this.metadata = metadata; return this; }

        public PactDecision build() {
            return new PactDecision(id, eventId, status, triggeredRules, decidedAt, executionTimeMs, metadata);
        }
    }
}
