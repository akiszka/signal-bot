#![feature(bool_to_option)]
#![feature(never_type)]
#![feature(box_into_inner)]
mod signal_daemon;
mod signal_link;
mod signal_socket;

#[macro_use]
extern crate rocket;

use std::sync::Arc;

use qrcode::render::svg;
use qrcode::QrCode;
use rocket::{
    form::{self, Form},
    http::{ContentType, Status},
    response::content,
    serde::json::Json,
    State,
};
use signal_daemon::DaemonManager;
use signal_socket::SignalRPCCommand;

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

#[get("/link")]
async fn link(daemon: &State<Arc<DaemonManager>>) -> Result<String, Status> {
    let daemon = (*daemon.inner()).clone();

    signal_link::link(daemon).await.map_err(|err| {
        println!("{:?}", err);
        Status::InternalServerError
    })
}

#[get("/link/qr")]
async fn link_qr(daemon: &State<Arc<DaemonManager>>) -> Result<content::Custom<Vec<u8>>, Status> {
    // reuse the link() function to get the joining link
    let uri = link(daemon).await?;
    let uri = uri.trim();

    let code = QrCode::new(uri.as_bytes()).map_err(|err| {
        println!("{:?}", err);
        Status::InternalServerError
    })?;
    let image = code
        .render()
        .min_dimensions(200, 200)
        .dark_color(svg::Color("#000000"))
        .light_color(svg::Color("#ffffff"))
        .build();

    Ok(content::Custom(ContentType::SVG, image.as_bytes().to_vec()))
}

#[rocket::main]
async fn main() {
    let daemon = Arc::new(signal_daemon::DaemonManager::new().await.unwrap());

    rocket::build()
        .manage(daemon.clone())
        .mount(
            "/",
            routes![index, forward_raw_command, notify, link, link_qr],
        )
        .launch()
        .await
        .unwrap_or_else(|err| {
            println!("Error in rocket: {}", err);
            ()
        });

    daemon.stop().await.unwrap();
}
