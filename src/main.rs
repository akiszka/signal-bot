#![feature(bool_to_option)]
mod signal_socket;
mod signal_link;
use rocket::{
    form::{self, Form},
    http::Status,
    serde::json::Json,
};
use signal_socket::SignalRPCCommand;
#[macro_use]
extern crate rocket;

#[derive(FromForm)]
struct Message<'a> {
    to_group: bool,

    #[field(validate = validate_recipient())]
    recipient: &'a str,
    #[field(validate = validate_sender())]
    sender: &'a str,

    text: &'a str,
}

fn validate_recipient<'v>(value: &str) -> form::Result<'v, ()> {
    match value
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || c == '+' || c == '/' || c == '=')
    {
        true => Ok(()),
        false => Err(form::Error::validation(
            "Invalid characters in sender or recipient",
        ))?,
    }
}

fn validate_sender<'v>(value: &str) -> form::Result<'v, ()> {
    match value.chars().all(|c| c.is_numeric() || c == '+') {
        true => Ok(()),
        false => Err(form::Error::validation("Bad characters in sender"))?,
    }
}

#[get("/")]
fn index() -> &'static str {
    "Welcome to the Parlour Signal Bot!\nhttps://parlour.dev"
}

#[post("/rpc_raw", data = "<command>")]
fn forward_raw_command(command: Json<SignalRPCCommand>) -> Result<String, Status> {
    signal_socket::send_command(command.into_inner()).map_err(|err| {
        println!("{:?}", err);
        Status::InternalServerError
    })
}

#[post("/notify", data = "<message>")]
fn notify(message: Form<Message<'_>>) -> Result<String, Status> {
    let message = message.into_inner();
    let command = if message.to_group {
        SignalRPCCommand::send_group(message.sender, message.recipient, message.text)
    } else {
        SignalRPCCommand::send_user(message.sender, message.recipient, message.text)
    };

    signal_socket::send_command(command).map_err(|err| {
        println!("{:?}", err);
        Status::InternalServerError
    })
}

#[post("/link")]
async fn link() -> Result<String, Status> {
    signal_link::link().await.map_err(|err| {
        println!("{:?}", err);
        Status::InternalServerError
    })
}

#[rocket::main]
async fn main() {
    rocket::build()
        .mount("/", routes![index, forward_raw_command, notify, link])
        .launch()
        .await
        .unwrap()
}
