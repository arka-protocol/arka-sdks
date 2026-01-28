using System.Collections.Concurrent;

namespace Pact.Sdk;

/// <summary>
/// Plugin registration entry.
/// </summary>
public record PluginRegistration(
    IDomainPlugin Plugin,
    DateTimeOffset RegisteredAt,
    string Status = "active",
    string? Error = null
);

/// <summary>
/// Registry event.
/// </summary>
public record RegistryEvent(string Type, string PluginId, Dictionary<string, object>? Data = null);

/// <summary>
/// Central registry for PACT domain plugins.
/// </summary>
public class PluginRegistry
{
    private readonly ConcurrentDictionary<string, PluginRegistration> _plugins = new();
    private readonly ConcurrentDictionary<string, string> _entityTypeToPlugin = new();
    private readonly ConcurrentDictionary<string, string> _eventTypeToPlugin = new();
    private readonly List<Action<RegistryEvent>> _eventHandlers = [];

    /// <summary>
    /// Registers a domain plugin.
    /// </summary>
    public async Task RegisterAsync(IDomainPlugin plugin)
    {
        var manifest = plugin.Manifest;

        // Check for duplicate registration
        if (_plugins.ContainsKey(manifest.Id))
            throw new InvalidOperationException($"Plugin '{manifest.Id}' is already registered");

        // Check dependencies
        foreach (var dep in manifest.Dependencies)
        {
            if (!_plugins.ContainsKey(dep))
                throw new InvalidOperationException($"Plugin '{manifest.Id}' requires '{dep}' which is not registered");
        }

        // Check for entity type conflicts
        foreach (var entityType in manifest.EntityTypes)
        {
            if (_entityTypeToPlugin.TryGetValue(entityType, out var existing))
                throw new InvalidOperationException($"Entity type '{entityType}' is already registered by plugin '{existing}'");
        }

        // Check for event type conflicts
        foreach (var eventType in manifest.EventTypes)
        {
            if (_eventTypeToPlugin.TryGetValue(eventType, out var existing))
                throw new InvalidOperationException($"Event type '{eventType}' is already registered by plugin '{existing}'");
        }

        // Call on_load hook
        if (plugin.Hooks?.OnLoad != null)
            await plugin.Hooks.OnLoad();

        // Register plugin
        var registration = new PluginRegistration(plugin, DateTimeOffset.UtcNow);
        _plugins[manifest.Id] = registration;

        // Map entity types
        foreach (var entityType in manifest.EntityTypes)
        {
            _entityTypeToPlugin[entityType] = manifest.Id;
            Emit(new RegistryEvent("entity_type:registered", manifest.Id,
                new() { ["entity_type"] = entityType }));
        }

        // Map event types
        foreach (var eventType in manifest.EventTypes)
            _eventTypeToPlugin[eventType] = manifest.Id;

        Emit(new RegistryEvent("plugin:registered", manifest.Id));
    }

    /// <summary>
    /// Unregisters a plugin.
    /// </summary>
    public async Task UnregisterAsync(string pluginId)
    {
        if (!_plugins.TryGetValue(pluginId, out var registration))
            throw new InvalidOperationException($"Plugin '{pluginId}' is not registered");

        // Check dependents
        foreach (var (otherId, otherReg) in _plugins)
        {
            if (otherId != pluginId && otherReg.Plugin.Manifest.Dependencies.Contains(pluginId))
                throw new InvalidOperationException($"Cannot unregister '{pluginId}': plugin '{otherId}' depends on it");
        }

        // Call on_unload hook
        if (registration.Plugin.Hooks?.OnUnload != null)
            await registration.Plugin.Hooks.OnUnload();

        // Remove mappings
        foreach (var entityType in registration.Plugin.Manifest.EntityTypes)
            _entityTypeToPlugin.TryRemove(entityType, out _);

        foreach (var eventType in registration.Plugin.Manifest.EventTypes)
            _eventTypeToPlugin.TryRemove(eventType, out _);

        _plugins.TryRemove(pluginId, out _);

        Emit(new RegistryEvent("plugin:unregistered", pluginId));
    }

    /// <summary>
    /// Gets a plugin by ID.
    /// </summary>
    public IDomainPlugin? GetPlugin(string pluginId) =>
        _plugins.TryGetValue(pluginId, out var reg) ? reg.Plugin : null;

    /// <summary>
    /// Gets the plugin responsible for an entity type.
    /// </summary>
    public IDomainPlugin? GetPluginForEntityType(string entityType) =>
        _entityTypeToPlugin.TryGetValue(entityType, out var pluginId) ? GetPlugin(pluginId) : null;

    /// <summary>
    /// Gets the plugin responsible for an event type.
    /// </summary>
    public IDomainPlugin? GetPluginForEventType(string eventType) =>
        _eventTypeToPlugin.TryGetValue(eventType, out var pluginId) ? GetPlugin(pluginId) : null;

    /// <summary>
    /// Gets all registered plugins.
    /// </summary>
    public IReadOnlyList<IDomainPlugin> GetAllPlugins() =>
        _plugins.Values.Where(r => r.Status == "active").Select(r => r.Plugin).ToList();

    /// <summary>
    /// Gets all entity types from all plugins.
    /// </summary>
    public IReadOnlyList<PactEntityType> GetAllEntityTypes() =>
        _plugins.Values
            .Where(r => r.Status == "active")
            .SelectMany(r => r.Plugin.GetEntityTypes())
            .ToList();

    /// <summary>
    /// Gets all default rules from all plugins.
    /// </summary>
    public IReadOnlyList<PactRule> GetAllDefaultRules() =>
        _plugins.Values
            .Where(r => r.Status == "active")
            .SelectMany(r => r.Plugin.GetDefaultRules())
            .ToList();

    /// <summary>
    /// Checks if a plugin is registered.
    /// </summary>
    public bool IsRegistered(string pluginId) => _plugins.ContainsKey(pluginId);

    /// <summary>
    /// Gets plugin status.
    /// </summary>
    public string? GetStatus(string pluginId) =>
        _plugins.TryGetValue(pluginId, out var reg) ? reg.Status : null;

    /// <summary>
    /// Subscribe to registry events.
    /// </summary>
    public void OnEvent(Action<RegistryEvent> handler)
    {
        lock (_eventHandlers)
            _eventHandlers.Add(handler);
    }

    private void Emit(RegistryEvent evt)
    {
        lock (_eventHandlers)
        {
            foreach (var handler in _eventHandlers)
            {
                try { handler(evt); }
                catch { /* ignore */ }
            }
        }
    }

    /// <summary>
    /// Gets registry statistics.
    /// </summary>
    public Dictionary<string, object> GetStats() => new()
    {
        ["plugin_count"] = _plugins.Count,
        ["entity_type_count"] = _entityTypeToPlugin.Count,
        ["event_type_count"] = _eventTypeToPlugin.Count,
        ["active_plugins"] = _plugins.Where(p => p.Value.Status == "active").Select(p => p.Key).ToList()
    };

    /// <summary>
    /// Clears all plugins.
    /// </summary>
    public void Clear()
    {
        _plugins.Clear();
        _entityTypeToPlugin.Clear();
        _eventTypeToPlugin.Clear();
    }
}

/// <summary>
/// Global registry singleton.
/// </summary>
public static class GlobalRegistry
{
    private static readonly Lazy<PluginRegistry> _instance = new(() => new PluginRegistry());

    public static PluginRegistry Instance => _instance.Value;

    public static void Reset() => Instance.Clear();
}
