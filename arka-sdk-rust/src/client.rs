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
