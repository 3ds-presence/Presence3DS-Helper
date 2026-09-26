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
