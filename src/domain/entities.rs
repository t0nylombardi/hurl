use crate::domain::value_objects::{JsonBody, Url};
use hyper::StatusCode;

/// Represents an HTTP request
#[derive(Debug, Clone)]
pub struct Request {
    pub method: Method,
    pub url: Url,
    pub body: Option<JsonBody>,
}

/// Represents an HTTP response
#[derive(Debug, Clone)]
pub struct Response {
    pub status: StatusCode,
    pub body: String,
}

/// HTTP method enum for simplicity
#[derive(Debug, Clone)]
pub enum Method {
    Get,
    Post,
}
