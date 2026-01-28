package com.arka.sdk.types;

import com.fasterxml.jackson.annotation.JsonValue;

/**
 * Rule severity levels.
 */
public enum Severity {
    LOW("LOW"),
    MEDIUM("MEDIUM"),
    HIGH("HIGH"),
    CRITICAL("CRITICAL");

    private final String value;

    Severity(String value) {
        this.value = value;
    }

    @JsonValue
    public String getValue() {
        return value;
    }
}
