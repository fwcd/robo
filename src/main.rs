mod ui;
mod controller;
mod protocol;
mod server;
mod state;

use clap::Parser;
use druid::{AppLauncher, WindowDesc, ExtEventSink};
use local_ip_address::local_ip;
use server::run_server;
use state::AppState;
use tokio::task::JoinHandle;
use ui::app_widget;

fn bootstrap_tracing() {
    let subscriber = tracing_subscriber::FmtSubscriber::new();
    tracing::subscriber::set_global_default(subscriber)
        .expect("Could not set up tracing subscriber");
}

fn bootstrap_server(host: &str, port: u16, event_sink: Option<ExtEventSink>) -> JoinHandle<()> {
    let host = host.to_owned();
    tokio::spawn(async move {
        run_server(&host, port, event_sink).await;
    })
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
    /// Runs the server without a GUI.
    #[clap(long)]
    headless: bool,
}

#[tokio::main]
async fn main() {
    bootstrap_tracing();

    let args = Args::parse();
    let launcher = if args.headless { None } else { Some(bootstrap_app()) };

    let server_handle = bootstrap_server(&args.host, args.port, launcher.as_ref().map(|l| l.get_external_handle()));

    if let Some(launcher) = launcher {
        // In non-headless mode the GUI's event loop blocks the main thread
        launch_app(launcher, &args.host, args.port);
    } else {
        // In headless mode we need to await the server
        server_handle.await.expect("Could not run server");
    }
}
