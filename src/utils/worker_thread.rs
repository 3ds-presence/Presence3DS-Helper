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
use ctru_sys::s32;
use io::Result;

pub enum ThreadPriority {
    High = 0x18,
    Normal = 0x2B, // hex((0x18 + 0x3F) // 2)
    Low = 0x3F,
}

pub fn spawn_with_priority<F>(stack_size: usize, job: F, priority: ThreadPriority) -> Result<()>
where
    F: FnOnce() + Send + 'static,
{
    spawn_with_arbitrary_priority(stack_size, job, priority as s32)
}

pub fn spawn_with_arbitrary_priority<F>(stack_size: usize, job: F, priority: s32) -> Result<()>
where
    F: FnOnce() + Send + 'static,
{
    let worker = thread::Builder::new()
        .stack_size(stack_size)
        .spawn(move || {
            set_priority(priority);
            job();
        })?;

    drop(worker);

    Ok(())
}

// https://github.com/rust3ds/ctru-rs/issues/82#issuecomment-2427878523
fn set_priority(prio: s32) {
    unsafe {
        let _ = ctru_sys::svcSetThreadPriority(ctru_sys::CUR_THREAD_HANDLE, prio);
    }
}
