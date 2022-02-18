use crate::signal_socket::{self, SignalRPCCommand};
use std::error::Error;

pub mod github;

pub trait WebhookPayload {
    fn to_string(&self) -> String;
    fn notify_user(&self, sender: &str, recipient: &str) -> Result<(), Box<dyn Error>> {
        signal_socket::send_command(SignalRPCCommand::send_user(
            sender,
            recipient,
            self.to_string().as_str(),
        ))
        .map(|_| ())
    }
}
