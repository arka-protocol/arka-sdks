package com.arka.sdk.plugin;

import com.arka.sdk.types.*;
import java.time.Instant;
import java.util.*;

/**
 * Base implementation of DomainPlugin.
 */
public abstract class BasePlugin implements DomainPlugin {
    private final PluginManifest manifest;
    private final List<PactEntityType> entityTypes;
    private final List<PactRule> defaultRules;
    private PluginHooks hooks;

    protected BasePlugin(
        PluginManifest manifest,
        List<PactEntityType> entityTypes,
        List<PactRule> defaultRules
    ) {
        this.manifest = manifest;
        this.entityTypes = entityTypes;
        this.defaultRules = defaultRules;
    }

    public void setHooks(PluginHooks hooks) {
        this.hooks = hooks;
    }

    @Override
    public PluginManifest getManifest() {
        return manifest;
    }

    @Override
    public PluginHooks getHooks() {
        return hooks;
    }

    @Override
    public List<PactEntityType> getEntityTypes() {
        return entityTypes;
    }

    @Override
    public List<PactRule> getDefaultRules() {
        return defaultRules;
    }

    @Override
    public PactEvent mapToCanonicalEvent(DomainEvent event) {
        Instant now = Instant.now();
        Instant occurredAt = event.occurredAt() != null ? event.occurredAt() : now;

        return PactEvent.builder()
            .id("evt_" + UUID.randomUUID().toString().substring(0, 12))
            .source(manifest.id())
            .type(event.type())
            .entityId(event.entityId())
            .entityType(inferEntityType(event))
            .jurisdiction(event.jurisdiction())
            .payload(event.payload())
            .occurredAt(occurredAt)
            .receivedAt(now)
            .metadata(event.metadata())
            .build();
    }

    protected String inferEntityType(DomainEvent event) {
        String[] parts = event.type().split("_");
        if (parts.length >= 2) {
            String entityName = parts[0].toLowerCase();
            return entityName.substring(0, 1).toUpperCase() + entityName.substring(1);
        }
        return null;
    }

    @Override
    public ValidationResult validateDomainData(String entityType, Map<String, Object> data) {
        PactEntityType entityTypeDef = entityTypes.stream()
            .filter(et -> et.name().equals(entityType))
            .findFirst()
            .orElse(null);

        if (entityTypeDef == null) {
            return ValidationResult.failure("entity_type",
                "Unknown entity type: " + entityType, "UNKNOWN_ENTITY_TYPE");
        }

        List<ValidationError> errors = new ArrayList<>();
        for (String field : entityTypeDef.requiredFields()) {
            if (!data.containsKey(field) || data.get(field) == null) {
                errors.add(new ValidationError(field,
                    "Required field '" + field + "' is missing", "REQUIRED_FIELD_MISSING"));
            }
        }

        if (errors.isEmpty()) {
            return ValidationResult.success();
        }
        return ValidationResult.failure(errors);
    }

    public String createRuleId() {
        return "rule_" + UUID.randomUUID().toString().substring(0, 12);
    }

    public PactRule createRule(String name, Condition condition, Consequence consequence) {
        return PactRule.builder()
            .id(createRuleId())
            .name(name)
            .description(name)
            .severity(Severity.MEDIUM)
            .condition(condition)
            .consequence(consequence)
            .tags(List.of(manifest.id()))
            .metadata(Map.of(
                "plugin_id", manifest.id(),
                "plugin_version", manifest.version()
            ))
            .build();
    }
}
