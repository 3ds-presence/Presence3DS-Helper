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

use std::io;
use std::thread;

pub fn spawn_low_priority<F>(stack_size: usize, job: F) -> io::Result<()>
where
    F: FnOnce() + Send + 'static,
{
    let worker = thread::Builder::new()
        .stack_size(stack_size)
        .spawn(move || {
            lower_own_priority();
            job();
        })?;

    drop(worker);

    Ok(())
}

// https://github.com/rust3ds/ctru-rs/issues/82#issuecomment-2427878523
fn lower_own_priority() {
    unsafe {
        let _ = ctru_sys::svcSetThreadPriority(ctru_sys::CUR_THREAD_HANDLE, 0x3F);
    }
}
