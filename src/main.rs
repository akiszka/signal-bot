#![feature(bool_to_option)]
#![feature(async_closure)]
mod jwt;
mod signal;
mod webhooks;

#[macro_use]
extern crate rocket;

use qrcode::render::svg;
use qrcode::QrCode;
use rocket::{
    form::{self, Form},
    http::{ContentType, Status},
    response::content,
    serde::json::Json,
    State,
};
use simple_logger::SimpleLogger;

use crate::signal::{socket::RPCCommand, Signal};

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
        false => Err(form::Errors::from(form::Error::validation(
            "Invalid characters in sender or recipient",
        ))),
    }
}

fn validate_sender<'v>(value: &str) -> form::Result<'v, ()> {
    match value.chars().all(|c| c.is_numeric() || c == '+') {
        true => Ok(()),
        false => Err(form::Errors::from(form::Error::validation(
            "Bad characters in sender",
        ))),
    }
}

#[get("/")]
fn index() -> &'static str {
    "Welcome to the Parlour Signal Bot!\nhttps://parlour.dev"
}

#[post("/rpc_raw", data = "<command>")]
async fn forward_raw_command(
    command: Json<RPCCommand>,
    signal: &State<Signal>,
) -> Result<String, Status> {
    signal
        .send_command(command.into_inner())
        .await
        .map_err(|err| {
            error!("{:?}", err);
            Status::InternalServerError
        })
}

#[post("/notify", data = "<message>")]
async fn notify(message: Form<Message<'_>>, signal: &State<Signal>) -> Result<String, Status> {
    let message = message.into_inner();
    let command = if message.to_group {
        RPCCommand::send_group(message.sender, message.recipient, message.text)
    } else {
        RPCCommand::send_user(message.sender, message.recipient, message.text)
    };

    signal.send_command(command).await.map_err(|err| {
        error!("{:?}", err);
        Status::InternalServerError
    })
}

#[get("/link")]
async fn link(signal: &State<Signal>) -> Result<String, Status> {
    signal.link().await.map_err(|err| {
        error!("{:?}", err);
        Status::InternalServerError
    })
}

#[get("/link/qr")]
async fn link_qr(signal: &State<Signal>) -> Result<content::Custom<Vec<u8>>, Status> {
    // reuse the link() function to get the joining link
    let uri = link(signal).await?;
    let uri = uri.trim();

    let code = QrCode::new(uri.as_bytes()).map_err(|err| {
        error!("{:?}", err);
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

#[post("/webhook/github?<sender>&<recipient>", data = "<payload>")]
async fn webhook_gh(
    payload: Json<webhooks::github::Payload<'_>>,
    sender: &str,
    recipient: &str,
    signal: &State<Signal>,
) -> Status {
    webhooks::notify_user(signal, payload.into_inner(), sender, recipient)
        .await
        .map_or(Status::InternalServerError, |_| Status::Ok)
}

#[rocket::main]
async fn main() {
    SimpleLogger::new().init().unwrap();

    let signal = signal::Signal::new().await.unwrap();

    rocket::build()
        .manage(signal.clone())
        .mount(
            "/",
            routes![
                index,
                forward_raw_command,
                notify,
                link,
                link_qr,
                webhook_gh
            ],
        )
        .launch()
        .await
        .unwrap_or_else(|err| {
            error!("Error in rocket: {}", err);
        });

    signal.stop().await.unwrap();
}
