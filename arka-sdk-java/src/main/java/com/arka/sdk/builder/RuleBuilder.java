package com.arka.sdk.builder;

import com.arka.sdk.types.*;
import java.time.Instant;
import java.util.*;

/**
 * Fluent builder for creating rules.
 */
public class RuleBuilder {
    private String id;
    private String name;
    private String description;
    private String jurisdiction;
    private Severity severity = Severity.MEDIUM;
    private Condition condition;
    private Consequence consequence;
    private List<String> tags = new ArrayList<>();
    private Instant effectiveFrom;
    private Instant effectiveTo;
    private Map<String, Object> metadata = new HashMap<>();

    public RuleBuilder id(String id) {
        this.id = id;
        return this;
    }

    public RuleBuilder name(String name) {
        this.name = name;
        return this;
    }

    public RuleBuilder description(String description) {
        this.description = description;
        return this;
    }

    public RuleBuilder jurisdiction(String jurisdiction) {
        this.jurisdiction = jurisdiction;
        return this;
    }

    public RuleBuilder severity(Severity severity) {
        this.severity = severity;
        return this;
    }

    public RuleBuilder when(Condition condition) {
        this.condition = condition;
        return this;
    }

    public RuleBuilder whenField(String field, String operator, Object value) {
        this.condition = new Condition.Compare(field, operator, value);
        return this;
    }

    public RuleBuilder thenDeny(String code, String message) {
        this.consequence = Consequence.deny(code, message);
        return this;
    }

    public RuleBuilder thenFlag(String code, String message) {
        this.consequence = Consequence.flag(code, message);
        return this;
    }

    public RuleBuilder thenAllow(String code, String message) {
        this.consequence = Consequence.allow(code, message);
        return this;
    }

    public RuleBuilder consequence(Consequence consequence) {
        this.consequence = consequence;
        return this;
    }

    public RuleBuilder tags(String... tags) {
        this.tags.addAll(Arrays.asList(tags));
        return this;
    }

    public RuleBuilder effectiveFrom(Instant date) {
        this.effectiveFrom = date;
        return this;
    }

    public RuleBuilder effectiveTo(Instant date) {
        this.effectiveTo = date;
        return this;
    }

    public RuleBuilder metadata(String key, Object value) {
        this.metadata.put(key, value);
        return this;
    }

    public PactRule build() {
        if (name == null) {
            throw new IllegalArgumentException("Rule name is required");
        }
        if (condition == null) {
            throw new IllegalArgumentException("Rule condition is required");
        }
        if (consequence == null) {
            throw new IllegalArgumentException("Rule consequence is required");
        }

        String ruleId = id != null ? id : "rule_" + UUID.randomUUID().toString().substring(0, 12);
        String ruleDesc = description != null ? description : name;

        return new PactRule(
            ruleId,
            name,
            ruleDesc,
            jurisdiction,
            severity,
            condition,
            consequence,
            List.copyOf(tags),
            effectiveFrom,
            effectiveTo,
            Map.copyOf(metadata)
        );
    }

    public static RuleBuilder rule() {
        return new RuleBuilder();
    }
}
