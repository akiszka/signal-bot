use std::error::Error;

pub mod github;

pub trait WebhookPayload {
    fn notify_user(&self, sender: &str, recipient: &str) -> Result<(), Box<dyn Error>>;
    fn to_string(&self) -> String;
}