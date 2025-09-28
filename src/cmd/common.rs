use crate::crypto::aead::AeadKey;
use crate::session::SessionStore;
use crate::util::errors::ObscuraResult;
use crate::util::io::{get_passphrase_from_env, prompt_passphrase};
use crate::vault::file::{decrypt_vault, decrypt_vault_with_dek, read_vault_file};
use crate::vault::model::{AliasesData, VaultFile};
use std::path::Path;

pub fn load_vault(vault_path: &Path) -> ObscuraResult<(AeadKey, AliasesData, VaultFile)> {
    let vault_file = read_vault_file(vault_path)?;

    if let Some(dek) = SessionStore::fetch_dek(vault_path)? {
        let aliases_data = decrypt_vault_with_dek(&vault_file, &dek)?;
        return Ok((dek, aliases_data, vault_file));
    }

    let passphrase = match get_passphrase_from_env() {
        Some(value) => value,
        None => prompt_passphrase()?,
    };

    let (dek, aliases_data) = decrypt_vault(&vault_file, &passphrase)?;

    Ok((dek, aliases_data, vault_file))
}

pub fn load_aliases(vault_path: &Path) -> ObscuraResult<(AeadKey, AliasesData)> {
    let (dek, aliases_data, _) = load_vault(vault_path)?;
    Ok((dek, aliases_data))
}
