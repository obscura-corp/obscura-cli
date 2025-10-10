use crate::session::SessionStore;
use crate::util::errors::{ObscuraError, ObscuraResult};
use crate::util::io::prompt_yes_no;
use crate::vault::file::vault_exists;
use crate::vault::manager::{VaultManager, VaultType};
use clap::Args;

#[derive(Args)]
pub struct DeleteArgs {
    #[arg(long, short = 'g', help = "Delete the global vault")]
    pub global: bool,

    #[arg(long, short = 'p', help = "Delete the project vault for the current directory")]
    pub project: bool,

    #[arg(long, help = "Skip the confirmation prompt")]
    pub yes: bool,
}

pub fn handle_delete(args: DeleteArgs) -> ObscuraResult<()> {
    let vault_info = VaultManager::resolve_vault(args.global, args.project)?;

    if !vault_exists(&vault_info.path) {
        return Err(ObscuraError::VaultNotFound);
    }

    if !args.yes {
        let message = match vault_info.vault_type {
            VaultType::Global => "Delete the global vault? This cannot be undone.",
            VaultType::Project => {
                "Delete the project vault for this directory? This cannot be undone."
            }
        };

        if !prompt_yes_no(message)? {
            println!("Cancelled");
            return Ok(());
        }
    }

    VaultManager::delete_vault(&vault_info)?;
    SessionStore::clear(Some(&vault_info.path))?;

    let scope = match vault_info.vault_type {
        VaultType::Global => "global",
        VaultType::Project => "project",
    };

    println!("Deleted {} vault", scope);
    Ok(())
}
