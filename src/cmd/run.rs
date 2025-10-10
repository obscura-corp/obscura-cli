use crate::cmd::common::load_aliases;
use crate::util::errors::{ObscuraError, ObscuraResult};
use crate::vault::file::vault_exists;
use crate::vault::manager::VaultManager;
use clap::Args;
use std::collections::HashMap;
use std::path::Path;
use std::process::Command;

#[derive(Args)]
pub struct RunArgs {
    #[arg(long, short = 'g', help = "Use the global vault")]
    pub global: bool,

    #[arg(long, short = 'p', help = "Use the project vault")]
    pub project: bool,

    #[arg(help = "Command and arguments to execute (after --)")]
    pub command: Vec<String>,
}

pub fn handle_run(args: RunArgs) -> ObscuraResult<()> {
    if args.command.is_empty() {
        return Err(ObscuraError::InvalidVaultFormat);
    }

    let vault_info = VaultManager::resolve_vault(args.global, args.project)?;

    if !vault_exists(&vault_info.path) {
        return Err(ObscuraError::VaultNotFound);
    }

    let env_vars = get_secrets_as_env_vars(&vault_info.path)?;

    let (command, cmd_args) = split_command_args(args.command)?;

    let exit_code = execute_with_env_vars(&command, &cmd_args, env_vars)?;

    std::process::exit(exit_code);
}

fn get_secrets_as_env_vars(vault_path: &Path) -> ObscuraResult<HashMap<String, String>> {
    let (dek, aliases_data) = load_aliases(vault_path)?;

    let aliases = aliases_data.list_aliases();
    let mut env_vars = HashMap::new();

    for alias in &aliases {
        if let Some(value) = aliases_data.get_alias(alias, &dek)? {
            env_vars.insert(alias.clone(), value);
        }
    }

    Ok(env_vars)
}

fn split_command_args(command_args: Vec<String>) -> ObscuraResult<(String, Vec<String>)> {
    if command_args.is_empty() {
        return Err(ObscuraError::InvalidVaultFormat);
    }

    let command = command_args[0].clone();
    let args = command_args[1..].to_vec();

    Ok((command, args))
}

fn execute_with_env_vars(
    command: &str,
    args: &[String],
    env_vars: HashMap<String, String>,
) -> ObscuraResult<i32> {
    let mut cmd = Command::new(command);

    for arg in args {
        cmd.arg(arg);
    }

    for (key, value) in env_vars {
        cmd.env(key, value);
    }

    let mut child = cmd.spawn()?;
    let status = child.wait()?;
    Ok(status.code().unwrap_or(1))
}
