use anyhow::{Result, anyhow};
use colored::Colorize;
use serde_json::Value;

pub fn print_response_body(body: &str) -> Result<()> {
    match serde_json::from_str::<Value>(body) {
        Ok(json) => {
            let pretty = serde_json::to_string_pretty(&json)
                .map_err(|e| anyhow!("Failed to format JSON: {}", e))?;
            println!("{}", pretty.green());
        }
        Err(_) => println!("{}", body.white()),
    }
    Ok(())
}
