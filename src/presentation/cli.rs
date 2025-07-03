use crate::application::services::HttpRequestService;
use crate::domain::entities::{Method, Request};
use crate::domain::value_objects::{JsonBody, Url};
use anyhow::{anyhow, Result};
use clap::{Parser, Subcommand};
use colored::Colorize;
use serde_json::Value;

/// CLI configuration for Hurl
#[derive(Parser)]
#[command(name = "Hurl", version = "0.1.0", author = "Anthony Lombardi <me@t0nylombardi.com>")]
#[command(about = "Hurl: Rust-powered requests that hit hard", long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    command: Commands,
}

/// Available Hurl commands
#[derive(Subcommand)]
enum Commands {
    /// Send a GET request to the specified URL
    Get {
        /// The URL to send the GET request to
        #[arg(required = true)]
        url: String,
    },
    /// Send a POST request with JSON data
    Post {
        /// The URL to send the POST request to
        #[arg(required = true)]
        url: String,
        /// JSON data to send in the request body
        #[arg(long)]
        json: String,
    },
    /// Launch an interactive TUI to build requests
    Wizard,
    /// Load a profile with default headers and endpoints
    Profile {
        /// Profile name (e.g., dev-api)
        #[arg(required = true)]
        name: String,
    },
    /// Inspect and replay past requests from a log
    Inspect,
}

impl Cli {
    /// Runs the CLI command
    ///
    /// # Arguments
    /// * `request_service` - The service to handle HTTP requests
    ///
    /// # Returns
    /// * `Ok(())` - If the command succeeds
    /// * `Err(anyhow::Error)` - If the command fails
    pub async fn run(&self, request_service: &HttpRequestService) -> Result<()> {
        match &self.command {
            Commands::Get { url } => {
                let url = Url::new(url)?;
                let request = Request {
                    method: Method::Get,
                    url,
                    body: None,
                };
                let response = request_service.send_request(request).await?;
                println!("{}", format!("Status: {}", response.status).cyan());
                print_body(&response.body)?;
            }
            Commands::Post { url, json } => {
                let url = Url::new(url)?;
                let body = JsonBody::new(json)?;
                let request = Request {
                    method: Method::Post,
                    url,
                    body: Some(body),
                };
                let response = request_service.send_request(request).await?;
                println!("{}", format!("Status: {}", response.status).cyan());
                print_body(&response.body)?;
            }
            Commands::Wizard => {
                println!("{}", "Wizard mode not implemented yet.".yellow());
            }
            Commands::Profile { name } => {
                println!("{}", format!("Profile '{}' not implemented yet.", name).yellow());
            }
            Commands::Inspect => {
                println!("{}", "Inspect mode not implemented yet.".yellow());
            }
        }
        Ok(())
    }
}

/// Prints the response body with colored output
///
/// # Arguments
/// * `body` - The response body as a string
///
/// # Returns
/// * `Ok(())` - If printing succeeds
/// * `Err(anyhow::Error)` - If JSON parsing fails
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
