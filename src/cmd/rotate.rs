use crate::cmd::common::load_vault;
use crate::util::errors::{ObscuraError, ObscuraResult};
use crate::util::io::prompt_secret_value;
use crate::vault::file::{encrypt_and_save_vault, vault_exists};
use crate::vault::manager::VaultManager;
use clap::Args;

#[derive(Args)]
pub struct RotateArgs {
    #[arg(help = "Alias name to rotate")]
    pub alias: String,

    #[arg(long, help = "Rotate in the global vault")]
    pub global: bool,

    #[arg(long, help = "Rotate in the project vault")]
    pub project: bool,
}

pub fn handle_rotate(args: RotateArgs) -> ObscuraResult<()> {
    let vault_info = VaultManager::resolve_vault(args.global, args.project)?;

    if !vault_exists(&vault_info.path) {
        return Err(ObscuraError::VaultNotFound);
    }

    let (dek, mut aliases_data, vault_file) = load_vault(&vault_info.path)?;

    if !aliases_data.aliases.contains_key(&args.alias) {
        return Err(ObscuraError::AliasNotFound(args.alias));
    }

    let new_value = prompt_secret_value(&args.alias)?;
    aliases_data.rotate_alias(&args.alias, new_value, &dek)?;
    encrypt_and_save_vault(&vault_info.path, &vault_file, &aliases_data, &dek)?;

    println!("Rotated alias '{}'", args.alias);
    Ok(())
}
