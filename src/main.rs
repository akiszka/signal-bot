#![feature(bool_to_option)]
use std::process::Command;
use rocket::{http::Status, form::{Form, self}};
#[macro_use] extern crate rocket;

#[derive(FromForm)]
struct Message<'a> {
    to_group: bool,
    #[field(validate = validate_recipient())]
    recipient: &'a str,
    text: &'a str,
    #[field(validate = eq("TFu27M4a7lKXq33FHBihepP3XgSZTi7maTBARVxr"))]
    #[field(name = "key")]
    _key: &'a str
}

fn validate_recipient<'v>(value: &str) -> form::Result<'v, ()> {
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
        vec!["--dbus", "send", "-m", message.text, "-g", message.recipient]
    } else {
        vec!["--dbus", "send", "-m", message.text, message.recipient]
    };

    Command::new("signal-cli")
        .args(&args)
        .status()
        .map_or(Err(Status::InternalServerError), |status| {
            if status.success() {
                Ok("sent")
            } else {
                Err(Status::InternalServerError)
            }
        })
}


#[launch]
fn rocket() -> _ {
    rocket::build().mount("/", routes![index, send])
}