package pact

import (
	"context"
	"encoding/json"
	"net/http"
	"net/http/httptest"
	"testing"
	"time"
)

func TestNewClient(t *testing.T) {
	t.Run("creates client with default settings", func(t *testing.T) {
		client := NewClient("https://api.example.com")

		if client.baseURL != "https://api.example.com" {
			t.Errorf("expected baseURL %q, got %q", "https://api.example.com", client.baseURL)
		}
		if client.httpClient.Timeout != 30*time.Second {
			t.Errorf("expected timeout %v, got %v", 30*time.Second, client.httpClient.Timeout)
		}
		if client.apiKey != "" {
			t.Errorf("expected empty apiKey, got %q", client.apiKey)
		}
	})

	t.Run("creates client with API key", func(t *testing.T) {
		client := NewClient("https://api.example.com", WithAPIKey("test-key"))

		if client.apiKey != "test-key" {
			t.Errorf("expected apiKey %q, got %q", "test-key", client.apiKey)
		}
	})

	t.Run("creates client with custom timeout", func(t *testing.T) {
		client := NewClient("https://api.example.com", WithTimeout(10*time.Second))

		if client.httpClient.Timeout != 10*time.Second {
			t.Errorf("expected timeout %v, got %v", 10*time.Second, client.httpClient.Timeout)
		}
	})

	t.Run("creates client with custom headers", func(t *testing.T) {
		client := NewClient("https://api.example.com",
			WithHeader("X-Custom-Header", "custom-value"),
			WithHeader("X-Another-Header", "another-value"),
		)

		if client.headers["X-Custom-Header"] != "custom-value" {
			t.Errorf("expected header %q, got %q", "custom-value", client.headers["X-Custom-Header"])
		}
		if client.headers["X-Another-Header"] != "another-value" {
			t.Errorf("expected header %q, got %q", "another-value", client.headers["X-Another-Header"])
		}
	})

	t.Run("creates client with multiple options", func(t *testing.T) {
		client := NewClient("https://api.example.com",
			WithAPIKey("test-key"),
			WithTimeout(15*time.Second),
			WithHeader("X-Custom", "value"),
		)

		if client.apiKey != "test-key" {
			t.Errorf("expected apiKey %q, got %q", "test-key", client.apiKey)
		}
		if client.httpClient.Timeout != 15*time.Second {
			t.Errorf("expected timeout %v, got %v", 15*time.Second, client.httpClient.Timeout)
		}
		if client.headers["X-Custom"] != "value" {
			t.Errorf("expected header %q, got %q", "value", client.headers["X-Custom"])
		}
	})
}

func TestClientSubmitEvent(t *testing.T) {
	t.Run("submits event and returns decision", func(t *testing.T) {
		server := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
			if r.Method != http.MethodPost {
				t.Errorf("expected POST, got %s", r.Method)
			}
			if r.URL.Path != "/api/v1/events" {
				t.Errorf("expected /api/v1/events, got %s", r.URL.Path)
			}
			if r.Header.Get("Content-Type") != "application/json" {
				t.Errorf("expected Content-Type application/json, got %s", r.Header.Get("Content-Type"))
			}
			if r.Header.Get("Authorization") != "Bearer test-key" {
				t.Errorf("expected Authorization Bearer test-key, got %s", r.Header.Get("Authorization"))
			}

			var event PactEvent
			if err := json.NewDecoder(r.Body).Decode(&event); err != nil {
				t.Fatalf("failed to decode request body: %v", err)
			}
			if event.Type != "payment_initiated" {
				t.Errorf("expected event type %q, got %q", "payment_initiated", event.Type)
			}

			decision := PactDecision{
				ID:      "dec_123",
				EventID: event.ID,
				Status:  DecisionStatusAllow,
				RuleEvaluations: []RuleEvaluation{
					{
						RuleID:   "rule_001",
						RuleName: "Amount Limit",
						Result:   EvaluationResultPass,
					},
				},
			}
			json.NewEncoder(w).Encode(decision)
		}))
		defer server.Close()

		client := NewClient(server.URL, WithAPIKey("test-key"))
		ctx := context.Background()

		event := &PactEvent{
			ID:       "evt_123",
			Type:     "payment_initiated",
			EntityID: "user_456",
			Payload:  map[string]interface{}{"amount": 100.0},
		}

		decision, err := client.SubmitEvent(ctx, event)
		if err != nil {
			t.Fatalf("SubmitEvent failed: %v", err)
		}

		if decision.ID != "dec_123" {
			t.Errorf("expected decision ID %q, got %q", "dec_123", decision.ID)
		}
		if decision.Status != DecisionStatusAllow {
			t.Errorf("expected status %q, got %q", DecisionStatusAllow, decision.Status)
		}
		if len(decision.RuleEvaluations) != 1 {
			t.Errorf("expected 1 rule evaluation, got %d", len(decision.RuleEvaluations))
		}
	})

	t.Run("handles server error", func(t *testing.T) {
		server := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
			w.WriteHeader(http.StatusInternalServerError)
			w.Write([]byte(`{"error": "internal server error"}`))
		}))
		defer server.Close()

		client := NewClient(server.URL)
		ctx := context.Background()

		_, err := client.SubmitEvent(ctx, &PactEvent{})
		if err == nil {
			t.Error("expected error for server error response")
		}
	})
}

