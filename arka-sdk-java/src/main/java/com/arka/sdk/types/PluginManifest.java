package com.arka.sdk.types;

import com.fasterxml.jackson.annotation.JsonProperty;
import java.util.List;
import java.util.Map;

/**
 * Plugin manifest describing capabilities.
 */
public record PluginManifest(
    String id,
    String name,
    String version,
    String author,
    String description,
    @JsonProperty("entity_types") List<String> entityTypes,
    @JsonProperty("event_types") List<String> eventTypes,
    List<String> dependencies,
    @JsonProperty("pact_core_version") String pactCoreVersion,
    @JsonProperty("config_schema") Map<String, Object> configSchema
) {
    public PluginManifest {
        if (entityTypes == null) entityTypes = List.of();
        if (eventTypes == null) eventTypes = List.of();
        if (dependencies == null) dependencies = List.of();
    }

    public static Builder builder() {
        return new Builder();
    }

    public static class Builder {
        private String id;
        private String name;
        private String version;
        private String author;
        private String description;
        private List<String> entityTypes = List.of();
        private List<String> eventTypes = List.of();
        private List<String> dependencies = List.of();
        private String pactCoreVersion;
        private Map<String, Object> configSchema;

        public Builder id(String id) { this.id = id; return this; }
        public Builder name(String name) { this.name = name; return this; }
        public Builder version(String version) { this.version = version; return this; }
        public Builder author(String author) { this.author = author; return this; }
        public Builder description(String description) { this.description = description; return this; }
        public Builder entityTypes(List<String> entityTypes) { this.entityTypes = entityTypes; return this; }
        public Builder eventTypes(List<String> eventTypes) { this.eventTypes = eventTypes; return this; }
        public Builder dependencies(List<String> dependencies) { this.dependencies = dependencies; return this; }
        public Builder pactCoreVersion(String pactCoreVersion) { this.pactCoreVersion = pactCoreVersion; return this; }
        public Builder configSchema(Map<String, Object> configSchema) { this.configSchema = configSchema; return this; }

        public PluginManifest build() {
            return new PluginManifest(id, name, version, author, description, entityTypes, eventTypes, dependencies, pactCoreVersion, configSchema);
        }
    }
}
