use crate::cmd::common::load_aliases;
use crate::util::errors::{ObscuraError, ObscuraResult};
use crate::vault::file::vault_exists;
use crate::vault::manager::VaultManager;
use clap::Args;
use std::fs;
use std::path::Path;

#[derive(Args)]
pub struct ExportDotenvArgs {
    #[arg(long, short = 'g', help = "Export from the global vault")]
    pub global: bool,

    #[arg(long, short = 'p', help = "Export from the project vault")]
    pub project: bool,

    #[arg(long, help = "Write output to this file path")]
    pub output: Option<String>,

    #[arg(long, help = "Allow overwriting the output file")]
    pub overwrite: bool,
}

pub fn handle_export_dotenv(args: ExportDotenvArgs) -> ObscuraResult<()> {
    let vault_info = VaultManager::resolve_vault(args.global, args.project)?;

    if !vault_exists(&vault_info.path) {
        return Err(ObscuraError::VaultNotFound);
    }

    let (dek, aliases_data) = load_aliases(&vault_info.path)?;
    let aliases = aliases_data.list_aliases();

    let mut dotenv_content = String::new();
    for alias in &aliases {
        if let Some(value) = aliases_data.get_alias(alias, &dek)? {
            dotenv_content.push_str(&format!("{}={}\n", alias, value));
        }
    }

    if let Some(output_path) = args.output {
        let path = Path::new(&output_path);

        if path.exists() && !args.overwrite {
            return Err(ObscuraError::FileExists(output_path));
        }

        fs::write(path, &dotenv_content).map_err(|_| ObscuraError::FilePermissionError)?;
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mut perms = fs::metadata(path)?.permissions();
            perms.set_mode(0o600);
            fs::set_permissions(path, perms)?;
        }

        println!("Exported {} aliases to {}", aliases.len(), output_path);
    } else {
        print!("{}", dotenv_content);
    }

    Ok(())
}
