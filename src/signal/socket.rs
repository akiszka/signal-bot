use rocket::serde::json::serde_json;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::error::Error;
use std::sync::Arc;
use std::sync::Mutex;
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

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct RPCResponse {
    jsonrpc: String,
    pub id: String,
    result: RPCResult,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct RPCResult {
    pub timestamp: u64,
    pub results: Vec<RPCResultInternal>,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct RPCResultInternal {
    pub r#type: String,
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

#[derive(Clone)]
pub struct Connection {
    request_tx: mpsc::Sender<(RPCCommand, oneshot::Sender<RPCResponse>)>,
}

impl Connection {
    fn init_socket() -> std::io::Result<UnixStream> {
        let stream = std::os::unix::net::UnixStream::connect("./signal.sock")?;

        stream.set_nonblocking(true)?;
        stream.set_read_timeout(Some(std::time::Duration::from_secs(5)))?;
        stream.set_write_timeout(Some(std::time::Duration::from_secs(5)))?;

        UnixStream::from_std(stream)
    }

    pub fn new() -> std::io::Result<Self> {
        // These sockets are used to communicate with Signal
        let (read_socket, mut write_socket) = Connection::init_socket()?.into_split();

        // You can send and recieve commands through these channels
        let (request_tx, mut request_rx) =
            mpsc::channel::<(RPCCommand, oneshot::Sender<RPCResponse>)>(100);

        // This maps request ids to channels awaiting responses from Signal
        let responses = Arc::new(Mutex::new(HashMap::new()));

        // This sends commands to Signal
        let sender_responses = responses.clone();
        tokio::spawn(async move {
            while let Some((mut cmd, response)) = request_rx.recv().await {
                // FIXME: handle errors
                let id = Uuid::new_v4().as_hyphenated().to_string();

                let command = serde_json::to_string(cmd.with_id(id.clone())).unwrap();
                let command = command.as_str();

                write_socket.write_all(command.as_bytes()).await.unwrap();
                write_socket.write_all(b"\n").await.unwrap();
                write_socket.flush().await.unwrap();

                let mut responses = sender_responses.lock().unwrap();
                responses.insert(id, response);
            }
        });

        // This reads responses from Signal
        let reader_responses = responses.clone();
        tokio::spawn(async move {
            let mut reader = BufReader::new(read_socket);

            let mut response_raw = String::new();
            while reader.read_line(&mut response_raw).await.is_ok() {
                if response_raw.trim() == "" {
                    continue;
                }

                let mut responses = reader_responses.lock().unwrap();
                let response: Option<RPCResponse> = serde_json::from_str(&response_raw).ok();

                if let Some(response) = response {
                    if let Some(response_channel) = responses.remove(&response.id) {
                        response_channel.send(response).unwrap();
                    }
                }

                response_raw.clear();
            }
        });

        Ok(Connection { request_tx })
    }

    pub async fn send_command(&self, command: RPCCommand) -> Result<RPCResponse, Box<dyn Error>> {
        let (response_tx, response_rx) = oneshot::channel();

        self.request_tx.send((command, response_tx)).await?;
        let response = response_rx.await?;

        Ok(response)
    }
}
