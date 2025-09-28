use crate::agent::protocol::{AgentRequest, AgentResponse};
use crate::crypto::aead::AeadKey;
use crate::util::errors::{ObscuraError, ObscuraResult};
use crate::util::paths::get_agent_socket_path;
use interprocess::local_socket::{LocalSocketListener, LocalSocketStream};
use std::collections::HashMap;
use std::io::{Read, Write};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};

struct StoredDek {
    dek: AeadKey,
    last_accessed: Instant,
}

pub struct AgentServer {
    deks: Arc<Mutex<HashMap<String, StoredDek>>>,
    timeout: Duration,
    shutdown: Arc<Mutex<bool>>,
}

impl AgentServer {
    pub fn new(timeout_minutes: u64) -> Self {
        Self {
            deks: Arc::new(Mutex::new(HashMap::new())),
            timeout: Duration::from_secs(timeout_minutes * 60),
            shutdown: Arc::new(Mutex::new(false)),
        }
    }

    pub fn run(&self) -> ObscuraResult<()> {
        let socket_path = get_agent_socket_path()?;
        let _ = std::fs::remove_file(&socket_path);
        let listener = LocalSocketListener::bind(socket_path)
            .map_err(|_| ObscuraError::FilePermissionError)?;
        println!("Agent started, listening on socket");
        let deks = Arc::clone(&self.deks);
        let timeout = self.timeout;
        let shutdown = Arc::clone(&self.shutdown);
        thread::spawn(move || loop {
            thread::sleep(Duration::from_secs(60));
            if *shutdown.lock().unwrap() {
                break;
            }
            let now = Instant::now();
            let mut deks_guard = deks.lock().unwrap();
            deks_guard
                .retain(|_, stored_dek| now.duration_since(stored_dek.last_accessed) < timeout);
        });
        for connection in listener.incoming() {
            if *self.shutdown.lock().unwrap() {
                break;
            }
            match connection {
                Ok(stream) => {
                    let deks = Arc::clone(&self.deks);
                    let shutdown = Arc::clone(&self.shutdown);
                    thread::spawn(move || {
                        if let Err(e) = Self::handle_connection(stream, deks, shutdown) {
                            eprintln!("Error handling connection: {}", e);
                        }
                    });
                }
                Err(e) => {
                    eprintln!("Connection error: {}", e);
                }
            }
        }
        let _ = std::fs::remove_file(get_agent_socket_path()?);
        println!("Agent stopped");
        Ok(())
    }

    fn handle_connection(
        mut stream: LocalSocketStream,
        deks: Arc<Mutex<HashMap<String, StoredDek>>>,
        shutdown: Arc<Mutex<bool>>,
    ) -> ObscuraResult<()> {
        let mut request_data = Vec::new();
        stream
            .read_to_end(&mut request_data)
            .map_err(|_| ObscuraError::DecryptionFailed)?;
        let request: AgentRequest =
            bincode::deserialize(&request_data).map_err(|_| ObscuraError::DecryptionFailed)?;
        let response = match request {
            AgentRequest::GetDek { vault_path } => {
                let mut deks_guard = deks.lock().unwrap();
                if let Some(stored_dek) = deks_guard.get_mut(&vault_path) {
                    stored_dek.last_accessed = Instant::now();
                    AgentResponse::Dek {
                        dek: stored_dek.dek.as_bytes().to_vec(),
                    }
                } else {
                    AgentResponse::NotFound
                }
            }
            AgentRequest::StoreDek { vault_path, dek } => {
                if dek.len() != 32 {
                    AgentResponse::Error("Invalid DEK length".to_string())
                } else {
                    let mut dek_array = [0u8; 32];
                    dek_array.copy_from_slice(&dek);
                    let aead_key = AeadKey::from_bytes(dek_array);
                    let mut deks_guard = deks.lock().unwrap();
                    deks_guard.insert(
                        vault_path,
                        StoredDek {
                            dek: aead_key,
                            last_accessed: Instant::now(),
                        },
                    );
                    AgentResponse::Ok
                }
            }
            AgentRequest::Ping => AgentResponse::Ok,
            AgentRequest::Shutdown => {
                *shutdown.lock().unwrap() = true;
                AgentResponse::Ok
            }
        };
        let response_data =
            bincode::serialize(&response).map_err(|_| ObscuraError::DecryptionFailed)?;
        stream
            .write_all(&response_data)
            .map_err(|_| ObscuraError::DecryptionFailed)?;
        Ok(())
    }
}
