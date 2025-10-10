use crate::cmd::common::{load_aliases, load_vault};
use crate::util::errors::{ObscuraError, ObscuraResult};
use crate::util::io::{prompt_secret_value, prompt_yes_no};
use crate::vault::file::{encrypt_and_save_vault, vault_exists};
use crate::vault::manager::{VaultManager, VaultType};
use clap::Args;

#[derive(Args)]
pub struct AddArgs {
    #[arg(help = "Alias name for the secret")]
    pub alias: String,

    #[arg(long, short = 'g', help = "Operate on the global vault")]
    pub global: bool,

    #[arg(long, short = 'p', help = "Operate on the project vault")]
    pub project: bool,

    #[arg(
        long,
        help = "Copy the alias from the global vault into the project vault"
    )]
    pub from_global: bool,
}

pub fn handle_add(args: AddArgs) -> ObscuraResult<()> {
    if args.from_global {
        add_from_global(&args.alias)
    } else {
        add_new_secret(&args.alias, args.global, args.project)
    }
}

fn add_new_secret(alias: &str, force_global: bool, force_project: bool) -> ObscuraResult<()> {
    let vault_info = VaultManager::resolve_vault(force_global, force_project)?;

    if !vault_exists(&vault_info.path) {
        match vault_info.vault_type {
            VaultType::Global => {
                println!("Global vault not found. Creating it...");
                crate::cmd::init::init_global_vault()?;
            }
            VaultType::Project => return Err(ObscuraError::VaultNotFound),
        }
    }

    let value = prompt_secret_value(alias)?;
    let (dek, mut aliases_data, vault_file) = load_vault(&vault_info.path)?;

    if aliases_data.aliases.contains_key(alias) {
        if !prompt_yes_no(&format!("Alias '{}' already exists. Overwrite?", alias))? {
            println!("Cancelled");
            return Ok(());
        }
    }

    aliases_data.add_alias(alias.to_string(), value, &dek)?;
    encrypt_and_save_vault(&vault_info.path, &vault_file, &aliases_data, &dek)?;

    let scope = match vault_info.vault_type {
        VaultType::Global => "global",
        VaultType::Project => "project",
    };
    println!("Added '{}' to {} vault", alias, scope);

    Ok(())
}

fn add_from_global(alias: &str) -> ObscuraResult<()> {
    let project_vault_info = VaultManager::resolve_vault(false, true)?;
    let global_vault_info = VaultManager::resolve_vault(true, false)?;

    if !vault_exists(&project_vault_info.path) || !vault_exists(&global_vault_info.path) {
        return Err(ObscuraError::VaultNotFound);
    }

    let (global_dek, global_aliases) = load_aliases(&global_vault_info.path)?;
    let (project_dek, mut project_aliases, project_vault_file) =
        load_vault(&project_vault_info.path)?;

    let value = global_aliases
        .get_alias(alias, &global_dek)?
        .ok_or_else(|| ObscuraError::AliasNotFound(alias.to_string()))?;

    if project_aliases.aliases.contains_key(alias) {
        if !prompt_yes_no(&format!(
            "Alias '{}' already exists in project vault. Overwrite?",
            alias
        ))? {
            println!("Cancelled");
            return Ok(());
        }
    }

    project_aliases.add_alias(alias.to_string(), value, &project_dek)?;
    encrypt_and_save_vault(
        &project_vault_info.path,
        &project_vault_file,
        &project_aliases,
        &project_dek,
    )?;

    println!("Copied '{}' from global vault to project vault", alias);
    Ok(())
}
