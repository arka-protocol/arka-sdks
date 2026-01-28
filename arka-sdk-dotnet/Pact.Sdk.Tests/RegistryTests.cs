using FluentAssertions;
using Xunit;

namespace Pact.Sdk.Tests;

/// <summary>
/// Tests for PluginRegistry and GlobalRegistry.
/// </summary>
public class RegistryTests : IDisposable
{
    private readonly PluginRegistry _registry;

    public RegistryTests()
    {
        _registry = new PluginRegistry();
    }

    public void Dispose()
    {
        _registry.Clear();
        GlobalRegistry.Reset();
    }

    #region Test Plugin Implementations

    private class SimplePlugin : BasePlugin
    {
        public SimplePlugin(string id = "simple-plugin") : base(
            new PluginManifest
            {
                Id = id,
                Name = $"Plugin {id}",
                Version = "1.0.0",
                Author = "Test",
                Description = "Simple test plugin",
                EntityTypes = [$"{id}:Entity"],
                EventTypes = [$"{id}:created", $"{id}:updated"],
                Dependencies = [],
                PactCoreVersion = "1.0.0"
            },
            [
                new PactEntityType
                {
                    Name = $"{id}:Entity",
                    Schema = new Dictionary<string, object>(),
                    RequiredFields = []
                }
            ],
            [
                new PactRule
                {
                    Id = $"rule_{id}_001",
                    Name = $"Default Rule for {id}",
                    Condition = new CompareCondition { Field = "x", Operator = "eq", Value = 1 },
                    Consequence = Consequence.Allow("OK", "ok")
                }
            ])
        {
        }
    }

    private class DependentPlugin : BasePlugin
    {
        public DependentPlugin(string id, List<string> dependencies) : base(
            new PluginManifest
            {
                Id = id,
                Name = $"Dependent Plugin {id}",
                Version = "1.0.0",
                Author = "Test",
                Description = "Plugin with dependencies",
                EntityTypes = [],
                EventTypes = [],
                Dependencies = dependencies,
                PactCoreVersion = "1.0.0"
            },
            [],
            [])
        {
        }
    }

    private class ConflictingEntityPlugin : BasePlugin
    {
        public ConflictingEntityPlugin(string entityType) : base(
            new PluginManifest
            {
                Id = $"conflict-{Guid.NewGuid()}",
                Name = "Conflicting Plugin",
                Version = "1.0.0",
                Author = "Test",
                Description = "Plugin with conflicting entity type",
                EntityTypes = [entityType],
                EventTypes = [],
                Dependencies = [],
                PactCoreVersion = "1.0.0"
            },
            [],
            [])
        {
        }
    }

    private class ConflictingEventPlugin : BasePlugin
    {
        public ConflictingEventPlugin(string eventType) : base(
            new PluginManifest
            {
                Id = $"event-conflict-{Guid.NewGuid()}",
                Name = "Event Conflicting Plugin",
                Version = "1.0.0",
                Author = "Test",
                Description = "Plugin with conflicting event type",
                EntityTypes = [],
                EventTypes = [eventType],
                Dependencies = [],
                PactCoreVersion = "1.0.0"
            },
            [],
            [])
        {
        }
    }

    #endregion

    #region RegisterAsync Tests

    [Fact]
    public async Task RegisterAsync_AddsPluginToRegistry()
    {
        var plugin = new SimplePlugin();

        await _registry.RegisterAsync(plugin);

        _registry.IsRegistered("simple-plugin").Should().BeTrue();
    }

    [Fact]
    public async Task RegisterAsync_ThrowsForDuplicatePlugin()
    {
        var plugin1 = new SimplePlugin("duplicate-plugin");
        var plugin2 = new SimplePlugin("duplicate-plugin");

        await _registry.RegisterAsync(plugin1);

        var act = async () => await _registry.RegisterAsync(plugin2);

        await act.Should().ThrowAsync<InvalidOperationException>()
            .WithMessage("*already registered*");
    }

