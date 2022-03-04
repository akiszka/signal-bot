use std::{error::Error, process::Stdio, time::Duration};

use rocket::futures::FutureExt;
use tokio::{
    io::{AsyncBufReadExt, BufReader},
    process::{ChildStdout, Command},
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
            _ = tokio::time::sleep(Duration::from_secs(60*4)).fuse() => {
                error!("[LINK] signal-link timeout");

                output.kill().await.unwrap_or_else(|e| {
                    error!("[LINK] Failed to kill signal-link: {}", e);
                });
            },
            _ = read_until_phone_number(reader) => {
                let output_ok = output.wait().await.map_err(|e| {
                    error!("[LINK] Failed to wait for signal-link: {}", e);
                })
                .and_then(|status| {
                    if !status.success() {
                        error!("[LINK] signal-link exited with non-zero status");
                        return Err(())
                    }

                    Ok(())
                }).is_ok();

                if output_ok {
                    trace!("[LINK] restarting signal-link");
                    signal_to_restart.restart().await.unwrap_or_else(|e| {
                        panic!("[LINK] Failed to restart signal: {}", e);
                    });
                    trace!("[LINK] restarted");
                }
            }
        }
    });

    Ok(response)
}

/// This is a helper function that reads from the stdout of signal-cli link until it links successfully.
async fn read_until_phone_number(
    mut stdout: BufReader<ChildStdout>,
) -> Result<String, std::io::Error> {
    let phone_number;
    let mut response = String::new();

    let joining_pattern = "Associated with: ";

    loop {
        response.clear();
        stdout.read_line(&mut response).await?;

        if !response.is_empty() {
            trace!("[LINK] Signal: {}", &response.trim_end());

            if let Some(pattern_index) = response.find(joining_pattern) {
                phone_number = response[pattern_index + joining_pattern.len()..]
                    .split(' ')
                    .next()
                    .unwrap_or("")
                    .trim()
                    .to_string();
                debug!("[LINK] found phone number: {}", phone_number);
                break;
            }
        }
    }

    Ok(phone_number)
}
