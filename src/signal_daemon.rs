use nix::{
    sys::signal::{kill, Signal},
    unistd::Pid,
};
use rocket::futures::FutureExt;
use std::{error::Error, process::Stdio, time::Duration};
use tokio::{
    io::{AsyncBufReadExt, BufReader},
    process::{Child, ChildStderr, Command},
};

pub async fn start() -> Result<Child, std::io::Error> {
    println!("starting signal-cli");

    let mut command = Command::new("signal-cli")
        .args(&[
            "daemon",
            "--socket",
            "./signal.sock",
            "--no-receive-stdout",
            "--ignore-attachments",
        ])
        .kill_on_drop(true)
        .stdin(Stdio::null())
        .stdout(Stdio::inherit())
        .stderr(Stdio::piped())
        .spawn()?;

    let stderr = command.stderr.take().unwrap();
    let reader = BufReader::new(stderr);

    tokio::select! {
        _ = tokio::time::sleep(Duration::from_secs(10)).fuse() => {
            println!("Timeout: killing Signal.");
            stop(command).await.unwrap();
            panic!("Could not start signal-cli in time!");
        },
        value = read_until_listening(reader) => {
            value.unwrap();
            println!("signal-cli started");
        }
    }

    Ok(command)
}

// This is a helper function that reads from the stdout of signal-cli until it starts listening on the socket.
// This allows us to wait until the daemon is ready to accept connections.
async fn read_until_listening(mut stdout: BufReader<ChildStderr>) -> Result<(), std::io::Error> {
    let mut response = String::new();

    loop {
        stdout.read_line(&mut response).await?;
        println!("Signal: {}", response);

        if response.contains("Listening on socket") {
            println!("signal-cli is listening");
            break;
        }
    }

    Ok(())
}

// This tries to stop Signal gracefully to give it a chance to clean up.
// If it doesn't stop within a set delay, it kills the daemon.
pub async fn stop(mut daemon: Child) -> Result<(), Box<dyn Error>> {
    let pid = match daemon.id() {
        Some(x) => Pid::from_raw(x.try_into()?),
        None => return Ok(()),
    };

    kill(pid, Signal::SIGINT)?;

    tokio::select! {
        _ = daemon.wait() => {
            println!("signal-cli exitted successfully");
        },
        _ = tokio::time::sleep(Duration::from_secs(10)).fuse() => {
            println!("signal-cli exit timeout! killing...");
            daemon.kill().await?;
        }
    };

    Ok(())
}

// This restarts the Signal daemon, panicking on error.
pub async fn restart(daemon: Child) -> Child {
    stop(daemon).await.unwrap();
    start().await.unwrap()
}
