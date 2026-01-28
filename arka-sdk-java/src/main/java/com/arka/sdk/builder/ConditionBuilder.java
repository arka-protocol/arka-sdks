package com.arka.sdk.builder;

import com.arka.sdk.types.Condition;
import java.util.List;

/**
 * Fluent builder for creating conditions.
 */
public class ConditionBuilder {
    private String field;
    private Condition condition;

    public ConditionBuilder field(String path) {
        this.field = path;
        return this;
    }

    public ConditionBuilder eq(Object value) {
        this.condition = new Condition.Compare(field, "eq", value);
        return this;
    }

    public ConditionBuilder ne(Object value) {
        this.condition = new Condition.Compare(field, "ne", value);
        return this;
    }

    public ConditionBuilder gt(Object value) {
        this.condition = new Condition.Compare(field, "gt", value);
        return this;
    }

    public ConditionBuilder gte(Object value) {
        this.condition = new Condition.Compare(field, "gte", value);
        return this;
    }

    public ConditionBuilder lt(Object value) {
        this.condition = new Condition.Compare(field, "lt", value);
        return this;
    }

    public ConditionBuilder lte(Object value) {
        this.condition = new Condition.Compare(field, "lte", value);
        return this;
    }

    public ConditionBuilder contains(String value) {
        this.condition = new Condition.Compare(field, "contains", value);
        return this;
    }

    public ConditionBuilder matches(String pattern) {
        this.condition = new Condition.Compare(field, "regex", pattern);
        return this;
    }

    public ConditionBuilder exists() {
        this.condition = new Condition.Exists(field);
        return this;
    }

    public ConditionBuilder in(List<Object> values) {
        this.condition = new Condition.In(field, values);
        return this;
    }

    public ConditionBuilder between(Double min, Double max) {
        this.condition = new Condition.Range(field, min, max, true, true);
        return this;
    }

    public ConditionBuilder expression(String expr, String language) {
        this.condition = new Condition.Expression(expr, language != null ? language : "cel");
        return this;
    }

    public Condition build() {
        return condition;
    }

    // Static factory methods
    public static ConditionBuilder condition() {
        return new ConditionBuilder();
    }

    public static Condition and(List<Condition> conditions) {
        return new Condition.And(conditions);
    }

    public static Condition or(List<Condition> conditions) {
        return new Condition.Or(conditions);
    }

    public static Condition not(Condition condition) {
        return new Condition.Not(condition);
    }
}
