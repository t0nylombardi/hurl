use crate::application::builders::request_builder::RequestBuilder;
use crate::application::services::HttpRequestService;
use anyhow::Result;
use clap::Parser;
use colored::Colorize;

/// CLI configuration for Hurl
#[derive(Parser, Debug)]
#[command(
    name = "Hurl",
    version = "0.1.0",
    author = "Anthony Lombardi <me@t0nylombardi.com>"
)]
#[command(about = "Hurl: Rust-powered requests that hit hard", long_about = None)]
pub struct Cli {
    pub url: String,

    #[arg(short, long, default_value = "GET")]
    pub method: String,

    #[arg(short = 'H', long = "header")]
    pub headers: Vec<String>,

    #[arg(short = 'd', long = "data")]
    pub body: Option<String>,

    #[arg(short, long)]
    pub verbose: bool,

    #[arg(short, long)]
    pub output: Option<String>,

    #[arg(long)]
    pub wizard: bool,
}

impl Cli {
    pub async fn run(&self, request_service: &HttpRequestService) -> Result<()> {
        if self.wizard {
            println!("{}", "Wizard mode not implemented yet.".yellow());
            return Ok(());
        }

        let request = RequestBuilder::new()
            .method(&self.method)?
            .url(&self.url)?
            .headers(&self.headers)?
            .body(&self.body)?
            .build();

        let response = request_service.send_request(request).await?;

        if self.verbose {
            println!("{}", format!("Status: {}", response.status).cyan());
        }

        match &self.output {
            Some(path) => {
                std::fs::write(path, &response.body)?;
                if self.verbose {
                    println!("Saved response to {}", path);
                }
            }
            None => crate::infrastructure::output::print_response_body(&response.body)?,
        }

        Ok(())
    }
}
