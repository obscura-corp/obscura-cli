use crate::session::SessionStore;
use crate::util::errors::ObscuraResult;
use clap::Args;

#[derive(Args)]
pub struct LockArgs {
    #[arg(long, short = 'g', help = "Target the global vault")]
    pub global: bool,

    #[arg(long, short = 'p', help = "Target the project vault for the current directory")]
    pub project: bool,
}

pub fn handle_lock(args: LockArgs) -> ObscuraResult<()> {
    if args.global || args.project {
        let vault_info =
            crate::vault::manager::VaultManager::resolve_vault(args.global, args.project)?;
        SessionStore::clear(Some(&vault_info.path))?;
    } else {
        SessionStore::clear(None)?;
    }

    println!("Cleared cached vault keys");
    Ok(())
}
