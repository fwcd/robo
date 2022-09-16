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
pub struct SecurityInfo {
    /// The security kind.
    pub kind: String,
    /// The base64-encoded key.
    pub key_base64: String,
}

#[derive(Data, Clone, Debug, Serialize, Deserialize)]
pub struct ServerInfo {
    pub host: String,
    pub port: u16,
    pub security: SecurityInfo,
}

impl SecurityInfo {
    pub fn new(kind: String, key: &[u8]) -> Self {
        Self {
            kind,
            key_base64: base64::encode(key),
        }
    }
}

impl AppState {
    pub fn new(host: String, port: u16, security: SecurityInfo) -> Self {
        Self {
            server_info: ServerInfo {
                host,
                port,
                security,
            },
            connected_clients: im::Vector::new(),
        }
    }

    pub fn qr_code(&self) -> Result<QrCode> {
        let json = serde_json::to_string(&self.server_info)?;
        let qr = QrCode::encode_text(&format!("robo:{}", json), QrCodeEcc::Medium)?;
        Ok(qr)
    }
}
