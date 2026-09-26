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

use ctru::services::hid::{Hid, KeyPad};
use ctru_sys::svcGetSystemTick;

use crate::model::{Message, Printer, Screen};
use crate::utils::camera::blit_to_top_screen;
use crate::utils::constant::{COLOR_BACKGROUND, COLOR_RED};
use crate::utils::{Camera, ColorString};
use crate::views::{ConfirmScanPage, HomePage, Page, Route};

const SCAN_INTERVAL_TICKS: u64 = 268_111_856 * 3 / 10;

pub struct ImportConfigPage {
    camera: Result<Camera, String>,
    next_scan_tick: u64,
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

            let now = unsafe { svcGetSystemTick() };
            if now >= self.next_scan_tick {
                self.next_scan_tick = now + SCAN_INTERVAL_TICKS;
                if let Some(content) = camera.scan_qr() {
                    return Message::Goto(Route::ConfirmScan(ConfirmScanPage::new()), content);
                }
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
            next_scan_tick: 0,
        }
    }
}
