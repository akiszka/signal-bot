use crate::signal_socket::{self, RPCCommand};
use std::error::Error;

pub mod github;

pub trait WebhookPayload {
    fn to_string(&self) -> String;
}

pub async fn notify_user(
    payload: impl WebhookPayload,
    sender: &str,
    recipient: &str,
) -> Result<(), Box<dyn Error>> {
    signal_socket::relay_command(RPCCommand::send_user(
        sender,
        recipient,
        payload.to_string().as_str(),
    ))
    .await
    .map(|_| ())
}