    [Fact]
    public async Task RegisterAsync_ThrowsForMissingDependency()
    {
        var plugin = new DependentPlugin("dependent", ["missing-plugin"]);

        var act = async () => await _registry.RegisterAsync(plugin);

        await act.Should().ThrowAsync<InvalidOperationException>()
            .WithMessage("*requires*missing-plugin*");
    }

    [Fact]
    public async Task RegisterAsync_SucceedsWithSatisfiedDependency()
    {
        var basePlugin = new SimplePlugin("base-plugin");
        var dependentPlugin = new DependentPlugin("dependent", ["base-plugin"]);

        await _registry.RegisterAsync(basePlugin);
        await _registry.RegisterAsync(dependentPlugin);

        _registry.IsRegistered("dependent").Should().BeTrue();
    }

    [Fact]
    public async Task RegisterAsync_ThrowsForConflictingEntityType()
    {
        var plugin1 = new SimplePlugin("plugin-a");
        var plugin2 = new ConflictingEntityPlugin("plugin-a:Entity");

        await _registry.RegisterAsync(plugin1);

        var act = async () => await _registry.RegisterAsync(plugin2);

        await act.Should().ThrowAsync<InvalidOperationException>()
            .WithMessage("*Entity type*already registered*");
    }

    [Fact]
    public async Task RegisterAsync_ThrowsForConflictingEventType()
    {
        var plugin1 = new SimplePlugin("plugin-evt");
        var plugin2 = new ConflictingEventPlugin("plugin-evt:created");

        await _registry.RegisterAsync(plugin1);

        var act = async () => await _registry.RegisterAsync(plugin2);

        await act.Should().ThrowAsync<InvalidOperationException>()
            .WithMessage("*Event type*already registered*");
    }

    [Fact]
    public async Task RegisterAsync_CallsOnLoadHook()
    {
        var loadCalled = false;
        var plugin = new SimplePlugin();
        plugin.Hooks = new PluginHooks
        {
            OnLoad = () =>
            {
                loadCalled = true;
                return Task.CompletedTask;
            }
        };

        await _registry.RegisterAsync(plugin);

        loadCalled.Should().BeTrue();
    }

    [Fact]
    public async Task RegisterAsync_EmitsRegistrationEvent()
    {
        var events = new List<RegistryEvent>();
        _registry.OnEvent(evt => events.Add(evt));

        var plugin = new SimplePlugin();
        await _registry.RegisterAsync(plugin);

        events.Should().Contain(e => e.Type == "plugin:registered" && e.PluginId == "simple-plugin");
    }

    [Fact]
    public async Task RegisterAsync_EmitsEntityTypeRegisteredEvents()
    {
        var events = new List<RegistryEvent>();
        _registry.OnEvent(evt => events.Add(evt));

        var plugin = new SimplePlugin();
        await _registry.RegisterAsync(plugin);

        events.Should().Contain(e =>
            e.Type == "entity_type:registered" &&
            e.Data != null &&
            e.Data.ContainsKey("entity_type"));
    }

    #endregion

    #region UnregisterAsync Tests

    [Fact]
    public async Task UnregisterAsync_RemovesPluginFromRegistry()
    {
        var plugin = new SimplePlugin();
        await _registry.RegisterAsync(plugin);

        await _registry.UnregisterAsync("simple-plugin");

        _registry.IsRegistered("simple-plugin").Should().BeFalse();
    }

    [Fact]
    public async Task UnregisterAsync_ThrowsForNonExistentPlugin()
    {
        var act = async () => await _registry.UnregisterAsync("non-existent");

        await act.Should().ThrowAsync<InvalidOperationException>()
            .WithMessage("*not registered*");
    }

