using System.Net.Http.Json;
using System.Text.Json;

namespace Pact.Sdk;

/// <summary>
/// HTTP client for communicating with PACT Core.
/// </summary>
public class PactClient : IDisposable
{
    private readonly HttpClient _httpClient;
    private readonly JsonSerializerOptions _jsonOptions;

    public PactClient(string baseUrl, string? apiKey = null, TimeSpan? timeout = null)
    {
        _httpClient = new HttpClient
        {
            BaseAddress = new Uri(baseUrl.TrimEnd('/')),
            Timeout = timeout ?? TimeSpan.FromSeconds(30)
        };

        _httpClient.DefaultRequestHeaders.Add("Accept", "application/json");
        if (!string.IsNullOrEmpty(apiKey))
        {
            _httpClient.DefaultRequestHeaders.Add("Authorization", $"Bearer {apiKey}");
        }

        _jsonOptions = new JsonSerializerOptions
        {
            PropertyNamingPolicy = JsonNamingPolicy.SnakeCaseLower,
            WriteIndented = false
        };
    }

    // Event Operations

    public async Task<PactDecision> SubmitEventAsync(PactEvent evt, CancellationToken ct = default)
    {
        var response = await _httpClient.PostAsJsonAsync("/api/v1/events", evt, _jsonOptions, ct);
        response.EnsureSuccessStatusCode();
        return await response.Content.ReadFromJsonAsync<PactDecision>(_jsonOptions, ct)
            ?? throw new InvalidOperationException("Empty response");
    }

    public async Task<PactEvent> GetEventAsync(string eventId, CancellationToken ct = default)
    {
        return await _httpClient.GetFromJsonAsync<PactEvent>($"/api/v1/events/{eventId}", _jsonOptions, ct)
            ?? throw new InvalidOperationException("Event not found");
    }

    // Entity Operations

    public async Task<PactEntity> CreateEntityAsync(PactEntity entity, CancellationToken ct = default)
    {
        var response = await _httpClient.PostAsJsonAsync("/api/v1/entities", entity, _jsonOptions, ct);
        response.EnsureSuccessStatusCode();
        return await response.Content.ReadFromJsonAsync<PactEntity>(_jsonOptions, ct)
            ?? throw new InvalidOperationException("Empty response");
    }

    public async Task<PactEntity> GetEntityAsync(string entityId, CancellationToken ct = default)
    {
        return await _httpClient.GetFromJsonAsync<PactEntity>($"/api/v1/entities/{entityId}", _jsonOptions, ct)
            ?? throw new InvalidOperationException("Entity not found");
    }

    public async Task<PactEntity> UpdateEntityAsync(string entityId, Dictionary<string, object> data, CancellationToken ct = default)
    {
        var response = await _httpClient.PatchAsJsonAsync($"/api/v1/entities/{entityId}",
            new { data }, _jsonOptions, ct);
        response.EnsureSuccessStatusCode();
        return await response.Content.ReadFromJsonAsync<PactEntity>(_jsonOptions, ct)
            ?? throw new InvalidOperationException("Empty response");
    }

    public async Task DeleteEntityAsync(string entityId, CancellationToken ct = default)
    {
        var response = await _httpClient.DeleteAsync($"/api/v1/entities/{entityId}", ct);
        response.EnsureSuccessStatusCode();
    }

    // Rule Operations

    public async Task<PactRule> CreateRuleAsync(PactRule rule, CancellationToken ct = default)
    {
        var response = await _httpClient.PostAsJsonAsync("/api/v1/rules", rule, _jsonOptions, ct);
        response.EnsureSuccessStatusCode();
        return await response.Content.ReadFromJsonAsync<PactRule>(_jsonOptions, ct)
            ?? throw new InvalidOperationException("Empty response");
    }

    public async Task<PactRule> GetRuleAsync(string ruleId, CancellationToken ct = default)
    {
        return await _httpClient.GetFromJsonAsync<PactRule>($"/api/v1/rules/{ruleId}", _jsonOptions, ct)
            ?? throw new InvalidOperationException("Rule not found");
    }

    public async Task<PactRule> UpdateRuleAsync(string ruleId, PactRule rule, CancellationToken ct = default)
    {
        var response = await _httpClient.PutAsJsonAsync($"/api/v1/rules/{ruleId}", rule, _jsonOptions, ct);
        response.EnsureSuccessStatusCode();
        return await response.Content.ReadFromJsonAsync<PactRule>(_jsonOptions, ct)
            ?? throw new InvalidOperationException("Empty response");
    }

    public async Task DeleteRuleAsync(string ruleId, CancellationToken ct = default)
    {
        var response = await _httpClient.DeleteAsync($"/api/v1/rules/{ruleId}", ct);
        response.EnsureSuccessStatusCode();
    }

    // Decision Operations

    public async Task<PactDecision> GetDecisionAsync(string decisionId, CancellationToken ct = default)
    {
        return await _httpClient.GetFromJsonAsync<PactDecision>($"/api/v1/decisions/{decisionId}", _jsonOptions, ct)
            ?? throw new InvalidOperationException("Decision not found");
    }

    // Validation Operations

    public async Task<ValidationResult> ValidateAsync(string entityType, Dictionary<string, object> data, CancellationToken ct = default)
    {
        var response = await _httpClient.PostAsJsonAsync("/api/v1/validate",
            new { entity_type = entityType, data }, _jsonOptions, ct);
        response.EnsureSuccessStatusCode();
        return await response.Content.ReadFromJsonAsync<ValidationResult>(_jsonOptions, ct)
            ?? throw new InvalidOperationException("Empty response");
    }

    // Health Operations

    public async Task<bool> IsHealthyAsync(CancellationToken ct = default)
    {
        try
        {
            var response = await _httpClient.GetAsync("/health", ct);
            return response.IsSuccessStatusCode;
        }
        catch
        {
            return false;
        }
    }

    public async Task<bool> IsReadyAsync(CancellationToken ct = default)
    {
        try
        {
            var response = await _httpClient.GetAsync("/ready", ct);
            return response.IsSuccessStatusCode;
        }
        catch
        {
            return false;
        }
    }

    public void Dispose()
    {
        _httpClient.Dispose();
        GC.SuppressFinalize(this);
    }
}
