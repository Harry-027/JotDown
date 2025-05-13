pub mod formatter;
pub mod jot;
pub mod notion;
pub mod util;

use anyhow::Result;
use jot::Jotter;
use notion::Notion;
use rmcp::{ServiceExt, transport::stdio};
use std::env;
use tracing_subscriber::{self, EnvFilter};

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize the tracing subscriber with file and stdout logging
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env().add_directive(tracing::Level::DEBUG.into()))
        .with_writer(std::io::stderr)
        .with_ansi(false)
        .init();

    tracing::info!("Starting Jotdown MCP server");

    dotenv::dotenv().ok();
    let token = env::var("NOTION_TOKEN").expect("NOTION_TOKEN not found");
    let data_store = Notion::new(&token);

    let service = Jotter::new(data_store)
        .serve(stdio())
        .await
        .inspect_err(|e| {
            tracing::error!("serving error: {:?}", e);
        })?;

    service.waiting().await?;
    Ok(())
}
