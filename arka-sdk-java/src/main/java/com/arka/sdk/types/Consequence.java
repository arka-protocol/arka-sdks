package com.arka.sdk.types;

import java.util.Map;

/**
 * Rule consequence definition.
 */
public record Consequence(
    Decision decision,
    String code,
    String message,
    Map<String, Object> metadata
) {
    public Consequence {
        if (metadata == null) metadata = Map.of();
    }

    public static Consequence deny(String code, String message) {
        return new Consequence(Decision.DENY, code, message, Map.of());
    }

    public static Consequence flag(String code, String message) {
        return new Consequence(Decision.FLAG, code, message, Map.of());
    }

    public static Consequence allow(String code, String message) {
        return new Consequence(Decision.ALLOW, code, message, Map.of());
    }
}
