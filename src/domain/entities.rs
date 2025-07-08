use crate::domain::value_objects::{JsonBody, Url};
use anyhow::{Result, anyhow};
use hyper::StatusCode;
use std::str::FromStr;

/// HTTP method enum for simplicity
#[derive(Debug, Clone)]
pub enum Method {
    Get,
    Post,
    Put,
    Delete,
    Patch,
    Head,
    Options,
}

impl FromStr for Method {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_uppercase().as_str() {
            "GET" => Ok(Method::Get),
            "POST" => Ok(Method::Post),
            "PUT" => Ok(Method::Put),
            "DELETE" => Ok(Method::Delete),
            "PATCH" => Ok(Method::Patch),
            "HEAD" => Ok(Method::Head),
            "OPTIONS" => Ok(Method::Options),
            other => Err(anyhow!("Unsupported HTTP method: '{}'", other)),
        }
    }
}

/// Represents an HTTP request
#[derive(Debug, Clone)]
pub struct Request {
    pub method: Method,
    pub url: Url,
    pub headers: Vec<(String, String)>, // Key-value pairs for headers
    pub body: Option<JsonBody>,
}

/// Represents an HTTP response
#[derive(Debug, Clone)]
pub struct Response {
    pub status: StatusCode,
    pub body: String,
}
