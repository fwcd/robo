mod ui;

use clap::Parser;
use druid::{AppLauncher, WindowDesc, widget::Label};
use tokio::net::TcpListener;
use tracing::info;

fn bootstrap_tracing() {
    let subscriber = tracing_subscriber::FmtSubscriber::new();
    tracing::subscriber::set_global_default(subscriber)
        .expect("Could not set up tracing subscriber");
}

fn bootstrap_server(host: String, port: u16) {
    tokio::spawn(async move {
        let listener = TcpListener::bind((host, port)).await.expect("Could not start TCP server");
        while let Ok((stream, client_addr)) = listener.accept().await {
            info!("Incoming connection from {}", client_addr);
            // TODO
        }
    });
}

fn run_gui() {
    let state = ();
    let window = WindowDesc::new(Label::new("Hello world!"))
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
    bootstrap_server(args.host, args.port);
    run_gui();
}