func TestClientGetEvent(t *testing.T) {
	t.Run("gets event by ID", func(t *testing.T) {
		server := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
			if r.Method != http.MethodGet {
				t.Errorf("expected GET, got %s", r.Method)
			}
			if r.URL.Path != "/api/v1/events/evt_123" {
				t.Errorf("expected /api/v1/events/evt_123, got %s", r.URL.Path)
			}

			event := PactEvent{
				ID:       "evt_123",
				Type:     "payment_initiated",
				EntityID: "user_456",
			}
			json.NewEncoder(w).Encode(event)
		}))
		defer server.Close()

		client := NewClient(server.URL)
		ctx := context.Background()

		event, err := client.GetEvent(ctx, "evt_123")
		if err != nil {
			t.Fatalf("GetEvent failed: %v", err)
		}

		if event.ID != "evt_123" {
			t.Errorf("expected event ID %q, got %q", "evt_123", event.ID)
		}
	})

	t.Run("handles not found", func(t *testing.T) {
		server := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
			w.WriteHeader(http.StatusNotFound)
			w.Write([]byte(`{"error": "event not found"}`))
		}))
		defer server.Close()

		client := NewClient(server.URL)
		ctx := context.Background()

		_, err := client.GetEvent(ctx, "nonexistent")
		if err == nil {
			t.Error("expected error for not found response")
		}
	})
}

func TestClientListEvents(t *testing.T) {
	t.Run("lists events with filters", func(t *testing.T) {
		server := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
			if r.Method != http.MethodGet {
				t.Errorf("expected GET, got %s", r.Method)
			}

			query := r.URL.Query()
			if query.Get("entity_id") != "user_123" {
				t.Errorf("expected entity_id %q, got %q", "user_123", query.Get("entity_id"))
			}
			if query.Get("type") != "payment_initiated" {
				t.Errorf("expected type %q, got %q", "payment_initiated", query.Get("type"))
			}
			if query.Get("limit") != "10" {
				t.Errorf("expected limit %q, got %q", "10", query.Get("limit"))
			}

			response := map[string]interface{}{
				"events": []PactEvent{
					{ID: "evt_1", Type: "payment_initiated"},
					{ID: "evt_2", Type: "payment_initiated"},
				},
			}
			json.NewEncoder(w).Encode(response)
		}))
		defer server.Close()

		client := NewClient(server.URL)
		ctx := context.Background()

		events, err := client.ListEvents(ctx, ListEventsOptions{
			EntityID:  "user_123",
			EventType: "payment_initiated",
			Limit:     10,
		})
		if err != nil {
			t.Fatalf("ListEvents failed: %v", err)
		}

		if len(events) != 2 {
			t.Errorf("expected 2 events, got %d", len(events))
		}
	})
}

