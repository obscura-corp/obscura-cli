use crate::crypto::aead::{decrypt_with_key, encrypt_with_key, AeadKey};
use crate::util::errors::{ObscuraError, ObscuraResult};
use crate::util::paths::{ensure_config_dir, get_config_dir};
use base64::{engine::general_purpose, Engine as _};
use chrono::{DateTime, Duration, Utc};
use fs2::FileExt;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::OnceLock;

#[derive(Default, Serialize, Deserialize)]
struct SessionData {
    vault_path: String,
    dek_b64: String,
    expires_at: String,
}

#[derive(Default, Serialize, Deserialize)]
struct SessionFile {
    entries: HashMap<String, SessionData>,
}

static SESSION_KEY_CACHE: OnceLock<AeadKey> = OnceLock::new();

pub struct SessionStore;

impl SessionStore {
    pub fn store_dek(vault_path: &Path, dek: &AeadKey, ttl_minutes: u64) -> ObscuraResult<()> {
        ensure_config_dir()?;

        let mut session = Self::load()?;
        let key = Self::normalize_path(vault_path)?;
        let expires_at = (Utc::now() + Duration::minutes(ttl_minutes as i64)).to_rfc3339();

        session.entries.insert(
            key,
            SessionData {
                vault_path: vault_path.to_string_lossy().to_string(),
                dek_b64: general_purpose::STANDARD.encode(dek.as_bytes()),
                expires_at,
            },
        );

        Self::save_encrypted(&session)
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
            Self::save_encrypted(&session)?;
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
                    Self::save_encrypted(&session)?;
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

        let encrypted_data = fs::read(&path).map_err(|_| ObscuraError::FilePermissionError)?;
        let session_key = Self::get_session_key()?;
        let decrypted_data = Self::decrypt_session_data(&encrypted_data, &session_key)?;

        let session: SessionFile = serde_json::from_slice(&decrypted_data)
            .map_err(|_| ObscuraError::InvalidVaultFormat)?;
        Ok(session)
    }

    fn save_encrypted(session: &SessionFile) -> ObscuraResult<()> {
        let path = Self::session_path()?;
        let session_key = Self::get_session_key()?;
        let session_data =
            serde_json::to_vec(session).map_err(|_| ObscuraError::EncryptionFailed)?;
        let encrypted_data = Self::encrypt_session_data(&session_data, &session_key)?;

        let temp_path = path.with_extension("tmp");
        {
            let mut file = OpenOptions::new()
                .create(true)
                .write(true)
                .truncate(true)
                .open(&temp_path)
                .map_err(|_| ObscuraError::FilePermissionError)?;

            file.lock_exclusive()
                .map_err(|_| ObscuraError::FilePermissionError)?;
            file.write_all(&encrypted_data)
                .map_err(|_| ObscuraError::FilePermissionError)?;
            file.sync_all()
                .map_err(|_| ObscuraError::FilePermissionError)?;
        }

        fs::rename(&temp_path, &path).map_err(|_| ObscuraError::FilePermissionError)?;
        Self::set_secure_permissions(&path)?;
        Ok(())
    }

    fn get_session_key() -> ObscuraResult<AeadKey> {
        Ok(SESSION_KEY_CACHE.get_or_init(|| Self::derive_session_key().unwrap()).clone())
    }

    fn derive_session_key() -> ObscuraResult<AeadKey> {
        let system_info = format!(
            "{}-{}-{}",
            std::env::var("USER").unwrap_or_else(|_| "unknown".to_string()),
            std::env::var("HOME").unwrap_or_else(|_| "unknown".to_string()),
            std::env::var("HOSTNAME").unwrap_or_else(|_| "unknown".to_string())
        );

        let mut key = [0u8; 32];
        argon2::Argon2::default()
            .hash_password_into(system_info.as_bytes(), b"obscura_session_salt", &mut key)
            .map_err(|_| ObscuraError::EncryptionFailed)?;

        Ok(AeadKey::from_bytes(key))
    }

    fn encrypt_session_data(data: &[u8], key: &AeadKey) -> ObscuraResult<Vec<u8>> {
        let aad = b"obscura_session";
        let aead_result = encrypt_with_key(data, key, aad)?;

        // Combine nonce and ciphertext
        let mut result = Vec::with_capacity(24 + aead_result.ciphertext.len());
        result.extend_from_slice(&aead_result.nonce);
        result.extend_from_slice(&aead_result.ciphertext);

        Ok(result)
    }

    fn decrypt_session_data(encrypted_data: &[u8], key: &AeadKey) -> ObscuraResult<Vec<u8>> {
        if encrypted_data.len() < 24 {
            return Err(ObscuraError::DecryptionFailed);
        }

        let nonce = &encrypted_data[0..24];
        let ciphertext = &encrypted_data[24..];

        let mut nonce_array = [0u8; 24];
        nonce_array.copy_from_slice(nonce);

        let aad = b"obscura_session";
        decrypt_with_key(ciphertext, key, &nonce_array, aad)
    }

    fn set_secure_permissions(path: &Path) -> ObscuraResult<()> {
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mut perms = fs::metadata(path)?.permissions();
            perms.set_mode(0o600);
            fs::set_permissions(path, perms)?;
        }
        #[cfg(windows)]
        {
            // On Windows, we rely on the file being created with restricted permissions
            // The file should only be accessible to the current user
        }
        Ok(())
    }

    fn session_path() -> ObscuraResult<PathBuf> {
        let config_dir = get_config_dir()?;
        Ok(config_dir.join("session.enc"))
    }

    fn normalize_path(path: &Path) -> ObscuraResult<String> {
        match path.canonicalize() {
            Ok(canonical) => Ok(canonical.to_string_lossy().to_string()),
            Err(_) => Ok(path.to_string_lossy().to_string()),
        }
    }
}
