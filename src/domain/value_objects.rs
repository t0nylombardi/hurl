use anyhow::{anyhow, Result};
use hyper::http::Uri;
use serde_json::Value;

/// Represents a validated URL
#[derive(Debug, Clone)]
pub struct Url(pub Uri);


impl Url {
    /// Creates a new Url with validation
    ///
    /// # Arguments
    /// * `url` - The URL string to parse
    ///
    /// # Returns
    /// * `Ok(Url)` - Validated URL
    /// * `Err(anyhow::Error)` - If the URL is invalid
    pub fn new(url: &str) -> Result<Self> {
        let uri = url.parse::<Uri>().map_err(|e| anyhow!("Invalid URL: {}", e))?;
        Ok(Url(uri))
    }

    /// Returns the URL as a string
    pub fn as_str(&self) -> String {
        self.0.to_string()
    }
}

/// Represents a validated JSON body
#[derive(Debug, Clone)]
pub struct JsonBody(pub String);

impl JsonBody {
    /// Creates a new JsonBody with validation
    ///
    /// # Arguments
    /// * `json` - The JSON string to validate
    ///
    /// # Returns
    /// * `Ok	JsonBody)` - Validated JSON
    /// * `Err(anyhow::Error)` - If the JSON is invalid
    pub fn new(json: &str) -> Result<Self> {
        serde_json::from_str::<Value>(json)
            .map_err(|e| anyhow!("Invalid JSON: {}", e))?;
        Ok(JsonBody(json.to_string()))
    }
}
