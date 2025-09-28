use crate::crypto::kdf::KdfParams as CryptoKdfParams;
use crate::util::errors::{ObscuraError, ObscuraResult};
use base64::{engine::general_purpose, Engine as _};
use chrono::Utc;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VaultHeader {
    pub version: u32,
    pub created_at: String,
    pub kdf: KdfParams,
    pub dek_wrapped: EncryptedData,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KdfParams {
    pub alg: String,
    pub salt_b64: String,
    pub params: KdfParamsInner,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KdfParamsInner {
    pub mem_kib: u32,
    pub time: u32,
    pub lanes: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EncryptedData {
    pub nonce_b64: String,
    pub ciphertext_b64: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VaultBody {
    pub nonce_b64: String,
    pub ciphertext_b64: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VaultFile {
    pub version: u32,
    pub created_at: String,
    pub kdf: KdfParams,
    pub dek_wrapped: EncryptedData,
    pub body: VaultBody,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AliasesData {
    pub aliases: std::collections::HashMap<String, AliasData>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AliasData {
    pub value_enc: EncryptedData,
    pub created_at: String,
    pub rotated_at: Option<String>,
}

impl VaultFile {
    pub fn new(kdf_params: CryptoKdfParams, dek_wrapped: EncryptedData, body: VaultBody) -> Self {
        Self {
            version: 1,
            created_at: Utc::now().to_rfc3339(),
            kdf: KdfParams {
                alg: "argon2id".to_string(),
                salt_b64: general_purpose::STANDARD.encode(kdf_params.salt),
                params: KdfParamsInner {
                    mem_kib: kdf_params.memory_kib,
                    time: kdf_params.time,
                    lanes: kdf_params.lanes,
                },
            },
            dek_wrapped,
            body,
        }
    }
}

impl From<crate::crypto::kdf::KdfParams> for KdfParams {
    fn from(params: crate::crypto::kdf::KdfParams) -> Self {
        Self {
            alg: "argon2id".to_string(),
            salt_b64: general_purpose::STANDARD.encode(params.salt),
            params: KdfParamsInner {
                mem_kib: params.memory_kib,
                time: params.time,
                lanes: params.lanes,
            },
        }
    }
}

impl From<KdfParams> for crate::crypto::kdf::KdfParams {
    fn from(params: KdfParams) -> Self {
        let salt = general_purpose::STANDARD
            .decode(&params.salt_b64)
            .ok()
            .and_then(|bytes| bytes.try_into().ok())
            .unwrap_or_else(|| [0u8; 16]);

        Self {
            salt,
            memory_kib: params.params.mem_kib,
            time: params.params.time,
            lanes: params.params.lanes,
        }
    }
}

impl AliasesData {
    pub fn new() -> Self {
        Self {
            aliases: std::collections::HashMap::new(),
        }
    }

    pub fn add_alias(
        &mut self,
        alias: String,
        value: String,
        dek: &crate::crypto::aead::AeadKey,
    ) -> ObscuraResult<()> {
        let value_enc = encrypt_value(&value, dek)?;
        let alias_data = AliasData {
            value_enc,
            created_at: Utc::now().to_rfc3339(),
            rotated_at: None,
        };
        self.aliases.insert(alias, alias_data);
        Ok(())
    }

    pub fn get_alias(
        &self,
        alias: &str,
        dek: &crate::crypto::aead::AeadKey,
    ) -> ObscuraResult<Option<String>> {
        if let Some(alias_data) = self.aliases.get(alias) {
            let value = decrypt_value(&alias_data.value_enc, dek)?;
            Ok(Some(value))
        } else {
            Ok(None)
        }
    }

    pub fn remove_alias(&mut self, alias: &str) -> bool {
        self.aliases.remove(alias).is_some()
    }

    pub fn rotate_alias(
        &mut self,
        alias: &str,
        new_value: String,
        dek: &crate::crypto::aead::AeadKey,
    ) -> ObscuraResult<bool> {
        if let Some(alias_data) = self.aliases.get_mut(alias) {
            alias_data.value_enc = encrypt_value(&new_value, dek)?;
            alias_data.rotated_at = Some(Utc::now().to_rfc3339());
            Ok(true)
        } else {
            Ok(false)
        }
    }

    pub fn list_aliases(&self) -> Vec<String> {
        self.aliases.keys().cloned().collect()
    }
}

fn encrypt_value(value: &str, dek: &crate::crypto::aead::AeadKey) -> ObscuraResult<EncryptedData> {
    let aead_result = crate::crypto::aead::encrypt_with_key(value.as_bytes(), dek, b"")?;
    Ok(EncryptedData {
        nonce_b64: general_purpose::STANDARD.encode(aead_result.nonce),
        ciphertext_b64: general_purpose::STANDARD.encode(aead_result.ciphertext),
    })
}

fn decrypt_value(
    value_enc: &EncryptedData,
    dek: &crate::crypto::aead::AeadKey,
) -> ObscuraResult<String> {
    let nonce = general_purpose::STANDARD
        .decode(&value_enc.nonce_b64)
        .map_err(|_| ObscuraError::DecryptionFailed)?;
    let ciphertext = general_purpose::STANDARD
        .decode(&value_enc.ciphertext_b64)
        .map_err(|_| ObscuraError::DecryptionFailed)?;

    if nonce.len() != 24 {
        return Err(ObscuraError::DecryptionFailed);
    }

    let mut nonce_array = [0u8; 24];
    nonce_array.copy_from_slice(&nonce);

    let plaintext = crate::crypto::aead::decrypt_with_key(&ciphertext, dek, &nonce_array, b"")?;
    String::from_utf8(plaintext).map_err(|_| ObscuraError::DecryptionFailed)
}
