use std::{error::Error, process::Stdio, time::Duration};

use rocket::futures::FutureExt;
use tokio::{
    io::{AsyncBufReadExt, BufReader},
    process::Command,
};

use super::Signal;

pub async fn link(signal: Signal) -> Result<String, Box<dyn Error>> {
    let mut output = Command::new("signal-cli")
        .args(&["link", "-n", "akiszka/signalbot"])
        .kill_on_drop(false)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::inherit())
        .spawn()?;

    let stdout = output.stdout.take().unwrap();
    let mut reader = BufReader::new(stdout);
    let mut response = String::new();
    reader.read_line(&mut response).await?;

    debug!("got join link from signal-cli: {}", &response);

    // This will either let Signal finish or kill it after 4 minutes
    #[allow(unused_must_use)]
    tokio::spawn(async move {
        tokio::select! {
            _ = output.wait() => {
                // Linking was successful. TODO: restart the Signal daemon.
                debug!("[LINK] Link successful. Restarting...");
                signal.restart().await;
                debug!("[LINK] Restarted!");
            },
            _ = tokio::time::sleep(Duration::from_secs(60*4)).fuse() => {
                error!("signal-link timeout");
                output.kill().await;
            }
        }
    });

    Ok(response)
}