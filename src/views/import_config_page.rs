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

use std::sync::{Arc, Mutex};

use crate::model::{Message, Printer, Screen};
use crate::utils::camera::{blit_to_top_screen, decode_qr_from_luma};
use crate::utils::constant::{COLOR_BACKGROUND, COLOR_RED};
use crate::utils::worker_thread;
use crate::utils::{Camera, ColorString};
use crate::views::{ConfirmScanPage, HomePage, Page, Route};
use ctru::services::hid::{Hid, KeyPad};

enum QrScanResult {
    NotFound,
    Found(String),
}

pub struct ImportConfigPage {
    camera: Result<Camera, String>,
    pending: Option<Arc<Mutex<Option<QrScanResult>>>>,
}

impl Page for ImportConfigPage {
    fn render(&self, printer: &mut Printer<'_>) {
        printer.clear_with_background(Screen::Bottom, COLOR_BACKGROUND);
        match &self.camera {
            Ok(_) => printer.println(Screen::Bottom, "Point the QR code at the camera."),
            Err(error) => printer.println(
                Screen::Bottom,
                ColorString::new(error).with_fg_color(COLOR_RED),
            ),
        }
        printer.println(Screen::Bottom, "B: back | START: exit");
    }

    fn handle_input(&mut self, hid: &Hid) -> Message {
        let input = hid.keys_down();
        if input.contains(KeyPad::START) {
            return Message::Exit;
        }
        if input.contains(KeyPad::B) {
            return Message::Goto(Route::Home(HomePage::new()), String::new());
        }

        Message::None
    }

    fn update(&mut self) -> Message {
        let Ok(camera) = &mut self.camera else {
            return Message::None;
        };

        if camera.poll() {
            let (width, height) = camera.size();
            blit_to_top_screen(camera.frame(), width, height);

            if let Some(content) = Self::scan_qr(camera, &mut self.pending) {
                return Message::Goto(Route::ConfirmScan(ConfirmScanPage::new()), content);
            }
        }

        Message::None
    }

    fn draws_top_screen_manually(&self) -> bool {
        true
    }
}

impl ImportConfigPage {
    pub fn new() -> Self {
        Self {
            camera: Camera::new().map_err(|error| error.to_string()),
            pending: None,
        }
    }

    fn scan_qr(
        camera: &Camera,
        pending: &mut Option<Arc<Mutex<Option<QrScanResult>>>>,
    ) -> Option<String> {
        let finished = pending
            .as_ref()
            .and_then(|slot| slot.try_lock().ok())
            .and_then(|mut guard| guard.take());
        if let Some(result) = finished {
            *pending = None;
            // Scan result
            if let QrScanResult::Found(content) = result {
                return Some(content);
            }
        }

        // Scan in progress
        if pending.is_some() {
            return None;
        }

        // New scan
        let (width, height) = camera.size();
        let luma = camera.frame_luma();
        let slot = Arc::new(Mutex::new(None));
        let worker_slot = Arc::clone(&slot);
        let job = move || {
            let content = decode_qr_from_luma(width, height, &luma);
            let result = content.map_or(QrScanResult::NotFound, QrScanResult::Found);
            if let Ok(mut guard) = worker_slot.lock() {
                *guard = Some(result);
            }
        };

        match worker_thread::spawn_with_priority(512 * 1024, job, worker_thread::ThreadPriority::Low) {
            Ok(()) => {
                *pending = Some(slot);
                None
            }
            Err(_) => camera.scan_qr(),
        }
    }
}
