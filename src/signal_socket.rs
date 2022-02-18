use rocket::serde::json::serde_json;
use serde::{Deserialize, Serialize};
use std::{
    error::Error,
    io::{BufRead, BufReader, Write},
    os::unix::net::UnixStream,
};

#[derive(Deserialize, Serialize, Debug)]
pub struct SignalRPCCommand {
    pub jsonrpc: String,
    pub id: String,
    pub method: String,
    pub params: SignalRPCParams,
}

#[derive(Deserialize, Serialize, Debug, Default)]
#[allow(non_snake_case)]
pub struct SignalRPCParams {
    pub account: Option<String>,
    pub recipient: Option<Vec<String>>,
    pub groupId: Option<String>,
    pub message: Option<String>,
}

impl SignalRPCCommand {
    pub fn new(method: String, params: SignalRPCParams) -> Self {
        SignalRPCCommand {
            jsonrpc: "2.0".to_string(),
            id: "my special mark".to_string(),
            method,
            params,
        }
    }

    pub fn send_user(account: &'_ str, phone: &'_ str, message: &'_ str) -> Self {
        SignalRPCCommand::new(
            "send".to_string(),
            SignalRPCParams {
                account: Some(account.to_string()),
                recipient: Some(vec![phone.to_string()]),
                message: Some(message.to_string()),
                ..Default::default()
            },
        )
    }

    pub fn send_group(account: &'_ str, group_id: &'_ str, message: &'_ str) -> Self {
        SignalRPCCommand::new(
            "send".to_string(),
            SignalRPCParams {
                account: Some(account.to_string()),
                message: Some(message.to_string()),
                groupId: Some(group_id.to_string()),
                ..Default::default()
            },
        )
    }
}

pub fn send_command(command: SignalRPCCommand) -> Result<String, Box<dyn Error>> {
    let command = serde_json::to_string(&command)?;
    let command = command.as_str();
    send_command_raw(command).map_err(|err| err.into())
}

pub fn send_command_raw(command: &str) -> std::io::Result<String> {
    let mut stream = UnixStream::connect("./signal.sock")?;
    let mut reader = BufReader::new(stream.try_clone()?);

    stream.set_read_timeout(Some(std::time::Duration::from_secs(5)))?;
    stream.set_write_timeout(Some(std::time::Duration::from_secs(5)))?;

    stream.write_all(command.as_bytes())?;
    stream.write_all(b"\n")?;
    stream.flush()?;
    let mut response = String::new();
    reader.read_line(&mut response)?;
    stream.shutdown(std::net::Shutdown::Both)?;
    Ok(response)
}
