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

use std::{io::Read, sync::OnceLock};
use ureq::Agent;

pub fn fetch(url: &str) -> Result<String, String> {
    let agent = Agent::new_with_defaults();
    let response = agent.get(url).call().map_err(|error| error.to_string())?;

    let mut reader = response.into_body().into_reader();
    let mut text = String::new();
    reader
        .read_to_string(&mut text)
        .map_err(|error| format!("cannot read from {url}: {error}"))?;
    Ok(text)
}

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