    [Fact]
    public async Task UnregisterAsync_ThrowsWhenOtherPluginsDependOnIt()
    {
        var basePlugin = new SimplePlugin("base-plugin");
        var dependentPlugin = new DependentPlugin("dependent", ["base-plugin"]);

        await _registry.RegisterAsync(basePlugin);
        await _registry.RegisterAsync(dependentPlugin);

        var act = async () => await _registry.UnregisterAsync("base-plugin");

        await act.Should().ThrowAsync<InvalidOperationException>()
            .WithMessage("*Cannot unregister*depends on it*");
    }

    [Fact]
    public async Task UnregisterAsync_CallsOnUnloadHook()
    {
        var unloadCalled = false;
        var plugin = new SimplePlugin();
        plugin.Hooks = new PluginHooks
        {
            OnUnload = () =>
            {
                unloadCalled = true;
                return Task.CompletedTask;
            }
        };

        await _registry.RegisterAsync(plugin);
        await _registry.UnregisterAsync("simple-plugin");

        unloadCalled.Should().BeTrue();
    }

    [Fact]
    public async Task UnregisterAsync_RemovesEntityTypeMappings()
    {
        var plugin = new SimplePlugin();
        await _registry.RegisterAsync(plugin);

        await _registry.UnregisterAsync("simple-plugin");

        _registry.GetPluginForEntityType("simple-plugin:Entity").Should().BeNull();
    }

    [Fact]
    public async Task UnregisterAsync_RemovesEventTypeMappings()
    {
        var plugin = new SimplePlugin();
        await _registry.RegisterAsync(plugin);

        await _registry.UnregisterAsync("simple-plugin");

        _registry.GetPluginForEventType("simple-plugin:created").Should().BeNull();
    }

    [Fact]
    public async Task UnregisterAsync_EmitsUnregisteredEvent()
    {
        var events = new List<RegistryEvent>();
        _registry.OnEvent(evt => events.Add(evt));

        var plugin = new SimplePlugin();
        await _registry.RegisterAsync(plugin);
        await _registry.UnregisterAsync("simple-plugin");

        events.Should().Contain(e => e.Type == "plugin:unregistered" && e.PluginId == "simple-plugin");
    }

    #endregion

    #region GetPlugin Tests

    [Fact]
    public async Task GetPlugin_ReturnsRegisteredPlugin()
    {
        var plugin = new SimplePlugin();
        await _registry.RegisterAsync(plugin);

        var retrieved = _registry.GetPlugin("simple-plugin");

        retrieved.Should().NotBeNull();
        retrieved!.Manifest.Id.Should().Be("simple-plugin");
    }

    [Fact]
    public void GetPlugin_ReturnsNullForNonExistentPlugin()
    {
        var plugin = _registry.GetPlugin("non-existent");

        plugin.Should().BeNull();
    }

    #endregion

    #region GetPluginForEntityType Tests

    [Fact]
    public async Task GetPluginForEntityType_ReturnsCorrectPlugin()
    {
        var plugin = new SimplePlugin();
        await _registry.RegisterAsync(plugin);

        var retrieved = _registry.GetPluginForEntityType("simple-plugin:Entity");

        retrieved.Should().NotBeNull();
        retrieved!.Manifest.Id.Should().Be("simple-plugin");
    }

    [Fact]
    public void GetPluginForEntityType_ReturnsNullForUnknownType()
    {
        var plugin = _registry.GetPluginForEntityType("unknown:Entity");

        plugin.Should().BeNull();
    }

    #endregion

    #region GetPluginForEventType Tests

    [Fact]
    public async Task GetPluginForEventType_ReturnsCorrectPlugin()
    {
        var plugin = new SimplePlugin();
        await _registry.RegisterAsync(plugin);

        var retrieved = _registry.GetPluginForEventType("simple-plugin:created");

        retrieved.Should().NotBeNull();
        retrieved!.Manifest.Id.Should().Be("simple-plugin");
    }

