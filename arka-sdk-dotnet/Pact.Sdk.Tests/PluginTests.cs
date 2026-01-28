using FluentAssertions;
using Xunit;

namespace Pact.Sdk.Tests;

/// <summary>
/// Tests for plugin interface and base plugin implementation.
/// </summary>
public class PluginTests
{
    #region Test Plugin Implementation

    private class TestPlugin : BasePlugin
    {
        public TestPlugin() : base(
            new PluginManifest
            {
                Id = "test-plugin",
                Name = "Test Plugin",
                Version = "1.0.0",
                Author = "Test Author",
                Description = "A test plugin",
                EntityTypes = ["TestEntity", "AnotherEntity"],
                EventTypes = ["test_created", "test_updated", "test_deleted"],
                Dependencies = [],
                PactCoreVersion = "1.0.0"
            },
            [
                new PactEntityType
                {
                    Name = "TestEntity",
                    Description = "A test entity",
                    Schema = new Dictionary<string, object>
                    {
                        ["name"] = new { type = "string" },
                        ["value"] = new { type = "number" }
                    },
                    RequiredFields = ["name"]
                },
                new PactEntityType
                {
                    Name = "AnotherEntity",
                    Description = "Another test entity",
                    Schema = new Dictionary<string, object>
                    {
                        ["code"] = new { type = "string" }
                    },
                    RequiredFields = ["code"]
                }
            ],
            [
                new PactRule
                {
                    Id = "test-rule-001",
                    Name = "Test Default Rule",
                    Description = "A default rule for testing",
                    Severity = Severity.Medium,
                    Condition = new CompareCondition
                    {
                        Field = "payload.value",
                        Operator = "gt",
                        Value = 100
                    },
                    Consequence = Consequence.Flag("TEST_FLAG", "Value exceeds threshold"),
                    Tags = ["test"]
                }
            ])
        {
        }
    }

    private class PluginWithDependency : BasePlugin
    {
        public PluginWithDependency() : base(
            new PluginManifest
            {
                Id = "dependent-plugin",
                Name = "Dependent Plugin",
                Version = "1.0.0",
                Author = "Test",
                Description = "Plugin with dependency",
                Dependencies = ["test-plugin"],
                PactCoreVersion = "1.0.0"
            },
            [],
            [])
        {
        }
    }

    #endregion

    #region IDomainPlugin Interface Tests

    [Fact]
    public void Plugin_ReturnsManifest()
    {
        var plugin = new TestPlugin();

        plugin.Manifest.Should().NotBeNull();
        plugin.Manifest.Id.Should().Be("test-plugin");
        plugin.Manifest.Name.Should().Be("Test Plugin");
        plugin.Manifest.Version.Should().Be("1.0.0");
    }

    [Fact]
    public void Plugin_ReturnsEntityTypes()
    {
        var plugin = new TestPlugin();

        var entityTypes = plugin.GetEntityTypes();

        entityTypes.Should().HaveCount(2);
        entityTypes.Should().Contain(e => e.Name == "TestEntity");
        entityTypes.Should().Contain(e => e.Name == "AnotherEntity");
    }

    [Fact]
    public void Plugin_ReturnsDefaultRules()
    {
        var plugin = new TestPlugin();

        var rules = plugin.GetDefaultRules();

        rules.Should().HaveCount(1);
        rules[0].Id.Should().Be("test-rule-001");
        rules[0].Name.Should().Be("Test Default Rule");
    }

    #endregion

    #region BasePlugin.MapToCanonicalEvent Tests

    [Fact]
    public void Plugin_MapToCanonicalEvent_CreatesValidPactEvent()
    {
        var plugin = new TestPlugin();
        var domainEvent = new DomainEvent
        {
            Type = "test_created",
            EntityId = "entity-123",
            Jurisdiction = "US",
            Payload = new Dictionary<string, object>
            {
                ["name"] = "Test",
                ["value"] = 50
            }
        };

        var pactEvent = plugin.MapToCanonicalEvent(domainEvent);

        pactEvent.Should().NotBeNull();
        pactEvent.Id.Should().StartWith("evt_");
        pactEvent.Source.Should().Be("test-plugin");
        pactEvent.Type.Should().Be("test_created");
        pactEvent.EntityId.Should().Be("entity-123");
        pactEvent.Jurisdiction.Should().Be("US");
        pactEvent.Payload.Should().ContainKey("name");
    }

