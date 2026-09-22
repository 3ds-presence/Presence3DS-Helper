use crate::utils::sd_file::SdWriter;
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

pub struct DownloadJob {
    url: String,
    target: String,
    status: JobStatus,
    transfer: Option<Transfer>,
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

impl DownloadJob {
    pub fn new(url: &str, target: &str) -> Self {
        Self {
            url: url.to_owned(),
            target: target.to_owned(),
            status: JobStatus::Idle,
            transfer: None,
            finished: false,
        }
    }

    pub fn start(&mut self) {
        if self.transfer.is_some() {
            return;
        }

        self.finished = false;
        self.status = match Transfer::start(&self.url, &self.target) {
            Ok(transfer) => {
                self.transfer = Some(transfer);
                JobStatus::Connecting
            }
            Err(error) => JobStatus::Failed(error),
        };
    }

    pub fn cancel(&mut self) {
        self.finished = false;
        self.status = match self.transfer.take().map(Transfer::abort) {
            Some(Err(error)) => JobStatus::Failed(error),
            _ => JobStatus::Idle,
        };
    }

    pub fn update(&mut self) -> bool {
        let Some(transfer) = self.transfer.as_mut() else {
            return false;
        };

        match transfer.task.poll() {
            DownloadState::InProgress {
                current,
                total,
                percentage,
                current_chunk,
            } => {
                if let Err(error) = transfer.writer.write_chunk(current_chunk) {
                    self.fail(error.to_string());
                } else {
                    self.status = JobStatus::Downloading {
                        current,
                        total,
                        percentage,
                    };
                }
            }
            DownloadState::Done => {
                if let Some(transfer) = self.transfer.take() {
                    match transfer.commit() {
                        Ok(()) => {
                            self.finished = true;
                            self.status = JobStatus::Done;
                        }
                        Err(error) => self.status = JobStatus::Failed(error),
                    }
                }
            }
            DownloadState::Failed(error) => self.fail(error),
        }

        true
    }

    pub fn take_finished(&mut self) -> bool {
        std::mem::take(&mut self.finished)
    }

    pub const fn status(&self) -> &JobStatus {
        &self.status
    }

    pub const fn is_running(&self) -> bool {
        self.transfer.is_some()
    }

    pub fn fail(&mut self, error: String) {
        self.status = match self.transfer.take().map(Transfer::abort) {
            Some(Err(cleanup)) => JobStatus::Failed(format!("{error}; clean-up failed: {cleanup}")),
            _ => JobStatus::Failed(error),
        };
    }
}
