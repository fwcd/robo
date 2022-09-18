use std::{str, net::SocketAddr, sync::Arc};

use anyhow::Result;
use async_tungstenite::{tokio::accept_async, tungstenite::Message};
use futures::StreamExt;
use tokio::{net::{TcpListener, TcpStream}, sync::mpsc};
use tracing::{info, error, warn};

use crate::{state::ClientInfo, security::Security, protocol::Action};

#[derive(Debug)]
pub enum MainThreadMessage {
    Perform(Action),
    DidConnect(ClientInfo),
    DidDisconnect(ClientInfo),
    DidExit,
}

#[derive(Clone)]
pub struct ServerContext {
    pub security: Arc<dyn Security + Send + Sync>,
    pub main_thread_tx: mpsc::Sender<MainThreadMessage>,
}

fn decode_action(raw: &[u8], security: &dyn Security) -> Result<Action> {
    let raw = security.open(&raw)?;
    let raw_str = str::from_utf8(raw.as_ref())?;
    let action = serde_json::from_str(raw_str)?;
    Ok(action)
}

async fn run_client_loop(name: &str, stream: TcpStream, ctx: ServerContext) -> Result<()> {
    let mut ws_stream = accept_async(stream).await?;
    while let Some(msg) = ws_stream.next().await {
        match msg? {
            Message::Binary(raw) => {
                let action = decode_action(&raw, &*ctx.security);
                match action {
                    Ok(action) => {
                        info!("Client {} sent {:?}", name, action);
                        ctx.main_thread_tx.send(MainThreadMessage::Perform(action)).await?;
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

pub async fn handle_client(stream: TcpStream, addr: SocketAddr, ctx: ServerContext) -> Result<()> {
    let info = ClientInfo { name: addr.to_string() };

    ctx.main_thread_tx.send(MainThreadMessage::DidConnect(info.clone())).await?;
    info!("Client {} connected!", info.name);

    {
        let ctx = ctx.clone();
        if let Err(e) = run_client_loop(&info.name, stream, ctx).await {
            error!("Error while running client loop: {}", e);
        };
    }

    ctx.main_thread_tx.send(MainThreadMessage::DidDisconnect(info.clone())).await?;
    info!("Client {} disconnected", info.name);

    Ok(())
}

pub async fn run_server(host: &str, port: u16, ctx: ServerContext) {
    info!("Starting server on {}:{}", host, port);
    info!("Security: {} (key: {})", ctx.security.kind(), ctx.security.key().map(base64::encode).unwrap_or_else(|| "none".to_owned()));

    let listener = TcpListener::bind((host, port)).await.expect("Could not start TCP server");
    while let Ok((stream, client_addr)) = listener.accept().await {
        let ctx = ctx.clone();
        tokio::spawn(async move {
            handle_client(stream, client_addr, ctx).await.expect("Error while handling client");
        });
    }

    ctx.main_thread_tx.send(MainThreadMessage::DidExit).await.expect("Could not send exit message to main thread");
}
