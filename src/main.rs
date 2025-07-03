mod application;
mod domain;
mod infrastructure;
mod presentation;

use clap::Parser;
use crate::application::services::HttpRequestService;
use crate::infrastructure::http_client::HyperHttpClient;
use crate::presentation::cli::Cli;

/// Hurl: Rust-powered HTTP client that hits hard
///
/// A modern, fast, and safe alternative to cURL. Supports GET and POST requests
/// with colored JSON output, automatic headers for JSON, and plans for an
/// interactive TUI wizard, profile-based configs, and request replay.
#[tokio::main]
async fn main() {
    let cli = Cli::parse();
    let http_client = HyperHttpClient::new();
    let request_service = HttpRequestService::new(Box::new(http_client));

    if let Err(err) = cli.run(&request_service).await {
        eprintln!("{}", err);
        std::process::exit(1);
    }
}
