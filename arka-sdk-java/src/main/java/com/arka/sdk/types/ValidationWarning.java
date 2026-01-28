package com.arka.sdk.types;

/**
 * Validation warning details.
 */
public record ValidationWarning(
    String field,
    String message
) {}