    [Fact]
    public void GetPluginForEventType_ReturnsNullForUnknownType()
    {
        var plugin = _registry.GetPluginForEventType("unknown:event");

        plugin.Should().BeNull();
    }

    #endregion

    #region GetAllPlugins Tests

    [Fact]
    public async Task GetAllPlugins_ReturnsAllActivePlugins()
    {
        var plugin1 = new SimplePlugin("plugin-1");
        var plugin2 = new SimplePlugin("plugin-2");
        var plugin3 = new SimplePlugin("plugin-3");

        await _registry.RegisterAsync(plugin1);
        await _registry.RegisterAsync(plugin2);
        await _registry.RegisterAsync(plugin3);

        var allPlugins = _registry.GetAllPlugins();

        allPlugins.Should().HaveCount(3);
    }

    [Fact]
    public void GetAllPlugins_ReturnsEmptyWhenNoPlugins()
    {
        var allPlugins = _registry.GetAllPlugins();

        allPlugins.Should().BeEmpty();
    }

    #endregion

    #region GetAllEntityTypes Tests

    [Fact]
    public async Task GetAllEntityTypes_ReturnsAllEntityTypes()
    {
        var plugin1 = new SimplePlugin("plugin-1");
        var plugin2 = new SimplePlugin("plugin-2");

        await _registry.RegisterAsync(plugin1);
        await _registry.RegisterAsync(plugin2);

        var entityTypes = _registry.GetAllEntityTypes();

        entityTypes.Should().HaveCount(2);
        entityTypes.Should().Contain(e => e.Name == "plugin-1:Entity");
        entityTypes.Should().Contain(e => e.Name == "plugin-2:Entity");
    }

    #endregion

    #region GetAllDefaultRules Tests

    [Fact]
    public async Task GetAllDefaultRules_ReturnsAllDefaultRules()
    {
        var plugin1 = new SimplePlugin("plugin-1");
        var plugin2 = new SimplePlugin("plugin-2");

        await _registry.RegisterAsync(plugin1);
        await _registry.RegisterAsync(plugin2);

        var rules = _registry.GetAllDefaultRules();

        rules.Should().HaveCount(2);
        rules.Should().Contain(r => r.Id == "rule_plugin-1_001");
        rules.Should().Contain(r => r.Id == "rule_plugin-2_001");
    }

    #endregion

    #region GetStatus Tests

    [Fact]
    public async Task GetStatus_ReturnsActiveForRegisteredPlugin()
    {
        var plugin = new SimplePlugin();
        await _registry.RegisterAsync(plugin);

        var status = _registry.GetStatus("simple-plugin");

        status.Should().Be("active");
    }

    [Fact]
    public void GetStatus_ReturnsNullForNonExistentPlugin()
    {
        var status = _registry.GetStatus("non-existent");

        status.Should().BeNull();
    }

    #endregion

    #region GetStats Tests

    [Fact]
    public async Task GetStats_ReturnsCorrectCounts()
    {
        var plugin1 = new SimplePlugin("plugin-1");
        var plugin2 = new SimplePlugin("plugin-2");

        await _registry.RegisterAsync(plugin1);
        await _registry.RegisterAsync(plugin2);

        var stats = _registry.GetStats();

        stats["plugin_count"].Should().Be(2);
        stats["entity_type_count"].Should().Be(2);
        stats["event_type_count"].Should().Be(4); // 2 events per plugin
        ((List<string>)stats["active_plugins"]).Should().HaveCount(2);
    }

    [Fact]
    public void GetStats_ReturnsZerosWhenEmpty()
    {
        var stats = _registry.GetStats();

        stats["plugin_count"].Should().Be(0);
        stats["entity_type_count"].Should().Be(0);
        stats["event_type_count"].Should().Be(0);
    }

    #endregion

    #region Clear Tests