func TestClientEntityOperations(t *testing.T) {
	t.Run("creates entity", func(t *testing.T) {
		server := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
			if r.Method != http.MethodPost {
				t.Errorf("expected POST, got %s", r.Method)
			}
			if r.URL.Path != "/api/v1/entities" {
				t.Errorf("expected /api/v1/entities, got %s", r.URL.Path)
			}

			var entity PactEntity
			json.NewDecoder(r.Body).Decode(&entity)
			entity.ID = "ent_123"
			entity.CreatedAt = time.Now()
			json.NewEncoder(w).Encode(entity)
		}))
		defer server.Close()

		client := NewClient(server.URL)
		ctx := context.Background()

		entity := &PactEntity{
			Type: "User",
			Data: map[string]interface{}{"name": "John"},
		}

		created, err := client.CreateEntity(ctx, entity)
		if err != nil {
			t.Fatalf("CreateEntity failed: %v", err)
		}

		if created.ID != "ent_123" {
			t.Errorf("expected entity ID %q, got %q", "ent_123", created.ID)
		}
	})

	t.Run("gets entity by ID", func(t *testing.T) {
		server := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
			if r.Method != http.MethodGet {
				t.Errorf("expected GET, got %s", r.Method)
			}
			if r.URL.Path != "/api/v1/entities/ent_123" {
				t.Errorf("expected /api/v1/entities/ent_123, got %s", r.URL.Path)
			}

			entity := PactEntity{
				ID:   "ent_123",
				Type: "User",
				Data: map[string]interface{}{"name": "John"},
			}
			json.NewEncoder(w).Encode(entity)
		}))
		defer server.Close()

		client := NewClient(server.URL)
		ctx := context.Background()

		entity, err := client.GetEntity(ctx, "ent_123")
		if err != nil {
			t.Fatalf("GetEntity failed: %v", err)
		}

		if entity.ID != "ent_123" {
			t.Errorf("expected entity ID %q, got %q", "ent_123", entity.ID)
		}
	})

	t.Run("updates entity", func(t *testing.T) {
		server := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
			if r.Method != http.MethodPatch {
				t.Errorf("expected PATCH, got %s", r.Method)
			}

			entity := PactEntity{
				ID:   "ent_123",
				Type: "User",
				Data: map[string]interface{}{"name": "Jane"},
			}
			json.NewEncoder(w).Encode(entity)
		}))
		defer server.Close()

		client := NewClient(server.URL)
		ctx := context.Background()

		updated, err := client.UpdateEntity(ctx, "ent_123", map[string]interface{}{"name": "Jane"})
		if err != nil {
			t.Fatalf("UpdateEntity failed: %v", err)
		}

		if updated.Data["name"] != "Jane" {
			t.Errorf("expected name %q, got %q", "Jane", updated.Data["name"])
		}
	})

	t.Run("deletes entity", func(t *testing.T) {
		server := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
			if r.Method != http.MethodDelete {
				t.Errorf("expected DELETE, got %s", r.Method)
			}
			w.WriteHeader(http.StatusNoContent)
		}))
		defer server.Close()

		client := NewClient(server.URL)
		ctx := context.Background()

		err := client.DeleteEntity(ctx, "ent_123")
		if err != nil {
			t.Fatalf("DeleteEntity failed: %v", err)
		}
	})
}

func TestClientRuleOperations(t *testing.T) {
	t.Run("creates rule", func(t *testing.T) {
		server := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
			if r.Method != http.MethodPost {
				t.Errorf("expected POST, got %s", r.Method)
			}

			var rule PactRule
			json.NewDecoder(r.Body).Decode(&rule)
			rule.ID = "rule_123"
			json.NewEncoder(w).Encode(rule)
		}))
		defer server.Close()

		client := NewClient(server.URL)
		ctx := context.Background()

		rule := &PactRule{
			Name:     "Amount Limit",
			Severity: SeverityHigh,
		}

		created, err := client.CreateRule(ctx, rule)
		if err != nil {
			t.Fatalf("CreateRule failed: %v", err)
		}

		if created.ID != "rule_123" {
			t.Errorf("expected rule ID %q, got %q", "rule_123", created.ID)
		}
	})

	t.Run("gets rule by ID", func(t *testing.T) {
		server := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
			if r.Method != http.MethodGet {
				t.Errorf("expected GET, got %s", r.Method)
			}

			rule := PactRule{
				ID:       "rule_123",
				Name:     "Amount Limit",
				Severity: SeverityHigh,
			}
			json.NewEncoder(w).Encode(rule)
		}))
		defer server.Close()

		client := NewClient(server.URL)
		ctx := context.Background()

		rule, err := client.GetRule(ctx, "rule_123")
		if err != nil {
			t.Fatalf("GetRule failed: %v", err)
		}

		if rule.ID != "rule_123" {
			t.Errorf("expected rule ID %q, got %q", "rule_123", rule.ID)
		}
	})

	t.Run("updates rule", func(t *testing.T) {
		server := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
			if r.Method != http.MethodPut {
				t.Errorf("expected PUT, got %s", r.Method)
			}

			rule := PactRule{
				ID:       "rule_123",
				Name:     "Updated Rule",
				Severity: SeverityCritical,
			}
			json.NewEncoder(w).Encode(rule)
		}))
		defer server.Close()

		client := NewClient(server.URL)
		ctx := context.Background()

		rule := &PactRule{Name: "Updated Rule", Severity: SeverityCritical}
		updated, err := client.UpdateRule(ctx, "rule_123", rule)
		if err != nil {
			t.Fatalf("UpdateRule failed: %v", err)
		}

		if updated.Name != "Updated Rule" {
			t.Errorf("expected name %q, got %q", "Updated Rule", updated.Name)
		}
	})

	t.Run("deletes rule", func(t *testing.T) {
		server := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
			if r.Method != http.MethodDelete {
				t.Errorf("expected DELETE, got %s", r.Method)
			}
			w.WriteHeader(http.StatusNoContent)
		}))
		defer server.Close()

		client := NewClient(server.URL)
		ctx := context.Background()

		err := client.DeleteRule(ctx, "rule_123")
		if err != nil {
			t.Fatalf("DeleteRule failed: %v", err)
		}
	})
}

