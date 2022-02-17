use serde::{Serialize, Deserialize};
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
    fn to_string(&self) -> String {
        format!("GitHub: {} {}\nrepo: {}\n{}", self.sender.login, self.action, self.repository.full_name, self.repository.html_url)
    }
}