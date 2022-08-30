use std::net::SocketAddr;

use anyhow::Result;
use async_tungstenite::{tokio::accept_async, tungstenite::Message};
use druid::ExtEventSink;
use futures::StreamExt;
use tokio::net::{TcpListener, TcpStream};
use tracing::{info, error, warn};

use crate::{state::{AppState, ClientInfo}, controller::Controller};

async fn run_client_loop(name: &str, stream: TcpStream) -> Result<()> {
    let mut controller = Controller::new();
    let mut ws_stream = accept_async(stream).await?;
    while let Some(msg) = ws_stream.next().await {
        match msg? {
            Message::Text(txt) => {
                match serde_json::from_str(&txt) {
                    Ok(action) => {
                        info!("Client {} sent {:?}", name, action);
                        controller.perform(action)
                    },
                    Err(e) => warn!("Could not parse action: {}", e),
                }
            },
            Message::Close(_) => break,
            m => warn!("Unexpected message: {}", m),
        }
    }
    Ok(())
}

pub async fn handle_client(stream: TcpStream, addr: SocketAddr, event_sink: Option<ExtEventSink>) {
    let name = addr.to_string();
    info!("Client {} connected!", name);

    if let Some(ref event_sink) = event_sink {
        let name = name.clone();
        event_sink.add_idle_callback(move |state: &mut AppState| {
            state.connected_clients.push_back(ClientInfo { name });
        });
    }

    if let Err(e) = run_client_loop(&name, stream).await {
        error!("Error while running client loop: {}", e);
    };

    if let Some(ref event_sink) = event_sink {
        let name = name.clone();
        event_sink.add_idle_callback(move |state: &mut AppState| {
            state.connected_clients.retain(|c| c.name != name);
        });
    }

    info!("Client {} disconnected", name);
}

pub async fn run_server(host: &str, port: u16, event_sink: Option<ExtEventSink>) {
    let listener = TcpListener::bind((host, port)).await.expect("Could not start TCP server");
    while let Ok((stream, client_addr)) = listener.accept().await {
        let event_sink = event_sink.clone();
        tokio::spawn(async move {
            handle_client(stream, client_addr, event_sink).await;
        });
    }
}
