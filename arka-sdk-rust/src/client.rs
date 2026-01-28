//! PACT Client
//!
//! HTTP client for communicating with PACT Core.

use reqwest::{Client as HttpClient, StatusCode};
use std::collections::HashMap;
use std::time::Duration;
use thiserror::Error;

use crate::types::*;

/// Client errors.
#[derive(Error, Debug)]
pub enum ClientError {
    #[error("HTTP error: {0}")]
    Http(#[from] reqwest::Error),

    #[error("API error {status}: {message}")]
    Api { status: u16, message: String },

    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
}

/// Client for communicating with PACT Core.
pub struct PactClient {
    base_url: String,
    api_key: Option<String>,
    http: HttpClient,
}

impl PactClient {
    /// Creates a new PACT client.
    pub fn new(base_url: &str) -> Self {
        Self {
            base_url: base_url.trim_end_matches('/').to_string(),
            api_key: None,
            http: HttpClient::builder()
                .timeout(Duration::from_secs(30))
                .build()
                .expect("Failed to create HTTP client"),
        }
    }

    /// Sets the API key.
    pub fn with_api_key(mut self, key: &str) -> Self {
        self.api_key = Some(key.to_string());
        self
    }

    /// Sets a custom timeout.
    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        self.http = HttpClient::builder()
            .timeout(timeout)
            .build()
            .expect("Failed to create HTTP client");
        self
    }

    /// Performs a request and handles errors.
    async fn request<T: serde::de::DeserializeOwned>(
        &self,
        method: reqwest::Method,
        path: &str,
        body: Option<&impl serde::Serialize>,
    ) -> Result<T, ClientError> {
        let url = format!("{}{}", self.base_url, path);
        let mut request = self.http.request(method, &url);

        request = request
            .header("Content-Type", "application/json")
            .header("Accept", "application/json");

        if let Some(key) = &self.api_key {
            request = request.header("Authorization", format!("Bearer {}", key));
        }

        if let Some(b) = body {
            request = request.json(b);
        }

        let response = request.send().await?;
        let status = response.status();

        if status.is_success() {
            let result = response.json().await?;
            Ok(result)
        } else {
            let message = response.text().await.unwrap_or_default();
            Err(ClientError::Api {
                status: status.as_u16(),
                message,
            })
        }
    }

    // =========================================================================
    // Event Operations
    // =========================================================================

    /// Submits an event for evaluation.
    pub async fn submit_event(&self, event: &PactEvent) -> Result<PactDecision, ClientError> {
        self.request(reqwest::Method::POST, "/api/v1/events", Some(event))
            .await
    }

    /// Gets an event by ID.
    pub async fn get_event(&self, event_id: &str) -> Result<PactEvent, ClientError> {
        self.request(
            reqwest::Method::GET,
            &format!("/api/v1/events/{}", event_id),
            None::<&()>,
        )
        .await
    }

    /// Lists events with optional filters.
    pub async fn list_events(
        &self,
        entity_id: Option<&str>,
        event_type: Option<&str>,
        limit: Option<u32>,
        offset: Option<u32>,
    ) -> Result<Vec<PactEvent>, ClientError> {
        let mut params = Vec::new();
        if let Some(id) = entity_id {
            params.push(format!("entity_id={}", id));
        }
        if let Some(t) = event_type {
            params.push(format!("type={}", t));
        }
        if let Some(l) = limit {
            params.push(format!("limit={}", l));
        }
        if let Some(o) = offset {
            params.push(format!("offset={}", o));
        }

        let path = if params.is_empty() {
            "/api/v1/events".to_string()
        } else {
            format!("/api/v1/events?{}", params.join("&"))
        };

        #[derive(serde::Deserialize)]
        struct Response {
            events: Vec<PactEvent>,
        }

        let response: Response = self.request(reqwest::Method::GET, &path, None::<&()>).await?;
        Ok(response.events)
    }

    // =========================================================================
    // Entity Operations
    // =========================================================================

    /// Creates a new entity.
    pub async fn create_entity(&self, entity: &PactEntity) -> Result<PactEntity, ClientError> {
        self.request(reqwest::Method::POST, "/api/v1/entities", Some(entity))
            .await
    }

