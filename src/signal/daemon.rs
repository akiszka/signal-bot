use nix::{
    sys::signal::{kill, Signal},
    unistd::Pid,
};
use rocket::futures::FutureExt;
use std::{error::Error, process::Stdio, sync::Arc, time::Duration};
use tokio::{
    io::{AsyncBufReadExt, BufReader},
    process::{Child, ChildStderr, Command},
    sync::Mutex,
};

#[derive(Clone)]
pub struct DaemonManager {
    daemon: Arc<Mutex<Child>>,
}

impl DaemonManager {
    pub async fn new() -> Result<DaemonManager, Box<dyn Error>> {
        trace!("starting signal-cli");

        let command = DaemonManager::start_daemon().await?;

        let child = Mutex::new(command);
        let child = Arc::new(child);

        Ok(DaemonManager { daemon: child })
    }

    async fn start_daemon() -> Result<Child, Box<dyn Error>> {
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

        let stderr = command.stderr.take().ok_or("Failed to redirect stderr")?;
        let reader = BufReader::new(stderr);

        tokio::select! {
            _ = tokio::time::sleep(Duration::from_secs(10)).fuse() => {
                debug!("Timeout: killing Signal.");
                command.kill().await?;
                panic!("Could not start signal-cli in time!");
            },
            value = read_until_listening(reader) => {
                if value.is_err() {
                    command.kill().await?;
                    panic!("Could not read from signal-cli!");
                }

                trace!("signal-cli started and is ready");
            }
        }

        Ok(command)
    }

    pub async fn stop(&self) -> Result<(), Box<dyn Error>> {
        let mut daemon = self.daemon.lock().await;

        let pid = match daemon.id() {
            Some(x) => Pid::from_raw(x.try_into()?),
            None => return Ok(()),
        };

        kill(pid, Signal::SIGINT)?;

        tokio::select! {
            _ = daemon.wait() => {
                trace!("signal-cli exitted successfully");
            },
            _ = tokio::time::sleep(Duration::from_secs(10)).fuse() => {
                warn!("signal-cli exit timeout! killing...");
                daemon.kill().await?;
            }
        };

        Ok(())
    }

    pub async fn restart(&self) -> Result<(), Box<dyn Error>> {
        self.stop().await?;
        let mut daemon = self.daemon.lock().await;
        *daemon = DaemonManager::start_daemon().await?;

        Ok(())
    }
}

// This is a helper function that reads from the stdout of signal-cli until it starts listening on the socket.
// This allows us to wait until the daemon is ready to accept connections.
async fn read_until_listening(mut stdout: BufReader<ChildStderr>) -> Result<(), std::io::Error> {
    let mut response = String::new();

    loop {
        response.clear();
        stdout.read_line(&mut response).await?;

        if !response.is_empty() {
            trace!("Signal: {}", &response.trim_end());

            if response.contains("Listening on socket") {
                debug!("signal-cli is listening");
                break;
            }
        }
    }

    Ok(())
}
