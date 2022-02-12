use rocket::serde::{Deserialize, json::Json};
#[macro_use] extern crate rocket;

#[derive(Deserialize)]
struct Message<'a> {
    text: &'a str
}

#[get("/")]
fn index() -> &'static str {
    "Hello, world!"
}

#[post("/notify", data = "<message>")]
fn send(message: Json<Message<'_>>) -> &'_ str {
    message.text
}


#[launch]
fn rocket() -> _ {
    rocket::build().mount("/", routes![index, send])
}