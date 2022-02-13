#![feature(bool_to_option)]
use std::{process::{Command, ExitStatus}, os::unix::prelude::ExitStatusExt};
use rocket::{http::Status, form::{Form, self}};
#[macro_use] extern crate rocket;

#[derive(FromForm)]
struct Message<'a> {
    to_group: bool,

    #[field(validate = validate_sender_recipient())]
    recipient: &'a str,
    #[field(validate = validate_sender_recipient())]
    sender: &'a str,

    text: &'a str,

    #[field(validate = eq("TFu27M4a7lKXq33FHBihepP3XgSZTi7maTBARVxr"))]
    #[field(name = "key")]
    _key: &'a str
}

fn validate_sender_recipient<'v>(value: &str) -> form::Result<'v, ()> {
    if value.chars().all(|c| c.is_ascii_alphanumeric() || c == '+' || c == '/' || c == '=') {
        Ok(())
    } else {
        Err(form::Error::validation("Bad characters in recipient"))?
    }
}

#[get("/")]
fn index() -> &'static str {
    "Welcome to the Parlour Signal Bot!\nhttps://parlour.dev"
}

#[post("/notify", data = "<message>")]
fn send(message: Form<Message<'_>>) -> Result<&'_ str, Status> {
    let args = if message.to_group {
        vec!["-u", message.sender, "send", "-m", message.text, "-g", message.recipient]
    } else {
        vec!["-u", message.sender, "send", "-m", message.text, message.recipient]
    };

    let success = Command::new("/bin/signal-cli")
        .args(&args)
        .spawn()
        .map(|mut status| {
            status.wait().unwrap_or_else(|err| {
                eprintln!("Failed to wait for signal-cli: {}", err);
                ExitStatus::from_raw(1)
            }).success()
        })
        .unwrap_or_else(|err| {
            eprintln!("Failed to spawn signal-cli: {}", err);
            false
        });

    match success {
        true => Ok("Message sent"),
        false => Err(Status::InternalServerError)
    }
}


#[launch]
fn rocket() -> _ {
    rocket::build() 
    .mount("/", routes![index, send])
}