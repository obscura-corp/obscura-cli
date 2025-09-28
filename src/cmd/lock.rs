use crate::agent::client::AgentClient;
use crate::util::errors::ObscuraResult;
use clap::Args;

#[derive(Args)]
pub struct LockArgs;

pub fn handle_lock(_args: LockArgs) -> ObscuraResult<()> {
    if AgentClient::is_running() {
        AgentClient::shutdown()?;
        println!("Agent stopped");
    } else {
        println!("Agent not running");
    }

    Ok(())
}
