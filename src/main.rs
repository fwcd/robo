mod headless;
mod gui;
mod controller;
mod protocol;
mod security;
mod server;
mod utils;

use std::sync::Arc;

use clap::Parser;
use security::{ChaChaPolySecurity, EmptySecurity, Security};
use server::ServerContext;
use tokio::sync::mpsc;

fn bootstrap_tracing() {
    let subscriber = tracing_subscriber::FmtSubscriber::new();
    tracing::subscriber::set_global_default(subscriber)
        .expect("Could not set up tracing subscriber");
}

/// Keyboard and mouse server.
#[derive(Parser)]
struct Args {
    /// The host to serve on.
    #[clap(short, long, default_value = "0.0.0.0")]
    host: String,
    /// The port to serve on.
    #[clap(short, long, default_value_t = 19877)]
    port: u16,
    /// Runs the server without encryption.
    #[clap(long)]
    insecure: bool,
    /// Runs the server without a GUI.
    #[clap(long)]
    headless: bool,
}

fn main() {
    bootstrap_tracing();

    let Args { host, port, insecure, headless } = Args::parse();

    let security: Arc<dyn Security + Send + Sync> = if insecure {
        Arc::new(EmptySecurity)
    } else {
        Arc::new(ChaChaPolySecurity::new().expect("Could not set up security"))
    };

    let (tx, rx) = mpsc::channel(4);
    let ctx = ServerContext { host, port, security: security.clone(), main_thread_tx: tx };

    let runtime = tokio::runtime::Builder::new_multi_thread()
        .enable_io()
        .worker_threads(4)
        .build()
        .expect("Could not create Tokio runtime");

    {
        let ctx = ctx.clone();
        runtime.spawn(async move {
            server::run(ctx).await;
        });
    }
    
    if headless {
        headless::bootstrap(rx)
    } else {
        gui::bootstrap(ctx, rx, runtime)
    }
}
