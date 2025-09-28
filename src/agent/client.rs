use crate::agent::protocol::{AgentRequest, AgentResponse};
use crate::crypto::aead::AeadKey;
use crate::util::errors::{ObscuraError, ObscuraResult};
use crate::util::paths::get_agent_socket_path;
use interprocess::local_socket::LocalSocketStream;
use std::io::{Read, Write};
use std::path::Path;

pub struct AgentClient;

impl AgentClient {
    pub fn is_running() -> bool {
        let socket_path = match get_agent_socket_path() {
            Ok(path) => path,
            Err(_) => return false,
        };
        if !socket_path.exists() {
            return false;
        }

        match Self::send_request(AgentRequest::Ping) {
            Ok(AgentResponse::Ok) => true,
            _ => false,
        }
    }

    pub fn get_dek(vault_path: &Path) -> ObscuraResult<Option<AeadKey>> {
        let vault_path_str = vault_path.to_string_lossy().to_string();
        let response = Self::send_request(AgentRequest::GetDek {
            vault_path: vault_path_str,
        })?;
        match response {
            AgentResponse::Dek { dek } => {
                if dek.len() != 32 {
                    return Err(ObscuraError::DecryptionFailed);
                }
                let mut dek_array = [0u8; 32];
                dek_array.copy_from_slice(&dek);
                Ok(Some(AeadKey::from_bytes(dek_array)))
            }
            AgentResponse::NotFound => Ok(None),
            AgentResponse::Error(_msg) => Err(ObscuraError::DecryptionFailed),
            _ => Err(ObscuraError::DecryptionFailed),
        }
    }

    pub fn store_dek(vault_path: &Path, dek: &AeadKey) -> ObscuraResult<()> {
        let vault_path_str = vault_path.to_string_lossy().to_string();
        let response = Self::send_request(AgentRequest::StoreDek {
            vault_path: vault_path_str,
            dek: dek.as_bytes().to_vec(),
        })?;

        match response {
            AgentResponse::Ok => Ok(()),
            AgentResponse::Error(_msg) => Err(ObscuraError::DecryptionFailed),
            _ => Err(ObscuraError::DecryptionFailed),
        }
    }

    pub fn shutdown() -> ObscuraResult<()> {
        let response = Self::send_request(AgentRequest::Shutdown)?;
        match response {
            AgentResponse::Ok => Ok(()),
            AgentResponse::Error(_msg) => Err(ObscuraError::DecryptionFailed),
            _ => Err(ObscuraError::DecryptionFailed),
        }
    }

    fn send_request(request: AgentRequest) -> ObscuraResult<AgentResponse> {
        let socket_path = get_agent_socket_path()?;
        let mut connection =
            LocalSocketStream::connect(socket_path).map_err(|_| ObscuraError::AgentNotRunning)?;
        let request_data =
            bincode::serialize(&request).map_err(|_| ObscuraError::DecryptionFailed)?;
        connection
            .write_all(&request_data)
            .map_err(|_| ObscuraError::DecryptionFailed)?;
        let mut response_data = Vec::new();
        connection
            .read_to_end(&mut response_data)
            .map_err(|_| ObscuraError::DecryptionFailed)?;
        let response: AgentResponse =
            bincode::deserialize(&response_data).map_err(|_| ObscuraError::DecryptionFailed)?;
        Ok(response)
    }
}
