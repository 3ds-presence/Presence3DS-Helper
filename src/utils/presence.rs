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

use std::ffi::CString;
use std::fmt;
use std::sync::OnceLock;

use ctru::error::ResultCode;
use ctru::services::svc::{HandleExt, make_ipc_header};
use ctru_sys::Handle;

static PRESENCE_VERSION_SINGLETON: OnceLock<Option<PresenceVersion>> = OnceLock::new();

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PresenceVersion {
    pub major: u8,
    pub minor: u8,
    pub patch: u8,
}

impl PresenceVersion {
    const fn from_packed(packed: u32) -> Self {
        Self {
            major: ((packed >> 24) & 0xFF) as u8,
            minor: ((packed >> 16) & 0xFF) as u8,
            patch: ((packed >> 8) & 0xFF) as u8,
        }
    }
}

impl fmt::Display for PresenceVersion {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "v{}.{}.{}", self.major, self.minor, self.patch)
    }
}

struct ServiceHandle {
    handle: Handle,
}

impl ServiceHandle {
    const fn new(handle: Handle) -> Self {
        Self { handle }
    }

    fn new_from_service_name(service_name: &str) -> ctru::Result<Self> {
        let name = CString::new(service_name).expect("service name without NUL");

        let mut handle = Handle::default();
        let handle_result = unsafe {
            ResultCode(ctru_sys::srvGetServiceHandle(
                &raw mut handle,
                name.as_ptr(),
            ))
        };
        handle_result?;
        Ok(Self::new(handle))
    }

    unsafe fn send_service_request(
        &self,
        request: Vec<u32>,
        response_size: usize,
    ) -> ctru::Result<Vec<u32>> {
        unsafe { self.handle.send_service_request(request, response_size) }
    }
}

impl Drop for ServiceHandle {
    fn drop(&mut self) {
        unsafe {
            let _ = ctru_sys::svcCloseHandle(self.handle);
        }
    }
}

pub fn is_presence3ds_installed() -> bool {
    let name = CString::new("p3ds:ver").expect("service name without NUL");

    let mut registered = false;
    unsafe {
        if ctru_sys::srvIsServiceRegistered(&raw mut registered, name.as_ptr()) != 0 {
            return false;
        }
    }
    registered
}

/// Get the version of `Presence3DS` installed on the system or an error if `Presence3DS` is not installed or not running.
pub fn version() -> ctru::Result<PresenceVersion> {
    let data = PRESENCE_VERSION_SINGLETON.get_or_init(|| {
        if !is_presence3ds_installed() {
            return None;
        }

        let service = ServiceHandle::new_from_service_name("p3ds:ver").ok()?;
        let reply = unsafe {
            service
                .send_service_request(vec![make_ipc_header(0x1, 0, 0)], 3)
                .ok()?
        };

        Some(PresenceVersion::from_packed(reply[2]))
    });

    data.as_ref().map_or_else(
        || {
            Err(ctru::Error::Other(
                "Presence3DS is not installed or not running".to_string(),
            ))
        },
        |version| Ok(*version),
    )
}
