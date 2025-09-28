use crate::crypto::aead::AeadKey;
use crate::crypto::kdf::{KdfParams, KdfResult};
use crate::util::errors::{ObscuraError, ObscuraResult};
use crate::vault::model::{AliasesData, EncryptedData, VaultBody, VaultFile};
use base64::{engine::general_purpose, Engine as _};
use serde_json;
use std::fs;
use std::path::Path;

pub fn create_vault_file(path: &Path, passphrase: &str) -> ObscuraResult<()> {
    let kdf_result = KdfResult::derive(passphrase)?;
    let dek = AeadKey::new();

    let aliases_json =
        serde_json::to_vec(&AliasesData::new()).map_err(|_| ObscuraError::EncryptionFailed)?;
    let body_aead = crate::crypto::aead::encrypt_with_key(&aliases_json, &dek, b"")?;
    let body = VaultBody {
        nonce_b64: general_purpose::STANDARD.encode(body_aead.nonce),
        ciphertext_b64: general_purpose::STANDARD.encode(body_aead.ciphertext),
    };

    let dek_wrapped = wrap_dek(&dek, &kdf_result.key)?;
    let vault_file = VaultFile::new(kdf_result.params.clone(), dek_wrapped, body);
    write_vault_atomically(path, &vault_file)
}

pub fn read_vault_file(path: &Path) -> ObscuraResult<VaultFile> {
    let data = fs::read(path).map_err(|_| ObscuraError::VaultNotFound)?;
    let vault_file = serde_json::from_slice(&data).map_err(|_| ObscuraError::InvalidVaultFormat)?;
    Ok(vault_file)
}

pub fn decrypt_vault(
    vault_file: &VaultFile,
    passphrase: &str,
) -> ObscuraResult<(AeadKey, AliasesData)> {
    let kdf_params: KdfParams = vault_file.kdf.clone().into();
    let kdf_result = KdfResult::derive_with_params(passphrase, &kdf_params)?;
    let dek = unwrap_dek(&vault_file.dek_wrapped, &kdf_result.key)?;

    let body_nonce = general_purpose::STANDARD
        .decode(&vault_file.body.nonce_b64)
        .map_err(|_| ObscuraError::DecryptionFailed)?;
    let body_ciphertext = general_purpose::STANDARD
        .decode(&vault_file.body.ciphertext_b64)
        .map_err(|_| ObscuraError::DecryptionFailed)?;

    if body_nonce.len() != 24 {
        return Err(ObscuraError::DecryptionFailed);
    }

    let mut body_nonce_array = [0u8; 24];
    body_nonce_array.copy_from_slice(&body_nonce);

    let aliases_json =
        crate::crypto::aead::decrypt_with_key(&body_ciphertext, &dek, &body_nonce_array, b"")?;
    let aliases_data: AliasesData =
        serde_json::from_slice(&aliases_json).map_err(|_| ObscuraError::DecryptionFailed)?;

    Ok((dek, aliases_data))
}

pub fn decrypt_vault_with_dek(vault_file: &VaultFile, dek: &AeadKey) -> ObscuraResult<AliasesData> {
    let body_nonce = general_purpose::STANDARD
        .decode(&vault_file.body.nonce_b64)
        .map_err(|_| ObscuraError::DecryptionFailed)?;
    let body_ciphertext = general_purpose::STANDARD
        .decode(&vault_file.body.ciphertext_b64)
        .map_err(|_| ObscuraError::DecryptionFailed)?;

    if body_nonce.len() != 24 {
        return Err(ObscuraError::DecryptionFailed);
    }

    let mut body_nonce_array = [0u8; 24];
    body_nonce_array.copy_from_slice(&body_nonce);

    let aliases_json =
        crate::crypto::aead::decrypt_with_key(&body_ciphertext, dek, &body_nonce_array, b"")?;
    let aliases_data: AliasesData =
        serde_json::from_slice(&aliases_json).map_err(|_| ObscuraError::DecryptionFailed)?;

    Ok(aliases_data)
}

pub fn encrypt_and_save_vault(
    path: &Path,
    vault_file: &VaultFile,
    aliases_data: &AliasesData,
    dek: &AeadKey,
) -> ObscuraResult<()> {
    let aliases_json =
        serde_json::to_vec(aliases_data).map_err(|_| ObscuraError::EncryptionFailed)?;
    let body_aead = crate::crypto::aead::encrypt_with_key(&aliases_json, dek, b"")?;
    let updated_vault = VaultFile {
        body: VaultBody {
            nonce_b64: general_purpose::STANDARD.encode(body_aead.nonce),
            ciphertext_b64: general_purpose::STANDARD.encode(body_aead.ciphertext),
        },
        ..vault_file.clone()
    };

    write_vault_atomically(path, &updated_vault)
}

fn wrap_dek(dek: &AeadKey, kek: &[u8; 32]) -> ObscuraResult<EncryptedData> {
    let kek_key = AeadKey::from_bytes(*kek);
    let aead_result = crate::crypto::aead::encrypt_with_key(dek.as_bytes(), &kek_key, b"")?;
    Ok(EncryptedData {
        nonce_b64: general_purpose::STANDARD.encode(aead_result.nonce),
        ciphertext_b64: general_purpose::STANDARD.encode(aead_result.ciphertext),
    })
}

fn unwrap_dek(dek_wrapped: &EncryptedData, kek: &[u8; 32]) -> ObscuraResult<AeadKey> {
    let kek_key = AeadKey::from_bytes(*kek);
    let nonce = general_purpose::STANDARD
        .decode(&dek_wrapped.nonce_b64)
        .map_err(|_| ObscuraError::DecryptionFailed)?;
    let ciphertext = general_purpose::STANDARD
        .decode(&dek_wrapped.ciphertext_b64)
        .map_err(|_| ObscuraError::DecryptionFailed)?;
    if nonce.len() != 24 {
        return Err(ObscuraError::DecryptionFailed);
    }
    let mut nonce_array = [0u8; 24];
    nonce_array.copy_from_slice(&nonce);
    let dek_bytes =
        crate::crypto::aead::decrypt_with_key(&ciphertext, &kek_key, &nonce_array, b"")?;
    if dek_bytes.len() != 32 {
        return Err(ObscuraError::DecryptionFailed);
    }
    let mut dek_array = [0u8; 32];
    dek_array.copy_from_slice(&dek_bytes);
    Ok(AeadKey::from_bytes(dek_array))
}

fn write_vault_atomically(path: &Path, vault_file: &VaultFile) -> ObscuraResult<()> {
    let temp_path = path.with_extension("tmp");
    let data = serde_json::to_vec_pretty(vault_file).map_err(|_| ObscuraError::EncryptionFailed)?;
    fs::write(&temp_path, &data).map_err(|_| ObscuraError::FilePermissionError)?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = fs::metadata(&temp_path)?.permissions();
        perms.set_mode(0o600);
        fs::set_permissions(&temp_path, perms)?;
    }
    fs::rename(&temp_path, path).map_err(|_| ObscuraError::FilePermissionError)?;
    Ok(())
}

pub fn vault_exists(path: &Path) -> bool {
    path.exists()
}
