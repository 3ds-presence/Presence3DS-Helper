use std::io::Read;

use ureq::Agent;

pub struct DownloadTask<'a> {
    reader: Box<dyn Read>,
    chunk: Vec<u8>,
    current: usize,
    total: usize,
    terminal: Option<DownloadState<'a>>,
}

impl DownloadTask<'_> {
    pub fn start(url: &str, chunk_size: usize) -> Result<Self, ureq::Error> {
        let agent = Agent::new_with_defaults();
        let response = agent.get(url).call()?;

        let total = usize::try_from(response.body().content_length().unwrap_or(0)).unwrap_or(0);

        Ok(Self {
            reader: Box::new(response.into_body().into_reader()),
            chunk: vec![0u8; chunk_size],
            current: 0,
            total,
            terminal: None,
        })
    }

    pub fn poll(&mut self) -> DownloadState<'_> {
        if let Some(done) = self.terminal_state() {
            return done;
        }

        match self.reader.read(&mut self.chunk) {
            Ok(0) => {
                self.terminal = Some(DownloadState::Done);
                self.terminal.clone().unwrap()
            }
            Ok(bytes_read) => {
                self.current += bytes_read;

                DownloadState::InProgress {
                    current: self.current,
                    total: self.total,
                    percentage: self.percentage(),
                    current_chunk: &self.chunk[..bytes_read],
                }
            }
            Err(e) => {
                let state = DownloadState::Failed(e.to_string());
                self.terminal = Some(state.clone());
                state
            }
        }
    }

    pub const fn total_size(&self) -> usize {
        self.total
    }

    fn terminal_state(&self) -> Option<DownloadState<'_>> {
        self.terminal.clone()
    }

    fn percentage(&self) -> u8 {
        (self.current * 100)
            .checked_div(self.total)
            .map_or(0, |pct| u8::try_from(pct.min(100)).unwrap_or(100))
    }
}

#[derive(Clone, Debug)]
pub enum DownloadState<'a> {
    InProgress {
        current: usize,
        total: usize,
        percentage: u8,
        current_chunk: &'a [u8],
    },
    Done,
    Failed(String),
}
