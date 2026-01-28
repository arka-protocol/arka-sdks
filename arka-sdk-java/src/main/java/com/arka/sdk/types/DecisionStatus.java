package com.arka.sdk.types;

import com.fasterxml.jackson.annotation.JsonValue;

/**
 * Overall decision status.
 */
public enum DecisionStatus {
    ALLOW("ALLOW"),
    ALLOW_WITH_FLAGS("ALLOW_WITH_FLAGS"),
    DENY("DENY");

    private final String value;

    DecisionStatus(String value) {
        this.value = value;
    }

    @JsonValue
    public String getValue() {
        return value;
    }
}
