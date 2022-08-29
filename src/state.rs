use anyhow::Result;
use druid::{im, Data, Lens};
use qrcodegen::{QrCode, QrCodeEcc};
use serde::{Serialize, Deserialize};

#[derive(Data, Lens, Clone, Debug)]
pub struct AppState {
    pub server_info: ServerInfo,
    pub connected_clients: im::Vector<ClientInfo>,
}

#[derive(Data, Clone, Debug)]
pub struct ClientInfo {
    pub name: String,
}

#[derive(Data, Clone, Debug, Serialize, Deserialize)]
pub struct ServerInfo {
    pub host: String,
    pub port: u16,
    // TODO: Encryption key
}

impl AppState {
    pub fn new(host: String, port: u16) -> Self {
        Self {
            server_info: ServerInfo { host, port },
            connected_clients: im::Vector::new(),
        }
    }

    pub fn qr_code(&self) -> Result<QrCode> {
        let json = serde_json::to_string(&self.server_info)?;
        let qr = QrCode::encode_text(&json, QrCodeEcc::Medium)?;
        Ok(qr)
    }
}
