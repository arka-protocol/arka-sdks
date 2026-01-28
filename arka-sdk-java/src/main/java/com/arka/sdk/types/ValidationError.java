package com.arka.sdk.types;

/**
 * Validation error details.
 */
public record ValidationError(
    String field,
    String message,
    String code
) {}