    /// Gets an entity by ID.
    pub async fn get_entity(&self, entity_id: &str) -> Result<PactEntity, ClientError> {
        self.request(
            reqwest::Method::GET,
            &format!("/api/v1/entities/{}", entity_id),
            None::<&()>,
        )
        .await
    }

    /// Updates an entity.
    pub async fn update_entity(
        &self,
        entity_id: &str,
        data: HashMap<String, serde_json::Value>,
    ) -> Result<PactEntity, ClientError> {
        #[derive(serde::Serialize)]
        struct UpdateRequest {
            data: HashMap<String, serde_json::Value>,
        }

        self.request(
            reqwest::Method::PATCH,
            &format!("/api/v1/entities/{}", entity_id),
            Some(&UpdateRequest { data }),
        )
        .await
    }

    /// Deletes an entity.
    pub async fn delete_entity(&self, entity_id: &str) -> Result<(), ClientError> {
        let _: serde_json::Value = self
            .request(
                reqwest::Method::DELETE,
                &format!("/api/v1/entities/{}", entity_id),
                None::<&()>,
            )
            .await?;
        Ok(())
    }

    // =========================================================================
    // Rule Operations
    // =========================================================================

    /// Creates a new rule.
    pub async fn create_rule(&self, rule: &PactRule) -> Result<PactRule, ClientError> {
        self.request(reqwest::Method::POST, "/api/v1/rules", Some(rule))
            .await
    }

    /// Gets a rule by ID.
    pub async fn get_rule(&self, rule_id: &str) -> Result<PactRule, ClientError> {
        self.request(
            reqwest::Method::GET,
            &format!("/api/v1/rules/{}", rule_id),
            None::<&()>,
        )
        .await
    }

    /// Updates a rule.
    pub async fn update_rule(&self, rule_id: &str, rule: &PactRule) -> Result<PactRule, ClientError> {
        self.request(
            reqwest::Method::PUT,
            &format!("/api/v1/rules/{}", rule_id),
            Some(rule),
        )
        .await
    }

    /// Deletes a rule.
    pub async fn delete_rule(&self, rule_id: &str) -> Result<(), ClientError> {
        let _: serde_json::Value = self
            .request(
                reqwest::Method::DELETE,
                &format!("/api/v1/rules/{}", rule_id),
                None::<&()>,
            )
            .await?;
        Ok(())
    }

    // =========================================================================
    // Decision Operations
    // =========================================================================

    /// Gets a decision by ID.
    pub async fn get_decision(&self, decision_id: &str) -> Result<PactDecision, ClientError> {
        self.request(
            reqwest::Method::GET,
            &format!("/api/v1/decisions/{}", decision_id),
            None::<&()>,
        )
        .await
    }

    // =========================================================================
    // Validation Operations
    // =========================================================================

    /// Validates data against an entity type schema.
    pub async fn validate(
        &self,
        entity_type: &str,
        data: HashMap<String, serde_json::Value>,
    ) -> Result<ValidationResult, ClientError> {
        #[derive(serde::Serialize)]
        struct ValidateRequest {
            entity_type: String,
            data: HashMap<String, serde_json::Value>,
        }

        self.request(
            reqwest::Method::POST,
            "/api/v1/validate",
            Some(&ValidateRequest {
                entity_type: entity_type.to_string(),
                data,
            }),
        )
        .await
    }

    // =========================================================================
    // Health Operations
    // =========================================================================

    /// Checks service health.
    pub async fn health(&self) -> Result<HashMap<String, serde_json::Value>, ClientError> {
        self.request(reqwest::Method::GET, "/health", None::<&()>)
            .await
    }

