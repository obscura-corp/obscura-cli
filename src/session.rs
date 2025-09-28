use crate::crypto::aead::AeadKey;
use crate::util::errors::{ObscuraError, ObscuraResult};
use crate::util::paths::{ensure_config_dir, get_config_dir};
use base64::{engine::general_purpose, Engine as _};
use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Default, Serialize, Deserialize)]
struct SessionFile {
    entries: HashMap<String, SessionEntry>,
}

#[derive(Serialize, Deserialize)]
struct SessionEntry {
    dek_b64: String,
    expires_at: String,
}

pub struct SessionStore;

impl SessionStore {
    pub fn store_dek(vault_path: &Path, dek: &AeadKey, ttl_minutes: u64) -> ObscuraResult<()> {
        ensure_config_dir()?;
        let mut session = Self::load()?;
        let key = Self::normalize_path(vault_path)?;
        let expires_at = (Utc::now() + Duration::minutes(ttl_minutes as i64)).to_rfc3339();
        session.entries.insert(
            key,
            SessionEntry {
                dek_b64: general_purpose::STANDARD.encode(dek.as_bytes()),
                expires_at,
            },
        );
        Self::save(&session)
    }

    pub fn fetch_dek(vault_path: &Path) -> ObscuraResult<Option<AeadKey>> {
        let mut session = Self::load()?;
        let key = Self::normalize_path(vault_path)?;
        let mut dirty = false;
        let now = Utc::now();
        session.entries.retain(|_, entry| {
            if let Ok(expires_at) = entry.expires_at.parse::<DateTime<Utc>>() {
                if expires_at > now {
                    return true;
                }
            }
            dirty = true;
            false
        });
        if dirty {
            Self::save(&session)?;
        }
        if let Some(entry) = session.entries.get(&key) {
            let bytes = general_purpose::STANDARD
                .decode(&entry.dek_b64)
                .map_err(|_| ObscuraError::DecryptionFailed)?;
            if bytes.len() != 32 {
                return Err(ObscuraError::DecryptionFailed);
            }
            let mut dek = [0u8; 32];
            dek.copy_from_slice(&bytes);
            return Ok(Some(AeadKey::from_bytes(dek)));
        }
        Ok(None)
    }

    pub fn clear(vault_path: Option<&Path>) -> ObscuraResult<()> {
        match vault_path {
            Some(path) => {
                let mut session = Self::load()?;
                let key = Self::normalize_path(path)?;
                if session.entries.remove(&key).is_some() {
                    Self::save(&session)?;
                }
                Ok(())
            }
            None => {
                let path = Self::session_path()?;
                if path.exists() {
                    fs::remove_file(path).map_err(|_| ObscuraError::FilePermissionError)?;
                }
                Ok(())
            }
        }
    }

    fn load() -> ObscuraResult<SessionFile> {
        let path = Self::session_path()?;
        if !path.exists() {
            return Ok(SessionFile::default());
        }
        let data = fs::read(&path).map_err(|_| ObscuraError::FilePermissionError)?;
        let session: SessionFile =
            serde_json::from_slice(&data).map_err(|_| ObscuraError::InvalidVaultFormat)?;
        Ok(session)
    }

    fn save(session: &SessionFile) -> ObscuraResult<()> {
        let path = Self::session_path()?;
        let data =
            serde_json::to_vec_pretty(session).map_err(|_| ObscuraError::EncryptionFailed)?;
        fs::write(&path, data).map_err(|_| ObscuraError::FilePermissionError)?;
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mut perms = fs::metadata(&path)?.permissions();
            perms.set_mode(0o600);
            fs::set_permissions(&path, perms)?;
        }
        Ok(())
    }

    fn session_path() -> ObscuraResult<PathBuf> {
        let config_dir = get_config_dir()?;
        Ok(config_dir.join("session.json"))
    }

    fn normalize_path(path: &Path) -> ObscuraResult<String> {
        match path.canonicalize() {
            Ok(canonical) => Ok(canonical.to_string_lossy().to_string()),
            Err(_) => Ok(path.to_string_lossy().to_string()),
        }
    }
}
