use std::error::Error;

use serde::{Serialize, Deserialize};

use crate::signal_socket::{self, SignalRPCCommand};

use super::WebhookPayload;

#[derive(Serialize, Deserialize, Debug)]
pub struct Payload {
    action: String,
    repository: Repository,
    sender: Sender,
}

#[derive(Serialize, Deserialize, Debug)]
struct Sender {
    login: String
}

#[derive(Serialize, Deserialize, Debug)]
struct Repository {
    name: String,
    full_name: String,
    url: String,
    html_url: String,
}

impl WebhookPayload for Payload {
    fn notify_user(&self, sender: &str, recipient: &str) -> Result<(), Box<dyn Error>> {
        signal_socket::send_command(SignalRPCCommand::send_user(
            sender,
            recipient,
            self.to_string().as_str(),
        )).map(|_| ())
    }

    fn to_string(&self) -> String {
        format!("GitHub: {} {}\nrepo: {}\n{}", self.sender.login, self.action, self.repository.full_name, self.repository.html_url)
    }
}