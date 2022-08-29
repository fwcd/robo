mod ui;
mod state;

use clap::Parser;
use druid::{AppLauncher, WindowDesc};
use local_ip_address::local_ip;
use state::AppState;
use tokio::net::TcpListener;
use tracing::info;
use ui::app_widget;

fn bootstrap_tracing() {
    let subscriber = tracing_subscriber::FmtSubscriber::new();
    tracing::subscriber::set_global_default(subscriber)
        .expect("Could not set up tracing subscriber");
}

fn bootstrap_server(host: &str, port: u16) {
    let host = host.to_owned();
    tokio::spawn(async move {
        let listener = TcpListener::bind((host, port)).await.expect("Could not start TCP server");
        while let Ok((stream, client_addr)) = listener.accept().await {
            info!("Incoming connection from {}", client_addr);
            // TODO
        }
    });
}

fn run_gui(host: &str, port: u16) {
    let host = if host == "0.0.0.0" {
        local_ip().expect("No local IP found").to_string()
    } else {
        host.to_owned()
    };

    let state = AppState::new(host, port);
    let window = WindowDesc::new(app_widget())
        .title("Robo")
        .window_size((640., 480.));

    AppLauncher::with_window(window)
        .launch(state)
        .expect("Could not launch GUI");
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
}

#[tokio::main]
async fn main() {
    let args = Args::parse();
    bootstrap_tracing();
    bootstrap_server(&args.host, args.port);
    run_gui(&args.host, args.port);
}
