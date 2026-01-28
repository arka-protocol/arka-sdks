package pact

import (
	"bytes"
	"context"
	"encoding/json"
	"fmt"
	"io"
	"net/http"
	"net/url"
	"time"
)

// Client is an HTTP client for communicating with PACT Core.
type Client struct {
	baseURL    string
	apiKey     string
	httpClient *http.Client
	headers    map[string]string
}

// ClientOption configures a Client.
type ClientOption func(*Client)

// WithAPIKey sets the API key for authentication.
func WithAPIKey(key string) ClientOption {
	return func(c *Client) {
		c.apiKey = key
	}
}

// WithTimeout sets the HTTP client timeout.
func WithTimeout(timeout time.Duration) ClientOption {
	return func(c *Client) {
		c.httpClient.Timeout = timeout
	}
}

// WithHeader adds a custom header.
func WithHeader(key, value string) ClientOption {
	return func(c *Client) {
		c.headers[key] = value
	}
}

// NewClient creates a new PACT client.
func NewClient(baseURL string, opts ...ClientOption) *Client {
	c := &Client{
		baseURL: baseURL,
		httpClient: &http.Client{
			Timeout: 30 * time.Second,
		},
		headers: make(map[string]string),
	}

	for _, opt := range opts {
		opt(c)
	}

	return c
}

// request performs an HTTP request.
func (c *Client) request(ctx context.Context, method, path string, body interface{}) (*http.Response, error) {
	var bodyReader io.Reader
	if body != nil {
		jsonBody, err := json.Marshal(body)
		if err != nil {
			return nil, fmt.Errorf("failed to marshal body: %w", err)
		}
		bodyReader = bytes.NewReader(jsonBody)
	}

	req, err := http.NewRequestWithContext(ctx, method, c.baseURL+path, bodyReader)
	if err != nil {
		return nil, fmt.Errorf("failed to create request: %w", err)
	}

	req.Header.Set("Content-Type", "application/json")
	req.Header.Set("Accept", "application/json")

	if c.apiKey != "" {
		req.Header.Set("Authorization", "Bearer "+c.apiKey)
	}

	for key, value := range c.headers {
		req.Header.Set(key, value)
	}

	return c.httpClient.Do(req)
}

// parseResponse parses an HTTP response into the target.
func (c *Client) parseResponse(resp *http.Response, target interface{}) error {
	defer resp.Body.Close()

	if resp.StatusCode >= 400 {
		body, _ := io.ReadAll(resp.Body)
		return fmt.Errorf("HTTP %d: %s", resp.StatusCode, string(body))
	}

	if target != nil {
		return json.NewDecoder(resp.Body).Decode(target)
	}
	return nil
}

// SubmitEvent submits an event for evaluation.
func (c *Client) SubmitEvent(ctx context.Context, event *PactEvent) (*PactDecision, error) {
	resp, err := c.request(ctx, http.MethodPost, "/api/v1/events", event)
	if err != nil {
		return nil, err
	}

	var decision PactDecision
	if err := c.parseResponse(resp, &decision); err != nil {
		return nil, err
	}
	return &decision, nil
}

// GetEvent gets an event by ID.
func (c *Client) GetEvent(ctx context.Context, eventID string) (*PactEvent, error) {
	resp, err := c.request(ctx, http.MethodGet, "/api/v1/events/"+eventID, nil)
	if err != nil {
		return nil, err
	}

	var event PactEvent
	if err := c.parseResponse(resp, &event); err != nil {
		return nil, err
	}
	return &event, nil
}

// ListEventsOptions contains options for listing events.
type ListEventsOptions struct {
	EntityID  string
	EventType string
	Limit     int
	Offset    int
}

// ListEvents lists events with optional filters.
func (c *Client) ListEvents(ctx context.Context, opts ListEventsOptions) ([]PactEvent, error) {
	params := url.Values{}
	if opts.EntityID != "" {
		params.Set("entity_id", opts.EntityID)
	}
	if opts.EventType != "" {
		params.Set("type", opts.EventType)
	}
	if opts.Limit > 0 {
		params.Set("limit", fmt.Sprintf("%d", opts.Limit))
	}
	if opts.Offset > 0 {
		params.Set("offset", fmt.Sprintf("%d", opts.Offset))
	}

	path := "/api/v1/events"
	if len(params) > 0 {
		path += "?" + params.Encode()
	}

	resp, err := c.request(ctx, http.MethodGet, path, nil)
	if err != nil {
		return nil, err
	}

	var result struct {
		Events []PactEvent `json:"events"`
	}
	if err := c.parseResponse(resp, &result); err != nil {
		return nil, err
	}
	return result.Events, nil
}

