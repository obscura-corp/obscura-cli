use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub enum AgentRequest {
    GetDek { vault_path: String },
    StoreDek { vault_path: String, dek: Vec<u8> },
    Ping,
    Shutdown,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum AgentResponse {
    Dek { dek: Vec<u8> },
    NotFound,
    Ok,
    Error(String),
}
