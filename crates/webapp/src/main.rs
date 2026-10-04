#![feature(arc_is_unique)]

use std::path::PathBuf;

use anyhow::Context;
use axum::Router;
use clap::Parser;
use tower_http::{services::ServeDir, trace::TraceLayer};
use tracing::{error, info};
use tracing_subscriber::EnvFilter;

mod api;

#[derive(Debug, Clone, clap::Parser)]
struct CliArgs {
    #[arg(long = "log", default_value = "info")]
    log: EnvFilter,

    #[arg(long = "addr", default_value = "0.0.0.0:11313")]
    addr: String,

    #[arg(long = "frontend-path", default_value = "./frontend/dist")]
    frontend_path: PathBuf,
}

#[tokio::main]
async fn main() {
    let args = CliArgs::parse();

    tracing_subscriber::fmt()
        .pretty()
        .with_env_filter(EnvFilter::from_default_env())
        .with_env_filter(args.log.clone())
        .init();

    if let Err(e) = run(&args).await {
        error!(
            err = ?e,
            backtrace = ?e.backtrace(),
            "Fatal runtime error",
        );
    }
}

async fn run(args: &CliArgs) -> anyhow::Result<()> {
    let app_router = Router::new()
        .nest("/api", api::api_router().context("Initializing API router")?)
        .fallback_service(ServeDir::new(&args.frontend_path));

    let app_listener = tokio::net::TcpListener::bind(&args.addr)
        .await
        .context("Binding webapp listener")?;

    info!("Serving webapp at {}", app_listener.local_addr().unwrap());

    axum::serve(app_listener, app_router)
        .await
        .context("Serving webapp")?;

    Ok(())
}
