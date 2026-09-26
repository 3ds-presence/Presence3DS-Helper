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

use std::fs::File;
use std::io::{BufReader, Read};

use sha2::{Digest, Sha256};

use crate::utils::sd_file::{SdCardError, sd_path};

const HASH_CHUNK_SIZE: usize = 32 * 1024;

pub fn sha256_hex_of_file(path: &str) -> Result<String, SdCardError> {
    let resolved = sd_path(path)?;
    let file = File::open(&resolved).map_err(|source| SdCardError::Io {
        path: resolved.clone(),
        source,
    })?;
    let mut reader = BufReader::with_capacity(HASH_CHUNK_SIZE, file);
    let mut hasher = Sha256::new();
    let mut buffer = vec![0u8; HASH_CHUNK_SIZE];

    loop {
        let read = reader.read(&mut buffer).map_err(|source| SdCardError::Io {
            path: resolved.clone(),
            source,
        })?;
        if read == 0 {
            break;
        }
        hasher.update(&buffer[..read]);
    }

    let digest = hasher.finalize();
    Ok(format!("{digest:x}"))
}

pub fn fetch_expected_sha256(url: &str) -> Result<String, String> {
    let agent = ureq::Agent::new_with_defaults();
    let response = agent.get(url).call().map_err(|error| error.to_string())?;

    let mut reader = response.into_body().into_reader();
    let mut text = String::new();
    reader
        .read_to_string(&mut text)
        .map_err(|error| format!("cannot read sha256 file from {url}: {error}"))?;

    parse_sha256_text(&text).ok_or_else(|| {
        format!("invalid sha256 file from {url}: expected 64 hex characters, got {text:?}")
    })
}

pub fn verify_file_sha256(path: &str, hash_url: &str) -> Result<String, String> {
    let actual =
        sha256_hex_of_file(path).map_err(|error| format!("cannot hash {path:?}: {error}"))?;
    let expected = fetch_expected_sha256(hash_url)?;

    if actual == expected {
        Ok(actual)
    } else {
        Err(format!(
            "SHA256 mismatch for {path:?}: expected {expected}, got {actual}"
        ))
    }
}

fn parse_sha256_text(text: &str) -> Option<String> {
    let token = text.split_whitespace().next()?.trim().to_lowercase();
    if token.len() != 64 || !token.chars().all(|c| c.is_ascii_hexdigit()) {
        return None;
    }
    Some(token)
}