    [Fact]
    public async Task Clear_RemovesAllPlugins()
    {
        await _registry.RegisterAsync(new SimplePlugin("plugin-1"));
        await _registry.RegisterAsync(new SimplePlugin("plugin-2"));

        _registry.Clear();

        _registry.GetAllPlugins().Should().BeEmpty();
        _registry.GetStats()["plugin_count"].Should().Be(0);
    }

    #endregion

    #region OnEvent Tests

    [Fact]
    public void OnEvent_AddsEventHandler()
    {
        var eventsCaptured = new List<RegistryEvent>();

        _registry.OnEvent(evt => eventsCaptured.Add(evt));

        // Trigger an event
        _registry.RegisterAsync(new SimplePlugin()).Wait();

        eventsCaptured.Should().NotBeEmpty();
    }

    [Fact]
    public async Task OnEvent_MultipleHandlersReceiveEvents()
    {
        var handler1Events = new List<RegistryEvent>();
        var handler2Events = new List<RegistryEvent>();

        _registry.OnEvent(evt => handler1Events.Add(evt));
        _registry.OnEvent(evt => handler2Events.Add(evt));

        await _registry.RegisterAsync(new SimplePlugin());

        handler1Events.Should().NotBeEmpty();
        handler2Events.Should().NotBeEmpty();
        handler1Events.Count.Should().Be(handler2Events.Count);
    }

    [Fact]
    public async Task OnEvent_HandlerExceptionsDoNotPropagate()
    {
        _registry.OnEvent(evt => throw new Exception("Handler error"));

        // Should not throw
        var act = async () => await _registry.RegisterAsync(new SimplePlugin());

        await act.Should().NotThrowAsync();
    }

    #endregion

    #region GlobalRegistry Tests

    [Fact]
    public void GlobalRegistry_Instance_ReturnsSameInstance()
    {
        var instance1 = GlobalRegistry.Instance;
        var instance2 = GlobalRegistry.Instance;

        instance1.Should().BeSameAs(instance2);
    }

    [Fact]
    public async Task GlobalRegistry_Reset_ClearsAllPlugins()
    {
        await GlobalRegistry.Instance.RegisterAsync(new SimplePlugin("global-plugin"));

        GlobalRegistry.Reset();

        GlobalRegistry.Instance.IsRegistered("global-plugin").Should().BeFalse();
    }

    [Fact]
    public void GlobalRegistry_Instance_IsPluginRegistry()
    {
        GlobalRegistry.Instance.Should().BeOfType<PluginRegistry>();
    }

    #endregion

    #region RegistryEvent Tests

    [Fact]
    public void RegistryEvent_CanBeCreatedWithAllProperties()
    {
        var evt = new RegistryEvent(
            "test:event",
            "test-plugin",
            new Dictionary<string, object> { ["key"] = "value" }
        );

        evt.Type.Should().Be("test:event");
        evt.PluginId.Should().Be("test-plugin");
        evt.Data.Should().ContainKey("key");
    }

    [Fact]
    public void RegistryEvent_CanBeCreatedWithNullData()
    {
        var evt = new RegistryEvent("test:event", "test-plugin");

        evt.Data.Should().BeNull();
    }

    #endregion

    #region PluginRegistration Tests

    [Fact]
    public void PluginRegistration_StoresAllProperties()
    {
        var plugin = new SimplePlugin();
        var registeredAt = DateTimeOffset.UtcNow;

        var registration = new PluginRegistration(
            plugin,
            registeredAt,
            "active",
            null
        );

        registration.Plugin.Should().Be(plugin);
        registration.RegisteredAt.Should().Be(registeredAt);
        registration.Status.Should().Be("active");
        registration.Error.Should().BeNull();
    }

    [Fact]
    public void PluginRegistration_DefaultStatusIsActive()
    {
        var plugin = new SimplePlugin();
        var registration = new PluginRegistration(plugin, DateTimeOffset.UtcNow);

        registration.Status.Should().Be("active");
    }

    #endregion
}
