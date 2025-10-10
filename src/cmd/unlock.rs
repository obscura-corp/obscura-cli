use crate::session::SessionStore;
use crate::util::errors::{ObscuraError, ObscuraResult};
use crate::util::io::{get_passphrase_from_env, prompt_passphrase};
use crate::vault::file::{decrypt_vault, read_vault_file, vault_exists};
use crate::vault::manager::VaultManager;
use clap::Args;

#[derive(Args)]
pub struct UnlockArgs {
    #[arg(long, default_value_t = 60, help = "Cache timeout in minutes")]
    pub timeout: u64,

    #[arg(long, short = 'g', help = "Target the global vault")]
    pub global: bool,

    #[arg(long, short = 'p', help = "Target the project vault for the current directory")]
    pub project: bool,
}

pub fn handle_unlock(args: UnlockArgs) -> ObscuraResult<()> {
    if args.timeout == 0 {
        return Err(ObscuraError::InvalidTimeout);
    }

    let vault_info = VaultManager::resolve_vault(args.global, args.project)?;

    if !vault_exists(&vault_info.path) {
        return Err(ObscuraError::VaultNotFound);
    }

    let passphrase = match get_passphrase_from_env() {
        Some(value) => value,
        None => prompt_passphrase()?,
    };

    let vault_file = read_vault_file(&vault_info.path)?;
    let (dek, _) = decrypt_vault(&vault_file, &passphrase)?;

    SessionStore::store_dek(&vault_info.path, &dek, args.timeout)?;
    let scope = match vault_info.vault_type {
        crate::vault::manager::VaultType::Global => "global",
        crate::vault::manager::VaultType::Project => "project",
    };
    let unit = if args.timeout == 1 {
        "minute"
    } else {
        "minutes"
    };
    println!(
        "Cached vault key for {} {} (target: {})",
        args.timeout, unit, scope
    );

    Ok(())
}
