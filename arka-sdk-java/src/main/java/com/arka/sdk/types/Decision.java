package com.arka.sdk.types;

import com.fasterxml.jackson.annotation.JsonValue;

/**
 * Rule consequence decisions.
 */
public enum Decision {
    ALLOW("ALLOW"),
    DENY("DENY"),
    FLAG("FLAG");

    private final String value;

    Decision(String value) {
        this.value = value;
    }

    @JsonValue
    public String getValue() {
        return value;
    }
}
