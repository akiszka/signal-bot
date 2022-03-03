use std::error::Error;

use crate::signal::{socket::RPCCommand, Signal};

pub mod github;

pub trait WebhookPayload {
    fn to_string(&self) -> String;
}

pub async fn notify_user(
    signal: &Signal,
    payload: impl WebhookPayload,
    sender: &str,
    recipient: &str,
) -> Result<(), Box<dyn Error>> {
    signal
        .send_command(RPCCommand::send_user(
            sender,
            recipient,
            payload.to_string().as_str(),
        ))
        .await
        .map(|_| ())
}
