use anyhow::Result;
use clap::{Parser, Subcommand};

mod cmd;
mod crypto;
mod session;
mod util;
mod vault;

use cmd::*;

#[derive(Parser)]
#[command(
    name = "obscura",
    about = "Passphrase-secured local API key vault",
    version
)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    #[command(about = "Initialize a vault (project by default, global with --global)")]
    Init(init::InitArgs),

    #[command(about = "Add a secret to the vault")]
    Add(add::AddArgs),

    #[command(about = "Retrieve a secret from the vault")]
    Get(get::GetArgs),

    #[command(about = "List secrets in the vault")]
    List(list::ListArgs),

    #[command(about = "Remove a secret from the vault")]
    Remove(remove::RemoveArgs),

    #[command(about = "Delete an entire vault")]
    Delete(delete::DeleteArgs),

    #[command(about = "Rotate a secret in the vault")]
    Rotate(rotate::RotateArgs),

    #[command(about = "Export secrets as dotenv content")]
    Export {
        #[arg(long, help = "Export in dotenv format")]
        dotenv: bool,
        #[arg(long, help = "Use the global vault")]
        global: bool,
        #[arg(long, help = "Use the project vault")]
        project: bool,
        #[arg(long, help = "Destination file path")]
        output: Option<String>,
        #[arg(long, help = "Allow overwriting the destination file")]
        overwrite: bool,
    },

    #[command(about = "Run a command with secrets injected as environment variables")]
    Run(run::RunArgs),

    #[command(about = "Clear cached vault keys")]
    Lock(lock::LockArgs),

    #[command(about = "Cache vault keys for a limited time")]
    Unlock(unlock::UnlockArgs),
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Init(args) => handle_init(args)?,
        Commands::Add(args) => handle_add(args)?,
        Commands::Get(args) => handle_get(args)?,
        Commands::List(args) => handle_list(args)?,
        Commands::Remove(args) => handle_remove(args)?,
        Commands::Delete(args) => handle_delete(args)?,
        Commands::Rotate(args) => handle_rotate(args)?,
        Commands::Export {
            dotenv,
            global,
            project,
            output,
            overwrite,
        } => {
            if dotenv {
                handle_export_dotenv(export_dotenv::ExportDotenvArgs {
                    global,
                    project,
                    output,
                    overwrite,
                })?;
            } else {
                eprintln!("Only --dotenv export is currently supported");
                std::process::exit(1);
            }
        }
        Commands::Run(args) => handle_run(args)?,
        Commands::Lock(args) => handle_lock(args)?,
        Commands::Unlock(args) => handle_unlock(args)?,
    }

    Ok(())
}
