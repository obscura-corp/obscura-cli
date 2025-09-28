use crate::agent::client::AgentClient;
use crate::agent::server::AgentServer;
use crate::util::errors::{ObscuraError, ObscuraResult};
use clap::Args;
use std::thread;
use std::time::Duration;

#[derive(Args)]
pub struct UnlockArgs {
    #[arg(long, default_value = "30", help = "Agent timeout in minutes")]
    pub timeout: u64,
}

pub fn handle_unlock(args: UnlockArgs) -> ObscuraResult<()> {
    if args.timeout == 0 {
        return Err(ObscuraError::InvalidTimeout);
    }

    if AgentClient::is_running() {
        println!("Agent is already running");
        return Ok(());
    }

    let server = AgentServer::new(args.timeout);

    thread::spawn(move || {
        if let Err(e) = server.run() {
            eprintln!("Agent error: {}", e);
        }
    });

    thread::sleep(Duration::from_millis(100));

    if AgentClient::is_running() {
        println!("Agent started with {} minute timeout", args.timeout);
        Ok(())
    } else {
        Err(ObscuraError::AgentNotRunning)
    }
}
