use std::error::Error;

use self::socket::RPCCommand;

mod daemon;
pub mod link;
pub mod socket; // for RPCCommand

#[derive(Clone)]
pub struct Signal {
    manager: daemon::DaemonManager,
    connection: socket::Connection,
}

impl Signal {
    pub async fn new() -> Result<Self, Box<dyn Error>> {
        let manager = daemon::DaemonManager::new().await?;
        let connection = socket::Connection::new()?;

        Ok(Self {
            manager,
            connection,
        })
    }

    pub async fn restart(&self) -> Result<(), Box<dyn Error>> {
        self.manager.restart().await
    }

    pub async fn stop(&self) -> Result<(), Box<dyn Error>> {
        self.manager.stop().await
    }

    pub async fn send_command(&self, command: RPCCommand) -> Result<String, Box<dyn Error>> {
        self.connection.send_command(command).await
    }
}