func TestClientGetDecision(t *testing.T) {
	t.Run("gets decision by ID", func(t *testing.T) {
		server := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
			if r.Method != http.MethodGet {
				t.Errorf("expected GET, got %s", r.Method)
			}
			if r.URL.Path != "/api/v1/decisions/dec_123" {
				t.Errorf("expected /api/v1/decisions/dec_123, got %s", r.URL.Path)
			}

			decision := PactDecision{
				ID:      "dec_123",
				EventID: "evt_456",
				Status:  DecisionStatusDeny,
			}
			json.NewEncoder(w).Encode(decision)
		}))
		defer server.Close()

		client := NewClient(server.URL)
		ctx := context.Background()

		decision, err := client.GetDecision(ctx, "dec_123")
		if err != nil {
			t.Fatalf("GetDecision failed: %v", err)
		}

		if decision.ID != "dec_123" {
			t.Errorf("expected decision ID %q, got %q", "dec_123", decision.ID)
		}
		if decision.Status != DecisionStatusDeny {
			t.Errorf("expected status %q, got %q", DecisionStatusDeny, decision.Status)
		}
	})
}

func TestClientValidate(t *testing.T) {
	t.Run("validates data against schema", func(t *testing.T) {
		server := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
			if r.Method != http.MethodPost {
				t.Errorf("expected POST, got %s", r.Method)
			}
			if r.URL.Path != "/api/v1/validate" {
				t.Errorf("expected /api/v1/validate, got %s", r.URL.Path)
			}

			result := ValidationResult{
				Valid: true,
			}
			json.NewEncoder(w).Encode(result)
		}))
		defer server.Close()

		client := NewClient(server.URL)
		ctx := context.Background()

		result, err := client.Validate(ctx, "User", map[string]interface{}{"name": "John"})
		if err != nil {
			t.Fatalf("Validate failed: %v", err)
		}

		if !result.Valid {
			t.Error("expected validation to pass")
		}
	})

	t.Run("returns validation errors", func(t *testing.T) {
		server := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
			result := ValidationResult{
				Valid: false,
				Errors: []ValidationError{
					{Field: "name", Message: "name is required", Code: "REQUIRED"},
				},
			}
			json.NewEncoder(w).Encode(result)
		}))
		defer server.Close()

		client := NewClient(server.URL)
		ctx := context.Background()

		result, err := client.Validate(ctx, "User", map[string]interface{}{})
		if err != nil {
			t.Fatalf("Validate failed: %v", err)
		}

		if result.Valid {
			t.Error("expected validation to fail")
		}
		if len(result.Errors) != 1 {
			t.Errorf("expected 1 error, got %d", len(result.Errors))
		}
	})
}

func TestClientHealth(t *testing.T) {
	t.Run("returns health status", func(t *testing.T) {
		server := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
			if r.URL.Path != "/health" {
				t.Errorf("expected /health, got %s", r.URL.Path)
			}

			health := map[string]interface{}{
				"status":  "healthy",
				"version": "1.0.0",
			}
			json.NewEncoder(w).Encode(health)
		}))
		defer server.Close()

		client := NewClient(server.URL)
		ctx := context.Background()

		health, err := client.Health(ctx)
		if err != nil {
			t.Fatalf("Health failed: %v", err)
		}

		if health["status"] != "healthy" {
			t.Errorf("expected status %q, got %v", "healthy", health["status"])
		}
	})
}

func TestClientReady(t *testing.T) {
	t.Run("returns true when service is ready", func(t *testing.T) {
		server := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
			if r.URL.Path != "/ready" {
				t.Errorf("expected /ready, got %s", r.URL.Path)
			}
			w.WriteHeader(http.StatusOK)
		}))
		defer server.Close()

		client := NewClient(server.URL)
		ctx := context.Background()

		if !client.Ready(ctx) {
			t.Error("expected Ready to return true")
		}
	})

	t.Run("returns false when service is not ready", func(t *testing.T) {
		server := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
			w.WriteHeader(http.StatusServiceUnavailable)
		}))
		defer server.Close()

		client := NewClient(server.URL)
		ctx := context.Background()

		if client.Ready(ctx) {
			t.Error("expected Ready to return false")
		}
	})
}

func TestClientContextCancellation(t *testing.T) {
	t.Run("respects context cancellation", func(t *testing.T) {
		server := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
			time.Sleep(5 * time.Second)
			w.WriteHeader(http.StatusOK)
		}))
		defer server.Close()

		client := NewClient(server.URL)
		ctx, cancel := context.WithTimeout(context.Background(), 100*time.Millisecond)
		defer cancel()

		_, err := client.GetEvent(ctx, "evt_123")
		if err == nil {
			t.Error("expected error due to context cancellation")
		}
	})
}
