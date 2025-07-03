use crate::application::services::HttpRequestService;
use crate::domain::entities::{Request, Response, Method as DomainMethod};
use crate::domain::value_objects::{JsonBody, Url};

use anyhow::{anyhow, Result};
use async_trait::async_trait;
use hyper::body::Bytes;
use hyper::header::{HeaderValue, CONTENT_TYPE};
use hyper::{Method, Request as HyperRequest};
use hyper_util::client::legacy::Client;
use hyper_util::client::legacy::connect::HttpConnector;
use hyper_util::rt::TokioExecutor;
use http_body_util::{BodyExt, Full};

/// Infrastructure implementation of HttpClient using Hyper
/// This is a low-level HTTP transport that the application service uses
pub struct HyperHttpClient {
    client: Client<HttpConnector, Full<Bytes>>,
}

impl HyperHttpClient {
    pub fn new() -> Self {
        let connector = HttpConnector::new();
        let client = Client::builder(TokioExecutor::new())
            .build::<HttpConnector, Full<Bytes>>(connector);
        Self { client }
    }

    /// Creates a configured HTTP request service using this client
    pub fn create_request_service(self) -> HttpRequestService {
        HttpRequestService::new(Box::new(self))
    }
}

impl Default for HyperHttpClient {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl crate::application::services::HttpClient for HyperHttpClient {
    async fn send(&self, request: Request) -> Result<Response> {
        let hyper_request = RequestAdapter::to_hyper_request(request)?;
        let hyper_response = self.execute_http_request(hyper_request).await?;
        ResponseAdapter::to_domain_response(hyper_response).await
    }
}

impl HyperHttpClient {
    async fn execute_http_request(&self, request: HyperRequest<Full<Bytes>>) -> Result<hyper::Response<hyper::body::Incoming>> {
        self.client
            .request(request)
            .await
            .map_err(|e| anyhow!("HTTP request execution failed: {}", e))
    }
}

/// Adapter for converting domain requests to Hyper requests
struct RequestAdapter;

impl RequestAdapter {
    fn to_hyper_request(domain_request: Request) -> Result<HyperRequest<Full<Bytes>>> {
        let method = MethodAdapter::to_hyper_method(domain_request.method);
        let uri = UriAdapter::to_hyper_uri(&domain_request.url);
        let body = BodyAdapter::to_hyper_body(&domain_request.body);

        let mut builder = HyperRequest::builder()
            .method(method)
            .uri(uri);

        builder = HeaderAdapter::add_json_content_type(builder, &domain_request.body);

        builder.body(body)
            .map_err(|e| anyhow!("Failed to build HTTP request: {}", e))
    }
}

/// Adapter for converting domain responses from Hyper responses
struct ResponseAdapter;

impl ResponseAdapter {
    async fn to_domain_response(hyper_response: hyper::Response<hyper::body::Incoming>) -> Result<Response> {
        let status = hyper_response.status();
        let body = Self::extract_response_body(hyper_response).await?;

        Ok(Response { status, body })
    }

    async fn extract_response_body(response: hyper::Response<hyper::body::Incoming>) -> Result<String> {
        let body_bytes = response
            .into_body()
            .collect()
            .await
            .map_err(|e| anyhow!("Failed to read response body: {}", e))?
            .to_bytes();

        String::from_utf8(body_bytes.to_vec())
            .map_err(|e| anyhow!("Response body contains invalid UTF-8: {}", e))
    }
}

/// Adapter for converting domain HTTP methods to Hyper methods
struct MethodAdapter;

impl MethodAdapter {
    fn to_hyper_method(domain_method: DomainMethod) -> Method {
        match domain_method {
            DomainMethod::Get => Method::GET,
            DomainMethod::Post => Method::POST,
        }
    }
}

/// Adapter for converting domain URLs to Hyper URIs
struct UriAdapter;

impl UriAdapter {
    fn to_hyper_uri(domain_url: &Url) -> &hyper::Uri {
        &domain_url.0
    }
}

/// Adapter for converting domain request bodies to Hyper bodies
struct BodyAdapter;

impl BodyAdapter {
    fn to_hyper_body(domain_body: &Option<JsonBody>) -> Full<Bytes> {
        match domain_body {
            Some(json_body) => Full::new(Bytes::from(json_body.0.clone())),
            None => Full::new(Bytes::new()),
        }
    }
}

/// Adapter for handling HTTP headers
struct HeaderAdapter;

impl HeaderAdapter {
    fn add_json_content_type(
        builder: http::request::Builder,
        body: &Option<JsonBody>
    ) -> http::request::Builder {
        if body.is_some() {
            builder.header(CONTENT_TYPE, HeaderValue::from_static("application/json"))
        } else {
            builder
        }
    }
}
