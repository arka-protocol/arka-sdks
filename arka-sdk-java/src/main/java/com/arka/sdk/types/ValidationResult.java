package com.arka.sdk.types;

import java.util.List;

/**
 * Result of data validation.
 */
public record ValidationResult(
    boolean valid,
    List<ValidationError> errors,
    List<ValidationWarning> warnings
) {
    public ValidationResult {
        if (errors == null) errors = List.of();
        if (warnings == null) warnings = List.of();
    }

    public static ValidationResult success() {
        return new ValidationResult(true, List.of(), List.of());
    }

    public static ValidationResult failure(List<ValidationError> errors) {
        return new ValidationResult(false, errors, List.of());
    }

    public static ValidationResult failure(String field, String message, String code) {
        return failure(List.of(new ValidationError(field, message, code)));
    }
}