    [Fact]
    public void Plugin_MapToCanonicalEvent_InfersEntityType()
    {
        var plugin = new TestPlugin();
        var domainEvent = new DomainEvent
        {
            Type = "user_created",
            Payload = new Dictionary<string, object>()
        };

        var pactEvent = plugin.MapToCanonicalEvent(domainEvent);

        pactEvent.EntityType.Should().Be("User");
    }

    [Fact]
    public void Plugin_MapToCanonicalEvent_UsesProvidedOccurredAt()
    {
        var plugin = new TestPlugin();
        var occurredAt = DateTimeOffset.Parse("2024-01-15T10:30:00Z");
        var domainEvent = new DomainEvent
        {
            Type = "test_created",
            OccurredAt = occurredAt
        };

        var pactEvent = plugin.MapToCanonicalEvent(domainEvent);

        pactEvent.OccurredAt.Should().Be(occurredAt);
    }

    [Fact]
    public void Plugin_MapToCanonicalEvent_GeneratesOccurredAt_WhenNull()
    {
        var plugin = new TestPlugin();
        var domainEvent = new DomainEvent
        {
            Type = "test_created"
        };

        var beforeCall = DateTimeOffset.UtcNow;
        var pactEvent = plugin.MapToCanonicalEvent(domainEvent);
        var afterCall = DateTimeOffset.UtcNow;

        pactEvent.OccurredAt.Should().BeOnOrAfter(beforeCall);
        pactEvent.OccurredAt.Should().BeOnOrBefore(afterCall);
    }

    [Fact]
    public void Plugin_MapToCanonicalEvent_SetsReceivedAt()
    {
        var plugin = new TestPlugin();
        var domainEvent = new DomainEvent
        {
            Type = "test_created"
        };

        var beforeCall = DateTimeOffset.UtcNow;
        var pactEvent = plugin.MapToCanonicalEvent(domainEvent);
        var afterCall = DateTimeOffset.UtcNow;

        pactEvent.ReceivedAt.Should().BeOnOrAfter(beforeCall);
        pactEvent.ReceivedAt.Should().BeOnOrBefore(afterCall);
    }

    [Fact]
    public void Plugin_MapToCanonicalEvent_PreservesMetadata()
    {
        var plugin = new TestPlugin();
        var domainEvent = new DomainEvent
        {
            Type = "test_created",
            Metadata = new Dictionary<string, object>
            {
                ["trace_id"] = "trace-123",
                ["source_system"] = "external"
            }
        };

        var pactEvent = plugin.MapToCanonicalEvent(domainEvent);

        pactEvent.Metadata.Should().ContainKey("trace_id");
        pactEvent.Metadata["trace_id"].Should().Be("trace-123");
    }

    #endregion

    #region BasePlugin.ValidateDomainData Tests

    [Fact]
    public void Plugin_ValidateDomainData_ReturnsSuccess_WhenValid()
    {
        var plugin = new TestPlugin();
        var data = new Dictionary<string, object>
        {
            ["name"] = "Test Entity",
            ["value"] = 100
        };

        var result = plugin.ValidateDomainData("TestEntity", data);

        result.Valid.Should().BeTrue();
        result.Errors.Should().BeEmpty();
    }

    [Fact]
    public void Plugin_ValidateDomainData_ReturnsFailure_WhenRequiredFieldMissing()
    {
        var plugin = new TestPlugin();
        var data = new Dictionary<string, object>
        {
            ["value"] = 100
        };

        var result = plugin.ValidateDomainData("TestEntity", data);

        result.Valid.Should().BeFalse();
        result.Errors.Should().HaveCount(1);
        result.Errors[0].Field.Should().Be("name");
        result.Errors[0].Code.Should().Be("REQUIRED_FIELD_MISSING");
    }

