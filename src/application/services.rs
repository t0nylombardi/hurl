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

    /// Sends a request with retry logic (business logic)
    pub async fn send_with_retry(&self, request: Request, max_retries: u32) -> Result<Response> {
        self.validate_request(&request)?;

        for attempt in 0..=max_retries {
            match self.http_client.send(request.clone()).await {
                Ok(response) => return Ok(response),
                Err(e) if attempt == max_retries => return Err(e),
                Err(_) => continue,
            }
        }

        unreachable!()
    }

    /// Sends multiple requests concurrently
    pub async fn send_batch(&self, requests: Vec<Request>) -> Result<Vec<Response>> {
        for request in &requests {
            self.validate_request(request)?;
        }

        let futures = requests.into_iter()
            .map(|req| self.http_client.send(req));

        let results: Result<Vec<_>> = futures::future::try_join_all(futures).await;
        results
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::entities::{Method};
    use crate::domain::value_objects::{Url, JsonBody};
    use mockall::mock;

    mock! {
        TestHttpClient {}

        #[async_trait]
        impl HttpClient for TestHttpClient {
            async fn send(&self, request: Request) -> Result<Response>;
        }
    }

    #[tokio::test]
    async fn request_service_sends_valid_request() {
        let mut mock_client = MockTestHttpClient::new();
        mock_client
            .expect_send()
            .times(1)
            .returning(|_| Ok(create_test_response()));

        let service = HttpRequestService::new(Box::new(mock_client));
        let request = create_valid_request();

        let result = service.send_request(request).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn request_service_validates_before_sending() {
        let mock_client = MockTestHttpClient::new();
        let service = HttpRequestService::new(Box::new(mock_client));

        let invalid_request = Request {
            url: Url::new("").unwrap(),
            method: Method::Get,
            body: None,
        };

        let result = service.send_request(invalid_request).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn request_service_retries_on_failure() {
        let mut mock_client = MockTestHttpClient::new();
        mock_client
            .expect_send()
            .times(2)
            .returning(|_| Err(anyhow::anyhow!("Network error")))
            .returning(|_| Ok(create_test_response()));

        let service = HttpRequestService::new(Box::new(mock_client));
        let request = create_valid_request();

        let result = service.send_with_retry(request, 2).await;
        assert!(result.is_ok());
    }

    #[test]
    fn validator_rejects_empty_url() {
        let request = Request {
            url: Url::new("").unwrap(),
            method: Method::Get,
            body: None,
        };

        let result = RequestValidator::validate(&request);
        assert!(result.is_err());
    }

    #[test]
    fn validator_rejects_get_with_body() {
        let request = Request {
            url: Url::new("https://example.com").unwrap(),
            method: Method::Get,
            body: Some(JsonBody("{}".to_string())),
        };

        let result = RequestValidator::validate(&request);
        assert!(result.is_err());
    }

    #[test]
    fn validator_accepts_valid_request() {
        let request = create_valid_request();
        let result = RequestValidator::validate(&request);
        assert!(result.is_ok());
    }

    fn create_valid_request() -> Request {
        Request {
            url: Url::new("https://example.com").unwrap(),
            method: Method::Get,
            body: None,
        }
    }

    fn create_test_response() -> Response {
        Response {
            status: hyper::StatusCode::OK,
            body: "test response".to_string(),
        }
    }
}
