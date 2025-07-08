use crate::application::services::HttpRequestService;
use crate::domain::entities::{Method, Request};
use crate::domain::value_objects::{JsonBody, Url};
use anyhow::{Result, anyhow};
use clap::Parser;
use colored::Colorize;
use serde_json::Value;
use std::collections::HashMap;
use std::str::FromStr;

/// CLI configuration for Hurl
#[derive(Parser, Debug)]
#[command(
    name = "Hurl",
    version = "0.1.0",
    author = "Anthony Lombardi <me@t0nylombardi.com>"
)]
#[command(about = "Hurl: Rust-powered requests that hit hard", long_about = None)]
pub struct Cli {
    /// The URL to send the request to
    pub url: String,

    /// HTTP method (GET, POST, PUT, DELETE, etc.)
    #[arg(short, long, default_value = "GET")]
    pub method: String,

    /// Headers in the format "Key: Value"
    #[arg(short = 'H', long = "header")]
    pub headers: Vec<String>,

    /// Request body (usually JSON)
    #[arg(short = 'd', long = "data")]
    pub body: Option<String>,

    /// Enable verbose output
    #[arg(short, long)]
    pub verbose: bool,

    /// Output response to a file
    #[arg(short, long)]
    pub output: Option<String>,

    /// Launch an interactive wizard
    #[arg(long)]
    pub wizard: bool,
}

impl Cli {
    pub async fn run(&self, request_service: &HttpRequestService) -> Result<()> {
        if self.wizard {
            println!("{}", "Wizard mode not implemented yet.".yellow());
            return Ok(());
        }

        let url = Url::new(&self.url)?;
        let method = Method::from_str(&self.method)?;

        let headers = parse_headers(&self.headers)?;
        let body = match &self.body {
            Some(json) => Some(JsonBody::new(json)?),
            None => None,
        };

        let request = Request {
            method,
            url,
            headers: headers.into_iter().collect(),
            body,
        };

        let response = request_service.send_request(request).await?;

        if self.verbose {
            println!("{}", format!("Status: {}", response.status).cyan());
        }

        if let Some(path) = &self.output {
            std::fs::write(path, &response.body)?;
            if self.verbose {
                println!("Saved response to {}", path);
            }
        } else {
            print_body(&response.body)?;
        }

        Ok(())
    }
}

fn parse_headers(raw_headers: &[String]) -> Result<HashMap<String, String>> {
    let mut headers = HashMap::new();
    for raw in raw_headers {
        let parts: Vec<&str> = raw.splitn(2, ':').collect();
        if parts.len() != 2 {
            return Err(anyhow!(
                "Invalid header format: '{}'. Use 'Key: Value'",
                raw
            ));
        }
        headers.insert(parts[0].trim().to_string(), parts[1].trim().to_string());
    }
    Ok(headers)
}

fn print_body(body: &str) -> Result<()> {
    match serde_json::from_str::<Value>(body) {
        Ok(json) => println!(
            "{}",
            serde_json::to_string_pretty(&json)
                .map_err(|e| anyhow!("Failed to format JSON: {}", e))?
                .green()
        ),
        Err(_) => println!("{}", body.white()),
    }
    Ok(())
}
