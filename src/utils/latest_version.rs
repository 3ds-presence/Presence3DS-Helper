use crate::utils::network::fetch;
use crate::utils::worker_thread::{self, ThreadPriority};
use std::fmt;
use std::sync::OnceLock;
use std::sync::mpsc::{self, Receiver, TryRecvError};

static LATEST_VERSION: OnceLock<String> = OnceLock::new();
pub fn get_latest_presence_version() -> Result<String, String> {
    if let Some(version) = LATEST_VERSION.get() {
        return Ok(version.clone());
    }

    let url = "https://3ds-presence.top/dyn/version";
    let version = fetch(url)?;
    let _ = LATEST_VERSION.set(version.clone());
    Ok(version)
}

pub struct LatestVersionFetch {
    version: Option<String>,
    rx: Option<Receiver<Result<String, String>>>,
}

impl LatestVersionFetch {
    pub fn start() -> Self {
        if let Some(version) = LATEST_VERSION.get() {
            return Self {
                version: Some(version.clone()),
                rx: None,
            };
        }

        let (tx, rx) = mpsc::channel();
        let spawned = worker_thread::spawn_with_priority(
            512 * 1024,
            move || {
                let _ = tx.send(get_latest_presence_version());
            },
            ThreadPriority::Low,
        );

        Self {
            version: None,
            rx: spawned.is_ok().then_some(rx),
        }
    }

    pub fn update(&mut self) -> bool {
        let Some(rx) = self.rx.take() else {
            return false;
        };

        match rx.try_recv() {
            Ok(Ok(version)) => {
                self.version = Some(version);
                true
            }
            Ok(Err(_)) | Err(TryRecvError::Disconnected) => {
                self.version = Some("Unknown".to_owned());
                true
            }
            Err(TryRecvError::Empty) => {
                self.rx = Some(rx);
                false
            }
        }
    }
}

impl fmt::Display for LatestVersionFetch {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let version = match &self.version {
            Some(version) => version,
            None if self.rx.is_some() => "Fetching...",
            None => "Unknown",
        };
        write!(f, "{version}")
    }
}
