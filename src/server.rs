use std::net::SocketAddr;

use anyhow::Result;
use async_tungstenite::{tokio::accept_async, tungstenite::Message};
use druid::ExtEventSink;
use futures::StreamExt;
use tokio::net::{TcpListener, TcpStream};
use tracing::{info, error, warn};

use crate::{state::{AppState, ClientInfo}, controller::Controller};

async fn run_client_loop(stream: TcpStream) -> Result<()> {
    let mut controller = Controller::new();
    let mut ws_stream = accept_async(stream).await?;
    while let Some(msg) = ws_stream.next().await {
        match msg? {
            Message::Text(txt) => {
                match serde_json::from_str(&txt) {
                    Ok(action) => controller.perform(action),
                    Err(e) => warn!("Could not parse action: {}", e),
                }
            },
            Message::Close(_) => break,
            m => warn!("Unexpected message: {}", m),
        }
    }
    Ok(())
}

pub async fn handle_client(stream: TcpStream, addr: SocketAddr, event_sink: ExtEventSink) {
    event_sink.add_idle_callback(move |state: &mut AppState| {
        state.connected_clients.push_back(ClientInfo {
            name: addr.to_string(),
        });
    });

    if let Err(e) = run_client_loop(stream).await {
        error!("Error while running client loop: {}", e);
    };

    event_sink.add_idle_callback(move |state: &mut AppState| {
        state.connected_clients.retain(|c| c.name != addr.to_string());
    });
}

pub async fn run_server(host: &str, port: u16, event_sink: ExtEventSink) {
    let listener = TcpListener::bind((host, port)).await.expect("Could not start TCP server");
    while let Ok((stream, client_addr)) = listener.accept().await {
        info!("Incoming connection from {}", client_addr);
        let event_sink = event_sink.clone();
        tokio::spawn(async move {
            handle_client(stream, client_addr, event_sink).await;
        });
    }
}
