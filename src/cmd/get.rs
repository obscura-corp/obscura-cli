use crate::cmd::common::load_aliases;
use crate::util::errors::{ObscuraError, ObscuraResult};
use crate::vault::file::vault_exists;
use crate::vault::manager::VaultManager;
use clap::Args;

#[derive(Args)]
pub struct GetArgs {
    #[arg(help = "Alias name to retrieve")]
    pub alias: String,

    #[arg(long, help = "Read from the global vault")]
    pub global: bool,

    #[arg(long, help = "Read from the project vault")]
    pub project: bool,
}

pub fn handle_get(args: GetArgs) -> ObscuraResult<()> {
    let vault_info = VaultManager::resolve_vault(args.global, args.project)?;

    if !vault_exists(&vault_info.path) {
        return Err(ObscuraError::VaultNotFound);
    }

    let (dek, aliases_data) = load_aliases(&vault_info.path)?;

    match aliases_data.get_alias(&args.alias, &dek)? {
        Some(value) => {
            print!("{}", value);
            Ok(())
        }
        None => Err(ObscuraError::AliasNotFound(args.alias)),
    }
}
