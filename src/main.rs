mod ui;
mod controller;
mod protocol;
mod security;
mod server;
mod state;

use clap::Parser;
use druid::{AppLauncher, WindowDesc, ExtEventSink};
use local_ip_address::local_ip;
use security::{ChaChaPolySecurity, NoSecurity, Security};
use server::run_server;
use state::{AppState, SecurityInfo};
use tokio::task::JoinHandle;
use ui::app_widget;

fn bootstrap_tracing() {
    let subscriber = tracing_subscriber::FmtSubscriber::new();
    tracing::subscriber::set_global_default(subscriber)
        .expect("Could not set up tracing subscriber");
}

fn derive_security_info(security: &impl Security) -> SecurityInfo {
    SecurityInfo::new(security.kind().to_owned(), security.key().unwrap_or_default())
}

fn bootstrap_server(host: &str, port: u16, security: impl Security + Clone + Send + 'static, event_sink: Option<ExtEventSink>) -> (JoinHandle<()>, SecurityInfo) {
    let host = host.to_owned();
    let security_info = derive_security_info(&security);
    let handle = tokio::spawn(async move {
        run_server(&host, port, NoSecurity, event_sink).await;
    });
    (handle, security_info)
}

fn bootstrap_app() -> AppLauncher<AppState> {
    let window = WindowDesc::new(app_widget())
        .title("Robo")
        .window_size((640., 480.));

    AppLauncher::with_window(window)
}

fn launch_app(launcher: AppLauncher<AppState>, host: &str, port: u16, security_info: SecurityInfo) {
    let host = if host == "0.0.0.0" {
        local_ip().expect("No local IP found").to_string()
    } else {
        host.to_owned()
    };

    let state = AppState::new(host, port, security_info);
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
    /// Runs the server without encryption.
    #[clap(long)]
    insecure: bool,
    /// Runs the server without a GUI.
    #[clap(long)]
    headless: bool,
}

#[tokio::main]
async fn main() {
    bootstrap_tracing();

    let args = Args::parse();
    let launcher = if args.headless { None } else { Some(bootstrap_app()) };

    let event_sink = launcher.as_ref().map(|l| l.get_external_handle());
    let (server_handle, security_info) = if args.insecure {
        let security = NoSecurity;
        bootstrap_server(&args.host, args.port, security, event_sink)
    } else {
        let security = ChaChaPolySecurity::new().expect("Could not set up security");
        bootstrap_server(&args.host, args.port, security, event_sink)
    };
    
    if let Some(launcher) = launcher {
        // In non-headless mode the GUI's event loop blocks the main thread
        launch_app(launcher, &args.host, args.port, security_info);
    } else {
        // In headless mode we need to await the server
        server_handle.await.expect("Could not run server");
    }
}
