use std::{error::Error, process::Stdio, time::Duration};

use rocket::futures::FutureExt;
use tokio::{
    io::{AsyncBufReadExt, BufReader},
    process::Command,
};

use super::Signal;

pub(super) async fn link(signal: &Signal) -> Result<String, Box<dyn Error>> {
    let mut output = Command::new("signal-cli")
        .args(&["link", "-n", "akiszka/signalbot"])
        .kill_on_drop(false)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::inherit())
        .spawn()?;

    let stdout = output.stdout.take().ok_or("Failed to redirect stdout")?;
    let mut reader = BufReader::new(stdout);
    let mut response = String::new();
    reader.read_line(&mut response).await?;

    debug!("[LINK] got join link from signal-cli: {}", &response);

    // This will either let Signal finish or kill it after 4 minutes
    let signal_to_restart = signal.clone();
    tokio::spawn(async move {
        tokio::select! {
            _ = output.wait() => {
                debug!("[LINK] Link successful. Restarting...");
                signal_to_restart.restart().await.unwrap_or_else(|e| {
                    panic!("[LINK] Failed to restart signal: {}", e);
                });
                debug!("[LINK] Restarted!");
            },
            _ = tokio::time::sleep(Duration::from_secs(60*4)).fuse() => {
                error!("[LINK] signal-link timeout");

                output.kill().await.unwrap_or_else(|e| {
                    error!("[LINK] Failed to kill signal-link: {}", e);
                });
            }
        }
    });

    Ok(response)
}
