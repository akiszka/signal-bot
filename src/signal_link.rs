use std::{error::Error, process::Stdio};

use tokio::{
    io::{AsyncBufReadExt, BufReader},
    process::Command,
};

pub async fn link() -> Result<String, Box<dyn Error>> {
    let output = Command::new("signal-cli")
        .args(&["link", "-n", "akiszka/signalbot"])
        .kill_on_drop(false)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::inherit())
        .spawn()?;

    let stdout = output.stdout.unwrap();
    let mut reader = BufReader::new(stdout);
    let mut response = String::new();
    reader.read_line(&mut response).await?;

    Ok(response)
}