// CreateEntity creates a new entity.
func (c *Client) CreateEntity(ctx context.Context, entity *PactEntity) (*PactEntity, error) {
	resp, err := c.request(ctx, http.MethodPost, "/api/v1/entities", entity)
	if err != nil {
		return nil, err
	}

	var result PactEntity
	if err := c.parseResponse(resp, &result); err != nil {
		return nil, err
	}
	return &result, nil
}

// GetEntity gets an entity by ID.
func (c *Client) GetEntity(ctx context.Context, entityID string) (*PactEntity, error) {
	resp, err := c.request(ctx, http.MethodGet, "/api/v1/entities/"+entityID, nil)
	if err != nil {
		return nil, err
	}

	var entity PactEntity
	if err := c.parseResponse(resp, &entity); err != nil {
		return nil, err
	}
	return &entity, nil
}

// UpdateEntity updates an entity.
func (c *Client) UpdateEntity(ctx context.Context, entityID string, data map[string]interface{}) (*PactEntity, error) {
	resp, err := c.request(ctx, http.MethodPatch, "/api/v1/entities/"+entityID, map[string]interface{}{"data": data})
	if err != nil {
		return nil, err
	}

	var entity PactEntity
	if err := c.parseResponse(resp, &entity); err != nil {
		return nil, err
	}
	return &entity, nil
}

// DeleteEntity deletes an entity.
func (c *Client) DeleteEntity(ctx context.Context, entityID string) error {
	resp, err := c.request(ctx, http.MethodDelete, "/api/v1/entities/"+entityID, nil)
	if err != nil {
		return err
	}
	return c.parseResponse(resp, nil)
}

// CreateRule creates a new rule.
func (c *Client) CreateRule(ctx context.Context, rule *PactRule) (*PactRule, error) {
	resp, err := c.request(ctx, http.MethodPost, "/api/v1/rules", rule)
	if err != nil {
		return nil, err
	}

	var result PactRule
	if err := c.parseResponse(resp, &result); err != nil {
		return nil, err
	}
	return &result, nil
}

// GetRule gets a rule by ID.
func (c *Client) GetRule(ctx context.Context, ruleID string) (*PactRule, error) {
	resp, err := c.request(ctx, http.MethodGet, "/api/v1/rules/"+ruleID, nil)
	if err != nil {
		return nil, err
	}

	var rule PactRule
	if err := c.parseResponse(resp, &rule); err != nil {
		return nil, err
	}
	return &rule, nil
}

// UpdateRule updates a rule.
func (c *Client) UpdateRule(ctx context.Context, ruleID string, rule *PactRule) (*PactRule, error) {
	resp, err := c.request(ctx, http.MethodPut, "/api/v1/rules/"+ruleID, rule)
	if err != nil {
		return nil, err
	}

	var result PactRule
	if err := c.parseResponse(resp, &result); err != nil {
		return nil, err
	}
	return &result, nil
}

// DeleteRule deletes a rule.
func (c *Client) DeleteRule(ctx context.Context, ruleID string) error {
	resp, err := c.request(ctx, http.MethodDelete, "/api/v1/rules/"+ruleID, nil)
	if err != nil {
		return err
	}
	return c.parseResponse(resp, nil)
}

// GetDecision gets a decision by ID.
func (c *Client) GetDecision(ctx context.Context, decisionID string) (*PactDecision, error) {
	resp, err := c.request(ctx, http.MethodGet, "/api/v1/decisions/"+decisionID, nil)
	if err != nil {
		return nil, err
	}

	var decision PactDecision
	if err := c.parseResponse(resp, &decision); err != nil {
		return nil, err
	}
	return &decision, nil
}

// Validate validates data against an entity type schema.
func (c *Client) Validate(ctx context.Context, entityType string, data map[string]interface{}) (*ValidationResult, error) {
	resp, err := c.request(ctx, http.MethodPost, "/api/v1/validate", map[string]interface{}{
		"entity_type": entityType,
		"data":        data,
	})
	if err != nil {
		return nil, err
	}

	var result ValidationResult
	if err := c.parseResponse(resp, &result); err != nil {
		return nil, err
	}
	return &result, nil
}

// Health checks service health.
func (c *Client) Health(ctx context.Context) (map[string]interface{}, error) {
	resp, err := c.request(ctx, http.MethodGet, "/health", nil)
	if err != nil {
		return nil, err
	}

	var result map[string]interface{}
	if err := c.parseResponse(resp, &result); err != nil {
		return nil, err
	}
	return result, nil
}

// Ready checks if service is ready.
func (c *Client) Ready(ctx context.Context) bool {
	resp, err := c.request(ctx, http.MethodGet, "/ready", nil)
	if err != nil {
		return false
	}
	defer resp.Body.Close()
	return resp.StatusCode == http.StatusOK
}
