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

#![allow(dead_code)]
pub mod camera;
pub mod color_string;
pub mod constant;
pub mod download_job;
pub mod downloader;
pub mod hash;
pub mod sd_card;
pub mod sd_file;
pub mod shape;
pub mod validate_config;

pub use camera::Camera;
pub use color_string::ColorString;
pub use download_job::{DownloadJob, JobStatus};
pub use downloader::{DownloadState, DownloadTask};
pub use sd_card::SdCardInfo;
