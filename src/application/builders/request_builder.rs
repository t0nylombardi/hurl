use crate::domain::entities::{Method, Request};
use crate::domain::value_objects::{JsonBody, Url};
use anyhow::{Result, anyhow};
use std::collections::HashMap;
use std::str::FromStr;

pub struct RequestBuilder {
    method: Option<Method>,
    url: Option<Url>,
    headers: HashMap<String, String>,
    body: Option<JsonBody>,
}

impl RequestBuilder {
    pub fn new() -> Self {
        Self {
            method: None,
            url: None,
            headers: HashMap::new(),
            body: None,
        }
    }

    pub fn method(mut self, method: &str) -> Result<Self> {
        self.method = Some(Method::from_str(method)?);
        Ok(self)
    }

    pub fn url(mut self, raw_url: &str) -> Result<Self> {
        self.url = Some(Url::new(raw_url)?);
        Ok(self)
    }

    pub fn headers(mut self, raw_headers: &[String]) -> Result<Self> {
        for raw in raw_headers {
            let parts: Vec<&str> = raw.splitn(2, ':').collect();
            if parts.len() != 2 {
                return Err(anyhow!(
                    "Invalid header format: '{}'. Use 'Key: Value'",
                    raw
                ));
            }
            self.headers
                .insert(parts[0].trim().to_string(), parts[1].trim().to_string());
        }
        Ok(self)
    }

    pub fn body(mut self, json: &Option<String>) -> Result<Self> {
        if let Some(data) = json {
            self.body = Some(JsonBody::new(data)?);
        }
        Ok(self)
    }

    pub fn build(self) -> Request {
        Request {
            method: self.method.expect("Method is required"),
            url: self.url.expect("URL is required"),
            headers: self.headers.into_iter().collect(),
            body: self.body,
        }
    }
}
