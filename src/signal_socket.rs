use rocket::serde::json::serde_json;
use serde::{Deserialize, Serialize};
use std::error::Error;
use tokio::io::AsyncBufReadExt;
use tokio::io::AsyncWriteExt;
use tokio::io::BufReader;
use tokio::net::UnixStream;
use tokio::sync::mpsc;
use tokio::sync::oneshot;
use uuid::Uuid;

#[derive(Deserialize, Serialize, Debug)]
pub struct RPCCommand {
    pub jsonrpc: String,
    pub id: String,
    pub method: String,
    pub params: RPCParams,
}

#[derive(Deserialize, Serialize, Debug, Default)]
#[allow(non_snake_case)]
pub struct RPCParams {
    pub account: Option<String>,
    pub recipient: Option<Vec<String>>,
    pub groupId: Option<String>,
    pub message: Option<String>,
}

impl RPCCommand {
    pub fn new(method: String, params: RPCParams) -> Self {
        RPCCommand {
            jsonrpc: "2.0".to_string(),
            id: "my special mark".to_string(),
            method,
            params,
        }
    }

    pub fn with_id(&mut self, id: String) -> &mut Self {
        self.id = id;
        self
    }

    pub fn send_user(account: &'_ str, phone: &'_ str, message: &'_ str) -> Self {
        RPCCommand::new(
            "send".to_string(),
            RPCParams {
                account: Some(account.to_string()),
                recipient: Some(vec![phone.to_string()]),
                message: Some(message.to_string()),
                ..Default::default()
            },
        )
    }

    pub fn send_group(account: &'_ str, group_id: &'_ str, message: &'_ str) -> Self {
        RPCCommand::new(
            "send".to_string(),
            RPCParams {
                account: Some(account.to_string()),
                message: Some(message.to_string()),
                groupId: Some(group_id.to_string()),
                ..Default::default()
            },
        )
    }
}

#[derive(Debug, Clone)]
pub struct Connection {
    request_tx: mpsc::Sender<(RPCCommand, oneshot::Sender<String>)>,
}

impl Connection {
    fn init_socket() -> std::io::Result<UnixStream> {
        let stream = std::os::unix::net::UnixStream::connect("./signal.sock")?;

        stream.set_nonblocking(true)?;
        stream.set_read_timeout(Some(std::time::Duration::from_secs(5)))?;
        stream.set_write_timeout(Some(std::time::Duration::from_secs(5)))?;

        UnixStream::from_std(stream)
    }

    fn new() -> std::io::Result<Self> {
        let (read_stream, mut write_stream) = Connection::init_socket()?.into_split();

        let (request_tx, mut request_rx) =
            mpsc::channel::<(RPCCommand, oneshot::Sender<String>)>(100);

        tokio::spawn(async move {
            while let Some((mut cmd, response)) = request_rx.recv().await {
                // FIXME: handle errors
                let id = Uuid::new_v4().as_hyphenated().to_string();

                let command = serde_json::to_string(cmd.with_id(id.clone())).unwrap();
                let command = command.as_str();

                write_stream.write_all(command.as_bytes()).await.unwrap();
                write_stream.write_all(b"\n").await.unwrap();
                write_stream.flush().await.unwrap();

                response.send(id).unwrap();
            }
        });

        tokio::spawn(async move {
            let mut reader = BufReader::new(read_stream);

            let mut response = String::new();
            while reader.read_line(&mut response).await.is_ok() {
                if response.trim() == "" {
                    continue;
                }
                println!("Signal: {}", response);
                response.clear();
            }
        });

        Ok(Connection { request_tx })
    }

    pub async fn send_command(&self, command: RPCCommand) -> Result<String, Box<dyn Error>> {
        let (response_tx, response_rx) = oneshot::channel();

        self.request_tx.send((command, response_tx)).await?;
        let request_id = response_rx.await?;

        Ok(request_id)
    }

    pub async fn get_reply() -> Result<String, Box<dyn Error>> {
        Ok("test2".into())
    }
}

#[deprecated]
pub async fn relay_command(command: RPCCommand) -> Result<String, Box<dyn Error>> {
    let connection = Connection::new().unwrap();

    let request_id = connection.send_command(command).await?;

    Ok(request_id)
}
