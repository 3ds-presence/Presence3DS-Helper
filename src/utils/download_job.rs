// Presence3DS Helper — Helper Homebrew for Presence3DS
// Copyright (C) 2026 3DS Presence - LeonLeBreton
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with this program.  If not, see <https://www.gnu.org/licenses/>.

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{self, Receiver, TryRecvError};
use std::sync::Arc;

use crate::utils::sd_file::SdWriter;
use crate::utils::worker_thread::{self, ThreadPriority};
use crate::utils::{DownloadState, DownloadTask, SdCardInfo};

#[derive(PartialEq, Eq)]
pub enum JobStatus {
    Idle,
    Connecting,
    Downloading {
        current: usize,
        total: usize,
        percentage: u8,
    },
    Done,
    Failed(String),
}

impl JobStatus {
    pub const fn is_idle(&self) -> bool {
        matches!(self, Self::Idle)
    }

    pub const fn is_downloading(&self) -> bool {
        matches!(self, Self::Downloading { .. })
    }

    pub const fn is_connecting(&self) -> bool {
        matches!(self, Self::Connecting)
    }

    pub const fn is_done(&self) -> bool {
        matches!(self, Self::Done)
    }

    pub const fn is_failed(&self) -> bool {
        matches!(self, Self::Failed(_))
    }
}

pub struct DownloadJob {
    url: String,
    target: String,
    status: JobStatus,
    running: Option<Running>,
    finished: bool,
}

struct Transfer {
    task: DownloadTask<'static>,
    writer: SdWriter,
}

impl Transfer {
    fn start(url: &str, target: &str) -> Result<Self, String> {
        let sd_card = SdCardInfo::query().ok_or_else(|| "no SD card found".to_owned())?;
        let task = DownloadTask::start(url, sd_card.cluster_size as usize)
            .map_err(|error| error.to_string())?;

        let needed = task.total_size();
        if needed > 0 {
            let free = sd_card.free_size();
            if needed > free {
                return Err(format!(
                    "not enough space on the SD card: {needed} bytes needed, {free} bytes free"
                ));
            }
        }

        let writer = SdWriter::create(target).map_err(|error| error.to_string())?;

        Ok(Self { task, writer })
    }

    fn commit(self) -> Result<(), String> {
        self.writer.commit().map_err(|error| error.to_string())
    }

    fn abort(self) -> Result<(), String> {
        self.writer.abort().map_err(|error| error.to_string())
    }
}

struct Running {
    rx: Receiver<JobStatus>,
    cancel: Arc<AtomicBool>,
}

fn run_download(url: &str, target: &str, tx: &mpsc::Sender<JobStatus>, cancel: &Arc<AtomicBool>) {
    let mut transfer = match Transfer::start(url, target) {
        Ok(transfer) => transfer,
        Err(error) => {
            let _ = tx.send(JobStatus::Failed(error));
            return;
        }
    };

    loop {
        if cancel.load(Ordering::Relaxed) {
            let _ = transfer.abort();
            return;
        }
        match transfer.task.poll() {
            DownloadState::InProgress {
                current,
                total,
                percentage,
                current_chunk,
            } => {
                // Copy chunk: poll() borrows `transfer`, send() needs owned data.
                let chunk = current_chunk.to_vec();
                if let Err(error) = transfer.writer.write_chunk(&chunk) {
                    let _ = transfer.abort();
                    let _ = tx.send(JobStatus::Failed(error.to_string()));
                    return;
                }
                if tx
                    .send(JobStatus::Downloading {
                        current,
                        total,
                        percentage,
                    })
                    .is_err()
                {
                    let _ = transfer.abort();
                    return;
                }
            }
            DownloadState::Done => {
                let status = match transfer.commit() {
                    Ok(()) => JobStatus::Done,
                    Err(error) => JobStatus::Failed(error),
                };
                let _ = tx.send(status);
                return;
            }
            DownloadState::Failed(error) => {
                let _ = transfer.abort();
                let _ = tx.send(JobStatus::Failed(error));
                return;
            }
        }
    }
}

impl DownloadJob {
    pub fn new(url: &str, target: &str) -> Self {
        Self {
            url: url.to_owned(),
            target: target.to_owned(),
            status: JobStatus::Idle,
            running: None,
            finished: false,
        }
    }

    pub fn start(&mut self) {
        if self.running.is_some() {
            return;
        }

        self.finished = false;
        let (tx, rx) = mpsc::channel();
        let cancel = Arc::new(AtomicBool::new(false));
        let url = self.url.clone();
        let target = self.target.clone();
        let worker_cancel = Arc::clone(&cancel);
        if worker_thread::spawn_with_priority(
            512 * 1024,
            move || {
                run_download(&url, &target, &tx, &worker_cancel);
            },
            ThreadPriority::Low,
        )
        .is_err()
        {
            self.status = JobStatus::Failed("cannot start download thread".to_owned());
            return;
        }
        self.running = Some(Running { rx, cancel });
        self.status = JobStatus::Connecting;
    }

    pub fn cancel(&mut self) {
        self.finished = false;
        if let Some(running) = self.running.take() {
            running.cancel.store(true, Ordering::Relaxed);
            // Drain late worker messages so a stale Done/Failed cannot leak
            // into the next start().
            while running.rx.try_recv().is_ok() {}
        }
        self.status = JobStatus::Idle;
    }

    pub fn update(&mut self) -> bool {
        let Some(running) = self.running.as_ref() else {
            return false;
        };

        let mut progressed = false;
        loop {
            match running.rx.try_recv() {
                Ok(JobStatus::Downloading {
                    current,
                    total,
                    percentage,
                }) => {
                    self.status = JobStatus::Downloading {
                        current,
                        total,
                        percentage,
                    };
                    progressed = true;
                }
                Ok(JobStatus::Done) => {
                    self.running.take();
                    self.finished = true;
                    self.status = JobStatus::Done;
                    return true;
                }
                Ok(status) => {
                    self.running.take();
                    self.status = status;
                    return true;
                }
                Err(TryRecvError::Empty) => return progressed,
                Err(TryRecvError::Disconnected) => {
                    // Worker exited without a terminal status (e.g. cancelled):
                    self.running.take();
                    return progressed;
                }
            }
        }
    }

    pub fn take_finished(&mut self) -> bool {
        std::mem::take(&mut self.finished)
    }

    pub const fn status(&self) -> &JobStatus {
        &self.status
    }

    pub const fn is_running(&self) -> bool {
        self.running.is_some()
    }

    pub fn fail(&mut self, error: String) {
        if let Some(running) = self.running.take() {
            running.cancel.store(true, Ordering::Relaxed);
        }
        self.status = JobStatus::Failed(error);
    }
}
