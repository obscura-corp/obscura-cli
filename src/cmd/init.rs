use crate::util::errors::{ObscuraError, ObscuraResult};
use crate::util::io::{get_passphrase_from_env, prompt_passphrase_confirmation};
use crate::vault::file::create_vault_file;
use crate::vault::manager::VaultManager;
use clap::Args;

#[derive(Args)]
pub struct InitArgs {
    #[arg(long, help = "Initialize a project vault for the current directory")]
    pub project: bool,
}

pub fn handle_init(args: InitArgs) -> ObscuraResult<()> {
    if args.project {
        init_project_vault()
    } else {
        init_global_vault()
    }
}

pub fn init_global_vault() -> ObscuraResult<()> {
    VaultManager::ensure_global_vault()?;
    let vault_info = VaultManager::resolve_vault(true, false)?;

    if vault_info.path.exists() {
        println!("Global vault already exists");
        return Ok(());
    }

    let passphrase = match get_passphrase_from_env() {
        Some(value) => value,
        None => prompt_passphrase_confirmation()?,
    };

    create_vault_file(&vault_info.path, &passphrase)?;

    println!("Global vault created successfully");
    println!("WARNING: This vault is for local development only.");
    println!("WARNING: Losing the passphrase makes the data unrecoverable.");

    Ok(())
}

fn init_project_vault() -> ObscuraResult<()> {
    let current_dir = std::env::current_dir().map_err(|_| ObscuraError::FilePermissionError)?;
    VaultManager::ensure_project_vault(&current_dir)?;

    let vault_info = VaultManager::resolve_vault(false, true)?;

    if vault_info.path.exists() {
        println!("Project vault already exists for this directory");
        return Ok(());
    }

    let passphrase = match get_passphrase_from_env() {
        Some(value) => value,
        None => prompt_passphrase_confirmation()?,
    };

    create_vault_file(&vault_info.path, &passphrase)?;

    println!("Project vault created for {}", current_dir.display());
    println!("WARNING: This vault is for local development only.");
    println!("WARNING: Losing the passphrase makes the data unrecoverable.");

    Ok(())
}
