use crate::cmd::common::load_aliases;
use crate::util::errors::{ObscuraError, ObscuraResult};
use crate::vault::file::vault_exists;
use crate::vault::manager::{VaultManager, VaultType};
use clap::Args;
use serde_json::{json, to_string_pretty};

#[derive(Args)]
pub struct ListArgs {
    #[arg(long, short = 'g', help = "List entries from the global vault")]
    pub global: bool,

    #[arg(long, short = 'p', help = "List entries from the project vault")]
    pub project: bool,

    #[arg(long, help = "Render output as JSON")]
    pub json: bool,
}

pub fn handle_list(args: ListArgs) -> ObscuraResult<()> {
    let vault_info = VaultManager::resolve_vault(args.global, args.project)?;

    if !vault_exists(&vault_info.path) {
        return match vault_info.vault_type {
            VaultType::Global => {
                println!("Global vault not found. Creating it...");
                crate::cmd::init::init_global_vault()?;
                if args.json {
                    println!("{}", json!({ "aliases": [] }));
                }
                Ok(())
            }
            VaultType::Project => Err(ObscuraError::VaultNotFound),
        };
    }

    let (_, aliases_data) = load_aliases(&vault_info.path)?;
    let aliases = aliases_data.list_aliases();

    if args.json {
        println!("{}", to_string_pretty(&json!({ "aliases": aliases }))?);
    } else {
        for alias in aliases {
            println!("{}", alias);
        }
    }

    Ok(())
}