    [Fact]
    public void Plugin_ValidateDomainData_ReturnsFailure_WhenRequiredFieldNull()
    {
        var plugin = new TestPlugin();
        var data = new Dictionary<string, object>
        {
            ["name"] = null!,
            ["value"] = 100
        };

        var result = plugin.ValidateDomainData("TestEntity", data);

        result.Valid.Should().BeFalse();
        result.Errors.Should().Contain(e => e.Field == "name");
    }

    [Fact]
    public void Plugin_ValidateDomainData_ReturnsFailure_ForUnknownEntityType()
    {
        var plugin = new TestPlugin();
        var data = new Dictionary<string, object> { ["field"] = "value" };

        var result = plugin.ValidateDomainData("UnknownEntity", data);

        result.Valid.Should().BeFalse();
        result.Errors.Should().HaveCount(1);
        result.Errors[0].Field.Should().Be("entity_type");
        result.Errors[0].Code.Should().Be("UNKNOWN_ENTITY_TYPE");
    }

    [Fact]
    public void Plugin_ValidateDomainData_ValidatesAllRequiredFields()
    {
        var plugin = new TestPlugin();
        // AnotherEntity requires "code"
        var data = new Dictionary<string, object>();

        var result = plugin.ValidateDomainData("AnotherEntity", data);

        result.Valid.Should().BeFalse();
        result.Errors.Should().Contain(e => e.Field == "code");
    }

    #endregion

    #region BasePlugin.CreateRule Tests

    [Fact]
    public void Plugin_CreateRule_GeneratesRuleId()
    {
        var plugin = new TestPlugin();

        var ruleId = plugin.CreateRuleId();

        ruleId.Should().StartWith("rule_");
        ruleId.Should().HaveLength(17); // "rule_" + 12 chars
    }

    [Fact]
    public void Plugin_CreateRule_CreatesValidRule()
    {
        var plugin = new TestPlugin();
        var condition = new CompareCondition
        {
            Field = "amount",
            Operator = "gt",
            Value = 1000
        };
        var consequence = Consequence.Deny("HIGH_AMOUNT", "Amount too high");

        var rule = plugin.CreateRule("High Amount Rule", condition, consequence);

        rule.Should().NotBeNull();
        rule.Id.Should().StartWith("rule_");
        rule.Name.Should().Be("High Amount Rule");
        rule.Description.Should().Be("High Amount Rule");
        rule.Condition.Should().Be(condition);
        rule.Consequence.Should().Be(consequence);
        rule.Severity.Should().Be(Severity.Medium);
        rule.Tags.Should().Contain("test-plugin");
        rule.Metadata.Should().ContainKey("plugin_id");
        rule.Metadata.Should().ContainKey("plugin_version");
    }

    #endregion

    #region BasePlugin.Serialization Tests

    [Fact]
    public void Plugin_SerializeForChain_ReturnsBytes()
    {
        var plugin = new TestPlugin();
        var data = new { Name = "Test", Value = 123 };

        var bytes = plugin.SerializeForChain(data);

        bytes.Should().NotBeNull();
        bytes.Should().NotBeEmpty();
    }

    [Fact]
    public void Plugin_DeserializeFromChain_ReturnsObject()
    {
        var plugin = new TestPlugin();
        var json = """{"name":"Test","value":123}""";
        var bytes = System.Text.Encoding.UTF8.GetBytes(json);

        var result = plugin.DeserializeFromChain(bytes);

        result.Should().NotBeNull();
    }

    #endregion

    #region PluginHooks Tests

    [Fact]
    public void PluginHooks_CanBeCreated()
    {
        var hooks = new PluginHooks();

        hooks.Should().NotBeNull();
        hooks.OnLoad.Should().BeNull();
        hooks.OnUnload.Should().BeNull();
        hooks.BeforeEventProcess.Should().BeNull();
        hooks.AfterDecision.Should().BeNull();
        hooks.OnRulesUpdated.Should().BeNull();
    }

