use crate::domain::entities::{Request, Response};
use anyhow::Result;
use async_trait::async_trait;

/// Trait for HTTP clients to enable mocking and dependency inversion
#[async_trait]
pub trait HttpClient: Send + Sync {
    async fn send(&self, request: Request) -> Result<Response>;
}

/// Application service for orchestrating HTTP request workflows
/// This contains business logic and use cases
pub struct HttpRequestService {
    http_client: Box<dyn HttpClient>,
}

impl HttpRequestService {
    pub fn new(http_client: Box<dyn HttpClient>) -> Self {
        Self { http_client }
    }

    /// Sends a simple HTTP request
    pub async fn send_request(&self, request: Request) -> Result<Response> {
        self.validate_request(&request)?;
        self.http_client.send(request).await
    }

    fn validate_request(&self, request: &Request) -> Result<()> {
        RequestValidator::validate(request)
    }
}

/// Domain service for request validation
/// This contains domain business rules
pub struct RequestValidator;

impl RequestValidator {
    pub fn validate(request: &Request) -> Result<()> {
        Self::validate_url(&request.url)?;
        Self::validate_method_body_combination(request)?;
        Ok(())
    }

    fn validate_url(url: &crate::domain::value_objects::Url) -> Result<()> {
      let url_str = url.as_str();

      if url_str.is_empty() {
          return Err(anyhow::anyhow!("URL cannot be empty"));
      }
      if !url_str.starts_with("http://") && !url_str.starts_with("https://") {
          return Err(anyhow::anyhow!("URL must start with http:// or https://"));
      }
      Ok(())
    }

    fn validate_method_body_combination(request: &Request) -> Result<()> {
        use crate::domain::entities::Method;

        match (&request.method, &request.body) {
            (Method::Get, Some(_)) => {
                Err(anyhow::anyhow!("GET requests should not have a body"))
            },
            _ => Ok(())
        }
    }
}
