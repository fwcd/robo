use std::{str, net::SocketAddr};

use anyhow::Result;
use async_tungstenite::{tokio::accept_async, tungstenite::Message};
use druid::ExtEventSink;
use futures::StreamExt;
use tokio::net::{TcpListener, TcpStream};
use tracing::{info, error, warn};

use crate::{state::{AppState, ClientInfo}, controller::Controller, security::Security, protocol::Action};

fn decode_action(raw: &[u8], security: &impl Security) -> Result<Action> {
    let raw = security.open(&raw)?;
    let raw_str = str::from_utf8(raw.as_ref())?;
    let action = serde_json::from_str(raw_str)?;
    Ok(action)
}

async fn run_client_loop(name: &str, stream: TcpStream, security: impl Security) -> Result<()> {
    let mut controller = Controller::new();
    let mut ws_stream = accept_async(stream).await?;
    while let Some(msg) = ws_stream.next().await {
        match msg? {
            Message::Binary(raw) => {
                match decode_action(&raw, &security) {
                    Ok(action) => {
                        info!("Client {} sent {:?}", name, action);
                        controller.perform(action)
                    },
                    Err(e) => warn!("Could not decode action: {}", e),
                }
            },
            Message::Close(_) => break,
            m => warn!("Unexpected message: {}", m),
        }
    }
    Ok(())
}

pub async fn handle_client(stream: TcpStream, addr: SocketAddr, security: impl Security, event_sink: Option<ExtEventSink>) {
    let name = addr.to_string();
    info!("Client {} connected!", name);

    if let Some(ref event_sink) = event_sink {
        let name = name.clone();
        event_sink.add_idle_callback(move |state: &mut AppState| {
            state.connected_clients.push_back(ClientInfo { name });
        });
    }

    if let Err(e) = run_client_loop(&name, stream, security).await {
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

pub async fn run_server(host: &str, port: u16, security: impl Security + Clone + Send + 'static, event_sink: Option<ExtEventSink>) {
    info!("Starting server on {}:{}", host, port);
    info!("Security: {} (key: {})", security.kind(), security.key().map(base64::encode).unwrap_or_else(|| "none".to_owned()));

    let listener = TcpListener::bind((host, port)).await.expect("Could not start TCP server");
    while let Ok((stream, client_addr)) = listener.accept().await {
        let event_sink = event_sink.clone();
        let security = security.clone();
        tokio::spawn(async move {
            handle_client(stream, client_addr, security, event_sink).await;
        });
    }
}