    /// Checks if service is ready.
    pub async fn ready(&self) -> bool {
        let url = format!("{}/ready", self.base_url);
        match self.http.get(&url).send().await {
            Ok(resp) => resp.status() == StatusCode::OK,
            Err(_) => false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use wiremock::{Mock, MockServer, ResponseTemplate};
    use wiremock::matchers::{method, path, header};
    use serde_json::json;

    // =========================================================================
    // Client Initialization Tests
    // =========================================================================

    #[test]
    fn test_client_new() {
        let client = PactClient::new("http://localhost:8080");
        assert_eq!(client.base_url, "http://localhost:8080");
        assert!(client.api_key.is_none());
    }

    #[test]
    fn test_client_new_strips_trailing_slash() {
        let client = PactClient::new("http://localhost:8080/");
        assert_eq!(client.base_url, "http://localhost:8080");
    }

    #[test]
    fn test_client_with_api_key() {
        let client = PactClient::new("http://localhost:8080")
            .with_api_key("test-api-key");

        assert_eq!(client.api_key, Some("test-api-key".to_string()));
    }

    #[test]
    fn test_client_with_timeout() {
        let client = PactClient::new("http://localhost:8080")
            .with_timeout(Duration::from_secs(60));

        // Can't directly test timeout, but ensure client is created without panic
        assert_eq!(client.base_url, "http://localhost:8080");
    }

    #[test]
    fn test_client_builder_chain() {
        let client = PactClient::new("http://localhost:8080")
            .with_api_key("key")
            .with_timeout(Duration::from_secs(10));

        assert_eq!(client.api_key, Some("key".to_string()));
    }

    // =========================================================================
    // ClientError Tests
    // =========================================================================

    #[test]
    fn test_client_error_display() {
        let api_err = ClientError::Api {
            status: 404,
            message: "Not found".to_string(),
        };
        assert_eq!(api_err.to_string(), "API error 404: Not found");

        let json_err = serde_json::from_str::<serde_json::Value>("invalid").unwrap_err();
        let ser_err = ClientError::Serialization(json_err);
        assert!(ser_err.to_string().starts_with("Serialization error:"));
    }

    // =========================================================================
    // Health Operations Tests
    // =========================================================================

    #[tokio::test]
    async fn test_health_success() {
        let mock_server = MockServer::start().await;

        Mock::given(method("GET"))
            .and(path("/health"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({
                "status": "healthy",
                "version": "1.0.0"
            })))
            .mount(&mock_server)
            .await;

        let client = PactClient::new(&mock_server.uri());
        let result = client.health().await;

        assert!(result.is_ok());
        let health = result.unwrap();
        assert_eq!(health.get("status"), Some(&json!("healthy")));
    }

    #[tokio::test]
    async fn test_ready_success() {
        let mock_server = MockServer::start().await;

        Mock::given(method("GET"))
            .and(path("/ready"))
            .respond_with(ResponseTemplate::new(200))
            .mount(&mock_server)
            .await;

        let client = PactClient::new(&mock_server.uri());
        let result = client.ready().await;

        assert!(result);
    }

    #[tokio::test]
    async fn test_ready_not_ready() {
        let mock_server = MockServer::start().await;

        Mock::given(method("GET"))
            .and(path("/ready"))
            .respond_with(ResponseTemplate::new(503))
            .mount(&mock_server)
            .await;

        let client = PactClient::new(&mock_server.uri());
        let result = client.ready().await;

        assert!(!result);
    }

    #[tokio::test]
    async fn test_ready_connection_error() {
        // Use a non-existent server
        let client = PactClient::new("http://localhost:59999");
        let result = client.ready().await;

        assert!(!result);
    }

    // =========================================================================
    // Event Operations Tests
    // =========================================================================

    #[tokio::test]
    async fn test_submit_event_success() {
        let mock_server = MockServer::start().await;
        let now = chrono::Utc::now();

        Mock::given(method("POST"))
            .and(path("/api/v1/events"))
            .and(header("Content-Type", "application/json"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({
                "id": "dec_123",
                "event_id": "evt_456",
                "status": "ALLOW",
                "rule_evaluations": [],
                "created_at": now.to_rfc3339()
            })))
            .mount(&mock_server)
            .await;

        let client = PactClient::new(&mock_server.uri());

        let event = PactEvent {
            id: "evt_456".to_string(),
            source: "test".to_string(),
            event_type: "TEST_EVENT".to_string(),
            entity_id: Some("entity-1".to_string()),
            entity_type: Some("TestEntity".to_string()),
            jurisdiction: Some("US".to_string()),
            payload: HashMap::new(),
            occurred_at: now,
            received_at: now,
            metadata: HashMap::new(),
        };

        let result = client.submit_event(&event).await;

        assert!(result.is_ok());
        let decision = result.unwrap();
        assert_eq!(decision.id, "dec_123");
        assert_eq!(decision.event_id, "evt_456");
        assert_eq!(decision.status, DecisionStatus::Allow);
    }

    #[tokio::test]
    async fn test_get_event_success() {
        let mock_server = MockServer::start().await;
        let now = chrono::Utc::now();

        Mock::given(method("GET"))
            .and(path("/api/v1/events/evt_123"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({
                "id": "evt_123",
                "source": "test-plugin",
                "type": "TEST_EVENT",
                "entity_id": "entity-1",
                "payload": {"amount": 1000},
                "occurred_at": now.to_rfc3339(),
                "received_at": now.to_rfc3339()
            })))
            .mount(&mock_server)
            .await;

        let client = PactClient::new(&mock_server.uri());
        let result = client.get_event("evt_123").await;

        assert!(result.is_ok());
        let event = result.unwrap();
        assert_eq!(event.id, "evt_123");
        assert_eq!(event.event_type, "TEST_EVENT");
    }

    #[tokio::test]
    async fn test_get_event_not_found() {
        let mock_server = MockServer::start().await;

        Mock::given(method("GET"))
            .and(path("/api/v1/events/nonexistent"))
            .respond_with(ResponseTemplate::new(404).set_body_string("Event not found"))
            .mount(&mock_server)
            .await;

        let client = PactClient::new(&mock_server.uri());
        let result = client.get_event("nonexistent").await;

        assert!(result.is_err());
        match result.unwrap_err() {
            ClientError::Api { status, message } => {
                assert_eq!(status, 404);
                assert_eq!(message, "Event not found");
            }
            _ => panic!("Expected API error"),
        }
    }

    #[tokio::test]
    async fn test_list_events_success() {
        let mock_server = MockServer::start().await;
        let now = chrono::Utc::now();

        Mock::given(method("GET"))
            .and(path("/api/v1/events"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({
                "events": [
                    {
                        "id": "evt_1",
                        "source": "test",
                        "type": "EVENT_A",
                        "payload": {},
                        "occurred_at": now.to_rfc3339(),
                        "received_at": now.to_rfc3339()
                    },
                    {
                        "id": "evt_2",
                        "source": "test",
                        "type": "EVENT_B",
                        "payload": {},
                        "occurred_at": now.to_rfc3339(),
                        "received_at": now.to_rfc3339()
                    }
                ]
            })))
            .mount(&mock_server)
            .await;

        let client = PactClient::new(&mock_server.uri());
        let result = client.list_events(None, None, None, None).await;

        assert!(result.is_ok());
        let events = result.unwrap();
        assert_eq!(events.len(), 2);
    }

    #[tokio::test]
    async fn test_list_events_with_filters() {
        let mock_server = MockServer::start().await;

        Mock::given(method("GET"))
            .and(path("/api/v1/events"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({
                "events": []
            })))
            .mount(&mock_server)
            .await;

        let client = PactClient::new(&mock_server.uri());
        let result = client.list_events(
            Some("entity-1"),
            Some("TEST_EVENT"),
            Some(10),
            Some(0),
        ).await;

        assert!(result.is_ok());
    }

    // =========================================================================
    // Entity Operations Tests
    // =========================================================================

    #[tokio::test]
    async fn test_create_entity_success() {
        let mock_server = MockServer::start().await;
        let now = chrono::Utc::now();

        Mock::given(method("POST"))
            .and(path("/api/v1/entities"))
            .respond_with(ResponseTemplate::new(201).set_body_json(json!({
                "id": "ent_123",
                "type": "Account",
                "data": {"name": "Test Account"},
                "created_at": now.to_rfc3339(),
                "updated_at": now.to_rfc3339()
            })))
            .mount(&mock_server)
            .await;

        let client = PactClient::new(&mock_server.uri());

        let mut data = HashMap::new();
        data.insert("name".to_string(), json!("Test Account"));

        let entity = PactEntity {
            id: "".to_string(),
            entity_type: "Account".to_string(),
            data,
            created_at: now,
            updated_at: now,
            jurisdiction: None,
            metadata: HashMap::new(),
        };

        let result = client.create_entity(&entity).await;

        assert!(result.is_ok());
        let created = result.unwrap();
        assert_eq!(created.id, "ent_123");
        assert_eq!(created.entity_type, "Account");
    }

    #[tokio::test]
    async fn test_get_entity_success() {
        let mock_server = MockServer::start().await;
        let now = chrono::Utc::now();

        Mock::given(method("GET"))
            .and(path("/api/v1/entities/ent_123"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({
                "id": "ent_123",
                "type": "Account",
                "data": {"balance": 1000},
                "created_at": now.to_rfc3339(),
                "updated_at": now.to_rfc3339()
            })))
            .mount(&mock_server)
            .await;

        let client = PactClient::new(&mock_server.uri());
        let result = client.get_entity("ent_123").await;

        assert!(result.is_ok());
        let entity = result.unwrap();
        assert_eq!(entity.id, "ent_123");
        assert_eq!(entity.data.get("balance"), Some(&json!(1000)));
    }

    #[tokio::test]
    async fn test_update_entity_success() {
        let mock_server = MockServer::start().await;
        let now = chrono::Utc::now();

        Mock::given(method("PATCH"))
            .and(path("/api/v1/entities/ent_123"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({
                "id": "ent_123",
                "type": "Account",
                "data": {"balance": 2000},
                "created_at": now.to_rfc3339(),
                "updated_at": now.to_rfc3339()
            })))
            .mount(&mock_server)
            .await;

        let client = PactClient::new(&mock_server.uri());

        let mut updates = HashMap::new();
        updates.insert("balance".to_string(), json!(2000));

        let result = client.update_entity("ent_123", updates).await;

        assert!(result.is_ok());
        let entity = result.unwrap();
        assert_eq!(entity.data.get("balance"), Some(&json!(2000)));
    }

    #[tokio::test]
    async fn test_delete_entity_success() {
        let mock_server = MockServer::start().await;

        Mock::given(method("DELETE"))
            .and(path("/api/v1/entities/ent_123"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({
                "success": true
            })))
            .mount(&mock_server)
            .await;

        let client = PactClient::new(&mock_server.uri());
        let result = client.delete_entity("ent_123").await;

        assert!(result.is_ok());
    }

    // =========================================================================
    // Rule Operations Tests
    // =========================================================================

    #[tokio::test]
    async fn test_create_rule_success() {
        let mock_server = MockServer::start().await;

        Mock::given(method("POST"))
            .and(path("/api/v1/rules"))
            .respond_with(ResponseTemplate::new(201).set_body_json(json!({
                "id": "rule_123",
                "name": "Test Rule",
                "description": "A test rule",
                "severity": "MEDIUM",
                "condition": {
                    "type": "compare",
                    "field": "amount",
                    "operator": "gt",
                    "value": 1000
                },
                "consequence": {
                    "decision": "FLAG",
                    "code": "HIGH_VALUE",
                    "message": "High value"
                }
            })))
            .mount(&mock_server)
            .await;

        let client = PactClient::new(&mock_server.uri());

        let rule = PactRule {
            id: "".to_string(),
            name: "Test Rule".to_string(),
            description: "A test rule".to_string(),
            jurisdiction: None,
            severity: Severity::Medium,
            condition: Condition::Compare {
                field: "amount".to_string(),
                operator: "gt".to_string(),
                value: json!(1000),
            },
            consequence: Consequence {
                decision: Decision::Flag,
                code: "HIGH_VALUE".to_string(),
                message: "High value".to_string(),
                metadata: HashMap::new(),
            },
            tags: vec![],
            effective_from: None,
            effective_to: None,
            metadata: HashMap::new(),
        };

        let result = client.create_rule(&rule).await;

        assert!(result.is_ok());
        let created = result.unwrap();
        assert_eq!(created.id, "rule_123");
    }

    #[tokio::test]
    async fn test_get_rule_success() {
        let mock_server = MockServer::start().await;

        Mock::given(method("GET"))
            .and(path("/api/v1/rules/rule_123"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({
                "id": "rule_123",
                "name": "Test Rule",
                "description": "A test rule",
                "severity": "HIGH",
                "condition": {
                    "type": "exists",
                    "field": "risk_score"
                },
                "consequence": {
                    "decision": "DENY",
                    "code": "DENIED",
                    "message": "Denied"
                }
            })))
            .mount(&mock_server)
            .await;

        let client = PactClient::new(&mock_server.uri());
        let result = client.get_rule("rule_123").await;

        assert!(result.is_ok());
        let rule = result.unwrap();
        assert_eq!(rule.id, "rule_123");
        assert_eq!(rule.severity, Severity::High);
    }

    #[tokio::test]
    async fn test_update_rule_success() {
        let mock_server = MockServer::start().await;

        Mock::given(method("PUT"))
            .and(path("/api/v1/rules/rule_123"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({
                "id": "rule_123",
                "name": "Updated Rule",
                "description": "Updated description",
                "severity": "CRITICAL",
                "condition": {
                    "type": "compare",
                    "field": "amount",
                    "operator": "gt",
                    "value": 50000
                },
                "consequence": {
                    "decision": "DENY",
                    "code": "LIMIT",
                    "message": "Limit exceeded"
                }
            })))
            .mount(&mock_server)
            .await;

        let client = PactClient::new(&mock_server.uri());

        let rule = PactRule {
            id: "rule_123".to_string(),
            name: "Updated Rule".to_string(),
            description: "Updated description".to_string(),
            jurisdiction: None,
            severity: Severity::Critical,
            condition: Condition::Compare {
                field: "amount".to_string(),
                operator: "gt".to_string(),
                value: json!(50000),
            },
            consequence: Consequence {
                decision: Decision::Deny,
                code: "LIMIT".to_string(),
                message: "Limit exceeded".to_string(),
                metadata: HashMap::new(),
            },
            tags: vec![],
            effective_from: None,
            effective_to: None,
            metadata: HashMap::new(),
        };

        let result = client.update_rule("rule_123", &rule).await;

        assert!(result.is_ok());
        let updated = result.unwrap();
        assert_eq!(updated.name, "Updated Rule");
        assert_eq!(updated.severity, Severity::Critical);
    }

    #[tokio::test]
    async fn test_delete_rule_success() {
        let mock_server = MockServer::start().await;

        Mock::given(method("DELETE"))
            .and(path("/api/v1/rules/rule_123"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({
                "success": true
            })))
            .mount(&mock_server)
            .await;

        let client = PactClient::new(&mock_server.uri());
        let result = client.delete_rule("rule_123").await;

        assert!(result.is_ok());
    }

    // =========================================================================
    // Decision Operations Tests
    // =========================================================================

    #[tokio::test]
    async fn test_get_decision_success() {
        let mock_server = MockServer::start().await;
        let now = chrono::Utc::now();

        Mock::given(method("GET"))
            .and(path("/api/v1/decisions/dec_123"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({
                "id": "dec_123",
                "event_id": "evt_456",
                "status": "ALLOW_WITH_FLAGS",
                "rule_evaluations": [
                    {
                        "rule_id": "rule_1",
                        "rule_name": "Rule 1",
                        "result": "PASS",
                        "duration_ms": 5
                    },
                    {
                        "rule_id": "rule_2",
                        "rule_name": "Rule 2",
                        "result": "FAIL",
                        "code": "FLAG_1",
                        "message": "Flagged",
                        "duration_ms": 3
                    }
                ],
                "created_at": now.to_rfc3339()
            })))
            .mount(&mock_server)
            .await;

        let client = PactClient::new(&mock_server.uri());
        let result = client.get_decision("dec_123").await;

        assert!(result.is_ok());
        let decision = result.unwrap();
        assert_eq!(decision.id, "dec_123");
        assert_eq!(decision.status, DecisionStatus::AllowWithFlags);
        assert_eq!(decision.rule_evaluations.len(), 2);
    }

    // =========================================================================
    // Validation Operations Tests
    // =========================================================================

    #[tokio::test]
    async fn test_validate_success() {
        let mock_server = MockServer::start().await;

        Mock::given(method("POST"))
            .and(path("/api/v1/validate"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({
                "valid": true,
                "errors": [],
                "warnings": []
            })))
            .mount(&mock_server)
            .await;

        let client = PactClient::new(&mock_server.uri());

        let mut data = HashMap::new();
        data.insert("name".to_string(), json!("Test"));
        data.insert("email".to_string(), json!("test@example.com"));

        let result = client.validate("User", data).await;

        assert!(result.is_ok());
        let validation = result.unwrap();
        assert!(validation.valid);
        assert!(validation.errors.is_empty());
    }

    #[tokio::test]
    async fn test_validate_with_errors() {
        let mock_server = MockServer::start().await;

        Mock::given(method("POST"))
            .and(path("/api/v1/validate"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({
                "valid": false,
                "errors": [
                    {
                        "field": "email",
                        "message": "Invalid email format",
                        "code": "INVALID_FORMAT"
                    }
                ],
                "warnings": [
                    {
                        "field": "phone",
                        "message": "Phone number is recommended"
                    }
                ]
            })))
            .mount(&mock_server)
            .await;

        let client = PactClient::new(&mock_server.uri());

        let mut data = HashMap::new();
        data.insert("email".to_string(), json!("invalid"));

        let result = client.validate("User", data).await;

        assert!(result.is_ok());
        let validation = result.unwrap();
        assert!(!validation.valid);
        assert_eq!(validation.errors.len(), 1);
        assert_eq!(validation.errors[0].field, "email");
        assert_eq!(validation.warnings.len(), 1);
    }

    // =========================================================================
    // API Key Tests
    // =========================================================================

    #[tokio::test]
    async fn test_api_key_header() {
        let mock_server = MockServer::start().await;

        Mock::given(method("GET"))
            .and(path("/health"))
            .and(header("Authorization", "Bearer test-api-key"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({
                "status": "ok"
            })))
            .mount(&mock_server)
            .await;

        let client = PactClient::new(&mock_server.uri())
            .with_api_key("test-api-key");

        let result = client.health().await;

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_unauthorized_error() {
        let mock_server = MockServer::start().await;

        Mock::given(method("GET"))
            .and(path("/api/v1/events/evt_123"))
            .respond_with(ResponseTemplate::new(401).set_body_string("Unauthorized"))
            .mount(&mock_server)
            .await;

        let client = PactClient::new(&mock_server.uri());
        let result = client.get_event("evt_123").await;

        assert!(result.is_err());
        match result.unwrap_err() {
            ClientError::Api { status, .. } => {
                assert_eq!(status, 401);
            }
            _ => panic!("Expected API error"),
        }
    }

    // =========================================================================
    // Error Handling Tests
    // =========================================================================

    #[tokio::test]
    async fn test_server_error() {
        let mock_server = MockServer::start().await;

        Mock::given(method("GET"))
            .and(path("/health"))
            .respond_with(ResponseTemplate::new(500).set_body_string("Internal Server Error"))
            .mount(&mock_server)
            .await;

        let client = PactClient::new(&mock_server.uri());
        let result = client.health().await;

        assert!(result.is_err());
        match result.unwrap_err() {
            ClientError::Api { status, message } => {
                assert_eq!(status, 500);
                assert_eq!(message, "Internal Server Error");
            }
            _ => panic!("Expected API error"),
        }
    }

    #[tokio::test]
    async fn test_connection_error() {
        let client = PactClient::new("http://localhost:59999");
        let result = client.health().await;

        assert!(result.is_err());
        match result.unwrap_err() {
            ClientError::Http(_) => {}
            _ => panic!("Expected HTTP error"),
        }
    }
}
