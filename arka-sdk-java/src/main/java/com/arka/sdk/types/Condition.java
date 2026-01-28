package com.arka.sdk.types;

import com.fasterxml.jackson.annotation.JsonProperty;
import com.fasterxml.jackson.annotation.JsonSubTypes;
import com.fasterxml.jackson.annotation.JsonTypeInfo;
import java.util.List;

/**
 * Base class for rule conditions.
 */
@JsonTypeInfo(use = JsonTypeInfo.Id.NAME, property = "type")
@JsonSubTypes({
    @JsonSubTypes.Type(value = Condition.Compare.class, name = "compare"),
    @JsonSubTypes.Type(value = Condition.And.class, name = "and"),
    @JsonSubTypes.Type(value = Condition.Or.class, name = "or"),
    @JsonSubTypes.Type(value = Condition.Not.class, name = "not"),
    @JsonSubTypes.Type(value = Condition.Exists.class, name = "exists"),
    @JsonSubTypes.Type(value = Condition.In.class, name = "in"),
    @JsonSubTypes.Type(value = Condition.Range.class, name = "range"),
    @JsonSubTypes.Type(value = Condition.Expression.class, name = "expression")
})
public sealed interface Condition permits
    Condition.Compare, Condition.And, Condition.Or, Condition.Not,
    Condition.Exists, Condition.In, Condition.Range, Condition.Expression {

    record Compare(String field, String operator, Object value) implements Condition {}

    record And(List<Condition> conditions) implements Condition {}

    record Or(List<Condition> conditions) implements Condition {}

    record Not(Condition condition) implements Condition {}

    record Exists(String field) implements Condition {}

    record In(String field, List<Object> values) implements Condition {}

    record Range(
        String field,
        Double min,
        Double max,
        @JsonProperty("min_inclusive") boolean minInclusive,
        @JsonProperty("max_inclusive") boolean maxInclusive
    ) implements Condition {
        public Range {
            if (!minInclusive) minInclusive = true;
            if (!maxInclusive) maxInclusive = true;
        }
    }

    record Expression(String expression, String language) implements Condition {
        public Expression {
            if (language == null) language = "cel";
        }
    }
}
