package com.arka.sdk.plugin;

import com.arka.sdk.types.*;
import java.util.List;
import java.util.Map;

/**
 * Interface that all PACT domain plugins must implement.
 */
public interface DomainPlugin {

    /**
     * Returns the plugin manifest.
     */
    PluginManifest getManifest();

    /**
     * Returns optional lifecycle hooks.
     */
    default PluginHooks getHooks() {
        return null;
    }

    /**
     * Returns entity types defined by this plugin.
     */
    List<PactEntityType> getEntityTypes();

    /**
     * Returns default rules for this domain.
     */
    List<PactRule> getDefaultRules();

    /**
     * Converts a domain event to canonical format.
     */
    PactEvent mapToCanonicalEvent(DomainEvent event);

    /**
     * Validates domain-specific data.
     */
    ValidationResult validateDomainData(String entityType, Map<String, Object> data);

    /**
     * Returns context for rule evaluation.
     */
    default Map<String, Object> getEvaluationContext(PactEvent event, PactEntity entity) {
        return Map.of();
    }

    /**
     * Serializes data for blockchain.
     */
    default byte[] serializeForChain(Object data) {
        return CanonicalJson.serialize(data);
    }

    /**
     * Deserializes data from blockchain.
     */
    default Object deserializeFromChain(byte[] data) {
        return CanonicalJson.deserialize(data);
    }
}
