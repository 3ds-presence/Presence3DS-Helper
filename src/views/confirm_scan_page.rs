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

use crate::model::{Message, Printer, Screen};
use crate::utils::ColorString;
use crate::utils::constant::{COLOR_BACKGROUND, COLOR_GREEN, COLOR_RED};
use crate::utils::sd_file::{ensure_dir, write_file};
use crate::utils::validate_config::validate_config;
use crate::views::{HomePage, Page, Route};

pub struct ConfirmScanPage {
    uuid: Option<String>,
    aes: Option<String>,
    host: Option<String>,
    port: Option<String>,
    status: Option<Result<(), String>>,
}

impl Page for ConfirmScanPage {
    fn render(&self, printer: &mut Printer<'_>) {
        printer.clear_with_background(Screen::Top, COLOR_BACKGROUND);
        printer.println(Screen::Top, "Import config : ");
        if let (Some(uuid), Some(_aes), Some(host), Some(port)) =
            (&self.uuid, &self.aes, &self.host, &self.port)
        {
            printer.println(Screen::Top, format!("UUID: {uuid}"));
            printer.println(Screen::Top, format!("Host: {host}:{port}"));
        } else {
            printer.println(Screen::Top, "Invalid config.");
        }

        printer.clear_with_background(Screen::Bottom, COLOR_BACKGROUND);
        match &self.status {
            Some(Ok(())) => printer.println(
                Screen::Bottom,
                ColorString::new("Saved to sdmc:/presence3ds/discord_rpc.conf")
                    .with_fg_color(COLOR_GREEN),
            ),
            Some(Err(error)) => printer.println(
                Screen::Bottom,
                ColorString::new(error).with_fg_color(COLOR_RED),
            ),
            None => printer.println(Screen::Bottom, "A: save | B: back to home | START: exit"),
        }
    }

    fn handle_input(&mut self, hid: &Hid) -> Message {
        let input = hid.keys_down();
        if input.contains(KeyPad::START) {
            return Message::Exit;
        }
        if input.contains(KeyPad::B) {
            return Message::Goto(Route::Home(HomePage::new()), String::new());
        }
        if input.contains(KeyPad::A) && self.status.is_none() {
            self.status = Some(self.save());
            return Message::NeedRedraw;
        }

        Message::None
    }

    fn on_goto(&mut self, payload: String) {
        match validate_config(&payload) {
            Ok(config) => {
                let [uuid, aes, host, port] = config;
                self.uuid = Some(uuid);
                self.aes = Some(aes);
                self.host = Some(host);
                self.port = Some(port);
                self.status = None;
            }
            Err(error) => self.status = Some(Err(error)),
        }
    }
}

impl ConfirmScanPage {
    pub const fn new() -> Self {
        Self {
            uuid: None,
            aes: None,
            host: None,
            port: None,
            status: None,
        }
    }

    fn save(&self) -> Result<(), String> {
        ensure_dir("/presence3ds").map_err(|error| error.to_string())?;
        write_file(
            "/presence3ds/discord_rpc.conf",
            format!(
                "UUID={}\nAES_KEY={}\nSERVER_HOST={}\nSERVER_PORT={}\n",
                self.uuid.as_ref().unwrap_or(&String::new()),
                self.aes.as_ref().unwrap_or(&String::new()),
                self.host.as_ref().unwrap_or(&String::new()),
                self.port.as_ref().unwrap_or(&String::new())
            )
            .as_bytes(),
        )
        .map_err(|error| error.to_string())
    }
}
