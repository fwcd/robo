mod ui;
mod state;

use clap::Parser;
use druid::{AppLauncher, WindowDesc, ExtEventSink};
use local_ip_address::local_ip;
use state::AppState;
use tokio::net::TcpListener;
use tracing::info;
use ui::app_widget;

use crate::state::ClientInfo;

fn bootstrap_tracing() {
    let subscriber = tracing_subscriber::FmtSubscriber::new();
    tracing::subscriber::set_global_default(subscriber)
        .expect("Could not set up tracing subscriber");
}

fn bootstrap_server(host: &str, port: u16, event_sink: ExtEventSink) {
    let host = host.to_owned();
    tokio::spawn(async move {
        let listener = TcpListener::bind((host, port)).await.expect("Could not start TCP server");
        while let Ok((stream, client_addr)) = listener.accept().await {
            info!("Incoming connection from {}", client_addr);
            event_sink.add_idle_callback(move |state: &mut AppState| {
                state.connected_clients.push_back(ClientInfo {
                    name: client_addr.to_string(),
                });
            });
            // TODO
        }
    });
}

fn bootstrap_app() -> AppLauncher<AppState> {
    let window = WindowDesc::new(app_widget())
        .title("Robo")
        .window_size((640., 480.));

    AppLauncher::with_window(window)
}

fn launch_app(launcher: AppLauncher<AppState>, host: &str, port: u16) {
    let host = if host == "0.0.0.0" {
        local_ip().expect("No local IP found").to_string()
    } else {
        host.to_owned()
    };

    let state = AppState::new(host, port);
    launcher.launch(state)
        .expect("Could not launch app")
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
    bootstrap_tracing();

    let args = Args::parse();
    let launcher = bootstrap_app();

    bootstrap_server(&args.host, args.port, launcher.get_external_handle());
    launch_app(launcher, &args.host, args.port);
}
