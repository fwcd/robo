mod ui;
mod controller;
mod protocol;
mod security;
mod server;
mod state;
mod utils;

use std::sync::{Arc, Mutex};

use clap::Parser;
use controller::Controller;
use druid::{AppLauncher, WindowDesc, ExtEventSink};
use local_ip_address::local_ip;
use security::{ChaChaPolySecurity, NoSecurity, Security};
use server::{run_server, ServerContext, MainThreadMessage};
use state::{AppState, SecurityInfo};
use tokio::{task::JoinHandle, sync::mpsc};
use ui::app_widget;
use utils::UnsafeSync;

fn bootstrap_tracing() {
    let subscriber = tracing_subscriber::FmtSubscriber::new();
    tracing::subscriber::set_global_default(subscriber)
        .expect("Could not set up tracing subscriber");
}

fn bootstrap_server(host: &str, port: u16, security: Arc<dyn Security + Send + Sync>, main_thread_tx: mpsc::Sender<MainThreadMessage>) -> JoinHandle<()> {
    let host = host.to_owned();
    let ctx = ServerContext { security, main_thread_tx };
    tokio::spawn(async move {
        run_server(&host, port, ctx).await;
    })
}

fn derive_security_info(security: &dyn Security) -> SecurityInfo {
    SecurityInfo::new(security.kind().to_owned(), security.key().unwrap_or_default())
}

async fn run_gui_main_msg_loop(mut rx: mpsc::Receiver<MainThreadMessage>, event_sink: ExtEventSink) {
    // We use `UnsafeSync` since the compiler cannot verify that we indeed always call the controller
    // from the same (main) thread due to our use of idle callbacks.
    let controller = Arc::new(Mutex::new(UnsafeSync::new(Controller::new())));

    while let Some(msg) = rx.recv().await {
        if let MainThreadMessage::DidExit = msg {
            break;
        };
        let controller = controller.clone();
        event_sink.add_idle_callback(move |state: &mut AppState| {
            match msg {
                MainThreadMessage::Perform(action) => controller.lock().unwrap().perform(action),
                MainThreadMessage::DidConnect(client) => state.connected_clients.push_back(client),
                MainThreadMessage::DidDisconnect(client) => {
                    // TODO: Identify clients exactly, we currently rely on uniqueness of names
                    // (which currently is given since we name clients after their IP + port)
                    state.connected_clients.retain(|c| c.name != client.name);
                },
                _ => {},
            }
        });
    }
}

fn run_gui(host: &str, port: u16, security_info: SecurityInfo, rx: mpsc::Receiver<MainThreadMessage>) {
    let host = if host == "0.0.0.0" {
        local_ip().expect("No local IP found").to_string()
    } else {
        host.to_owned()
    };

    let state = AppState::new(host, port, security_info);
    let window = WindowDesc::new(app_widget())
        .title("Robo")
        .window_size((640., 480.));
    let launcher = AppLauncher::with_window(window);
    let event_sink = launcher.get_external_handle();
    
    tokio::spawn(async move {
        run_gui_main_msg_loop(rx, event_sink).await;
    });

    launcher
        .launch(state)
        .expect("Could not launch app")
}

async fn run_headless_main_msg_loop(mut rx: mpsc::Receiver<MainThreadMessage>) {
    let mut controller = Controller::new();
    while let Some(msg) = rx.recv().await {
        match msg {
            MainThreadMessage::Perform(action) => controller.perform(action),
            MainThreadMessage::DidExit => break,
            _ => {},
        }
    }
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

    let Args { host, port, insecure, headless } = Args::parse();
    let (tx, rx) = mpsc::channel(4);
    let security: Arc<dyn Security + Send + Sync> = if insecure {
        Arc::new(NoSecurity)
    } else {
        Arc::new(ChaChaPolySecurity::new().expect("Could not set up security"))
    };

    let security_info = derive_security_info(&*security);
    bootstrap_server(&host, port, security, tx);
    
    if headless {
        // In headless mode we run a 'event loop' that handles messages from the server
        run_headless_main_msg_loop(rx).await;
    } else {
        // In non-headless mode the GUI's event loop blocks the main thread
        run_gui(&host, port, security_info, rx);
    }
}
