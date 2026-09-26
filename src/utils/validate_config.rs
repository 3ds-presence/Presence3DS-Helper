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

pub fn validate_config(config: &str) -> Result<[String; 4], String> {
    let lines: Vec<&str> = config
        .lines()
        .map(str::trim)
        .filter(|l| !l.is_empty())
        .collect();
    let [uuid, aes, host, port] = lines[..] else {
        return Err(format!(
            "invalid config: expected 4 lines (uuid, aes key, host, port), got {}",
            lines.len()
        ));
    };

    validate_uuid(uuid)?;
    validate_aes(aes)?;
    validate_url(host)?;
    validate_port(port)?;

    Ok([
        uuid.to_owned(),
        aes.to_owned(),
        host.to_owned(),
        port.to_owned(),
    ])
}

pub fn validate_uuid(uuid: &str) -> Result<(), String> {
    let valid = uuid.len() == 36
        && uuid.bytes().enumerate().all(|(i, b)| {
            if matches!(i, 8 | 13 | 18 | 23) {
                b == b'-'
            } else {
                b.is_ascii_hexdigit()
            }
        });

    valid
        .then_some(())
        .ok_or_else(|| format!("invalid uuid {uuid:?}: expected 8-4-4-4-12 hexadecimal characters"))
}

pub fn validate_aes(key: &str) -> Result<(), String> {
    (key.len() == 64 && key.bytes().all(|b| b.is_ascii_hexdigit()))
        .then_some(())
        .ok_or_else(|| format!("invalid aes key {key:?}: expected 64 hexadecimal characters"))
}

pub fn validate_url(url: &str) -> Result<(), String> {
    let (host, path) = url
        .split_once('/')
        .map_or((url, None), |(h, p)| (h, Some(p)));
    let ipv4 = |h: &str| h.split('.').count() == 4 && h.split('.').all(|o| o.parse::<u8>().is_ok());
    let domain = |h: &str| {
        h.len() <= 253
            && h.split('.').all(|l| {
                !l.is_empty()
                    && l.len() <= 63
                    && !l.starts_with('-')
                    && !l.ends_with('-')
                    && l.bytes().all(|b| b.is_ascii_alphanumeric() || b == b'-')
            })
    };
    let host_ok = |h: &str| {
        if h.bytes().all(|b| b.is_ascii_digit() || b == b'.') {
            ipv4(h)
        } else {
            domain(h)
        }
    };
    let valid = !host.is_empty()
        && !host.contains(':')
        && !url.chars().any(char::is_whitespace)
        && host_ok(host)
        && path.is_none_or(|p| !p.chars().any(|c| c.is_whitespace() || c.is_control()));

    valid
        .then_some(())
        .ok_or_else(|| format!("invalid url {url:?}: expected a host"))
}

pub fn validate_port(port: &str) -> Result<(), String> {
    port.parse::<u16>()
        .ok()
        .filter(|p| *p > 0)
        .map(drop)
        .ok_or_else(|| format!("invalid port {port:?}: expected a number between 1 and 65535"))
}
