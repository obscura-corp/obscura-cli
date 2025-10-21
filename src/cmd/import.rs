use crate::cmd::common::load_vault;
use crate::util::errors::{ObscuraError, ObscuraResult};
use crate::vault::file::{encrypt_and_save_vault, vault_exists};
use crate::vault::manager::{VaultManager, VaultType};
use clap::Args;
use std::fs;

#[derive(Args)]
pub struct ImportArgs {
    #[arg(help = "Name of the .env file to import (e.g., '.env', '.env.local', 'production.env')")]
    pub env_file: String,

    #[arg(long, short = 'g', help = "Import to the global vault")]
    pub global: bool,

    #[arg(long, short = 'p', help = "Import to the project vault")]
    pub project: bool,
}

pub fn handle_import(args: ImportArgs) -> ObscuraResult<()> {
    // Default to project vault unless explicitly specified as global
    let force_global = args.global;
    let force_project = args.project || (!args.global && !args.project); // Default to project if neither specified
    
    let vault_info = VaultManager::resolve_vault(force_global, force_project)?;

    // Check if vault exists
    if !vault_exists(&vault_info.path) {
        match vault_info.vault_type {
            VaultType::Global => {
                return Err(ObscuraError::CustomError(
                    "Global vault not found. Please create a global vault first with 'obscura init --global'".to_string(),
                ));
            }
            VaultType::Project => {
                return Err(ObscuraError::CustomError(
                    "Project vault not found. Please create a project vault first with 'obscura init' or use --global to import to the global vault".to_string(),
                ));
            }
        }
    }

    // Find and read the .env file
    let env_content = find_and_read_env_file(&args.env_file)?;
    
    // Parse the .env content
    let env_vars = parse_env_content(&env_content)?;
    
    if env_vars.is_empty() {
        println!("No environment variables found in {}", args.env_file);
        return Ok(());
    }

    // Load the vault
    let (dek, mut aliases_data, vault_file) = load_vault(&vault_info.path)?;

    // Add each environment variable to the vault
    let mut added_count = 0;
    let mut skipped_count = 0;

    for (key, value) in env_vars {
        if aliases_data.aliases.contains_key(&key) {
            println!("Skipping '{}' - already exists in vault", key);
            skipped_count += 1;
            continue;
        }

        aliases_data.add_alias(key.clone(), value, &dek)?;
        added_count += 1;
        println!("Added '{}' to vault", key);
    }

    // Save the updated vault
    encrypt_and_save_vault(&vault_info.path, &vault_file, &aliases_data, &dek)?;

    let scope = match vault_info.vault_type {
        VaultType::Global => "global",
        VaultType::Project => "project",
    };

    println!("\nImport completed:");
    println!("  Added: {} variables", added_count);
    println!("  Skipped: {} variables (already exist)", skipped_count);
    println!("  Vault: {} vault", scope);

    Ok(())
}

fn find_and_read_env_file(env_file: &str) -> ObscuraResult<String> {
    let current_dir = std::env::current_dir().map_err(|_| ObscuraError::FilePermissionError)?;
    let env_path = current_dir.join(env_file);

    if !env_path.exists() {
        return Err(ObscuraError::CustomError(format!(
            "Environment file '{}' not found in current directory",
            env_file
        )));
    }

    if !env_path.is_file() {
        return Err(ObscuraError::CustomError(format!(
            "'{}' is not a file",
            env_file
        )));
    }

    fs::read_to_string(&env_path).map_err(|_| ObscuraError::CustomError(format!(
        "Failed to read '{}'",
        env_file
    )))
}

fn parse_env_content(content: &str) -> ObscuraResult<Vec<(String, String)>> {
    let mut env_vars = Vec::new();
    
    for line in content.lines() {
        let line = line.trim();
        
        // Skip empty lines and comments
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        
        // Parse KEY=VALUE format
        if let Some(equal_pos) = line.find('=') {
            let key = line[..equal_pos].trim().to_string();
            let value = line[equal_pos + 1..].trim().to_string();
            
            // Remove quotes if present
            let value = if (value.starts_with('"') && value.ends_with('"')) ||
                         (value.starts_with('\'') && value.ends_with('\'')) {
                value[1..value.len()-1].to_string()
            } else {
                value
            };
            
            if !key.is_empty() {
                env_vars.push((key, value));
            }
        }
    }
    
    Ok(env_vars)
}