    [Fact]
    public void PluginHooks_OnLoad_CanBeSet()
    {
        var called = false;
        var hooks = new PluginHooks
        {
            OnLoad = () =>
            {
                called = true;
                return Task.CompletedTask;
            }
        };

        hooks.OnLoad!();

        called.Should().BeTrue();
    }

    [Fact]
    public void PluginHooks_OnUnload_CanBeSet()
    {
        var called = false;
        var hooks = new PluginHooks
        {
            OnUnload = () =>
            {
                called = true;
                return Task.CompletedTask;
            }
        };

        hooks.OnUnload!();

        called.Should().BeTrue();
    }

    [Fact]
    public async Task PluginHooks_BeforeEventProcess_CanModifyEvent()
    {
        var hooks = new PluginHooks
        {
            BeforeEventProcess = evt =>
            {
                var modifiedEvent = evt with
                {
                    Metadata = new Dictionary<string, object>(evt.Metadata)
                    {
                        ["processed"] = true
                    }
                };
                return Task.FromResult(modifiedEvent);
            }
        };

        var originalEvent = new PactEvent
        {
            Id = "evt_test",
            Source = "test",
            Type = "test_event",
            OccurredAt = DateTimeOffset.UtcNow,
            ReceivedAt = DateTimeOffset.UtcNow
        };

        var result = await hooks.BeforeEventProcess!(originalEvent);

        result.Metadata.Should().ContainKey("processed");
    }

    [Fact]
    public async Task PluginHooks_AfterDecision_ReceivesEventAndDecision()
    {
        PactEvent? capturedEvent = null;
        PactDecision? capturedDecision = null;

        var hooks = new PluginHooks
        {
            AfterDecision = (evt, decision) =>
            {
                capturedEvent = evt;
                capturedDecision = decision;
                return Task.CompletedTask;
            }
        };

        var testEvent = new PactEvent
        {
            Id = "evt_test",
            Source = "test",
            Type = "test_event",
            OccurredAt = DateTimeOffset.UtcNow,
            ReceivedAt = DateTimeOffset.UtcNow
        };

        var testDecision = new PactDecision
        {
            Id = "dec_test",
            EventId = "evt_test",
            Status = DecisionStatus.Allow,
            CreatedAt = DateTimeOffset.UtcNow
        };

        await hooks.AfterDecision!(testEvent, testDecision);

        capturedEvent.Should().Be(testEvent);
        capturedDecision.Should().Be(testDecision);
    }

    [Fact]
    public async Task PluginHooks_OnRulesUpdated_ReceivesRules()
    {
        List<PactRule>? capturedRules = null;

        var hooks = new PluginHooks
        {
            OnRulesUpdated = rules =>
            {
                capturedRules = rules;
                return Task.CompletedTask;
            }
        };

        var testRules = new List<PactRule>
        {
            new()
            {
                Id = "rule_1",
                Name = "Rule 1",
                Condition = new CompareCondition { Field = "x", Operator = "eq", Value = 1 },
                Consequence = Consequence.Allow("OK", "ok")
            }
        };

        await hooks.OnRulesUpdated!(testRules);

        capturedRules.Should().HaveCount(1);
    }

    [Fact]
    public void Plugin_Hooks_CanBeSetOnPlugin()
    {
        var plugin = new TestPlugin();
        var loadCalled = false;

        plugin.Hooks = new PluginHooks
        {
            OnLoad = () =>
            {
                loadCalled = true;
                return Task.CompletedTask;
            }
        };

        plugin.Hooks.OnLoad!();

        loadCalled.Should().BeTrue();
    }

    #endregion

    #region Plugin GetEvaluationContext Tests

    [Fact]
    public void Plugin_GetEvaluationContext_ReturnsEmptyByDefault()
    {
        var plugin = new TestPlugin();
        var evt = new PactEvent
        {
            Id = "evt_test",
            Source = "test",
            Type = "test_event",
            OccurredAt = DateTimeOffset.UtcNow,
            ReceivedAt = DateTimeOffset.UtcNow
        };

        var context = plugin.GetEvaluationContext(evt, null);

        context.Should().NotBeNull();
        context.Should().BeEmpty();
    }

    #endregion
}
