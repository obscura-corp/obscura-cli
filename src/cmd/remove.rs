use crate::agent::client::AgentClient;
use crate::cmd::common::load_vault;
use crate::util::errors::{ObscuraError, ObscuraResult};
use crate::util::io::prompt_yes_no;
use crate::vault::file::{encrypt_and_save_vault, vault_exists};
use crate::vault::manager::VaultManager;
use clap::Args;

#[derive(Args)]
pub struct RemoveArgs {
    #[arg(help = "Alias name to remove")]
    pub alias: String,

    #[arg(long, help = "Remove from the global vault")]
    pub global: bool,

    #[arg(long, help = "Remove from the project vault")]
    pub project: bool,

    #[arg(long, help = "Skip the confirmation prompt")]
    pub yes: bool,
}

pub fn handle_remove(args: RemoveArgs) -> ObscuraResult<()> {
    let vault_info = VaultManager::resolve_vault(args.global, args.project)?;

    if !vault_exists(&vault_info.path) {
        return Err(ObscuraError::VaultNotFound);
    }

    let (dek, mut aliases_data, vault_file) = load_vault(&vault_info.path)?;

    if !aliases_data.aliases.contains_key(&args.alias) {
        return Err(ObscuraError::AliasNotFound(args.alias));
    }

    if !args.yes {
        if !prompt_yes_no(&format!("Remove alias '{}'?", args.alias))? {
            println!("Cancelled");
            return Ok(());
        }
    }

    aliases_data.remove_alias(&args.alias);
    encrypt_and_save_vault(&vault_info.path, &vault_file, &aliases_data, &dek)?;

    if AgentClient::is_running() {
        let _ = AgentClient::store_dek(&vault_info.path, &dek);
    }

    println!("Removed alias '{}'", args.alias);
    Ok(())
}
