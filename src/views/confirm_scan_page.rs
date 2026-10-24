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

use crate::router::{Message, Page, Route};
use crate::ui::constants::{COLOR_BACKGROUND, COLOR_GREEN, COLOR_RED, MAX_LINES};
use crate::ui::printer::{Printer, Screen};
use crate::ui::{style::ColorString, widgets};
use crate::utils::sd_file::{ensure_dir, write_file};
use crate::utils::validate_config::validate_config;
use crate::views::{HomePage, ImportConfigPage};

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
        printer.println(Screen::Top, widgets::title_bar(Screen::Top));
        match (&self.uuid, &self.host, &self.port) {
            (Some(uuid), Some(host), Some(port)) => {
                printer.println(
                    Screen::Top,
                    ColorString::centered_full_line(
                        "\nImport config",
                        Screen::Top,
                        COLOR_BACKGROUND,
                    ),
                );
                printer.goto_line(Screen::Top, 8);
                printer.println(
                    Screen::Top,
                    ColorString::centered_full_line(
                        &format!(
                            "UUID: {uuid}\n\
                             \n\
                             Host: {host}:{port}"
                        ),
                        Screen::Top,
                        COLOR_BACKGROUND,
                    ),
                );
            }
            _ => {
                printer.println(
                    Screen::Top,
                    ColorString::centered_full_line(
                        "\nInvalid config.",
                        Screen::Top,
                        COLOR_BACKGROUND,
                    )
                    .with_fg_color(COLOR_RED),
                );
            }
        }

        if let Some(Err(error)) = &self.status {
            printer.println(
                Screen::Top,
                ColorString::centered_full_line(error, Screen::Top, COLOR_BACKGROUND)
                    .with_fg_color(COLOR_RED),
            );
        }

        let (status, status_color) = match &self.status {
            None => ("Press A to save\n", None),
            Some(Err(_)) if self.uuid.is_some() => ("Press A to retry saving\n", None),
            Some(Err(_)) => ("Press A to scan again\n", None),
            Some(Ok(())) => ("Saved !\n", Some(COLOR_GREEN)),
        };

        printer.goto_line(Screen::Top, MAX_LINES - 3);
        let mut status_line =
            ColorString::centered_full_line(status, Screen::Top, COLOR_BACKGROUND);
        if let Some(color) = status_color {
            status_line = status_line.with_fg_color(color);
        }
        printer.println(Screen::Top, status_line);
        printer.println(
            Screen::Top,
            ColorString::centered_full_line(
                "Press B to return to the menu",
                Screen::Top,
                COLOR_BACKGROUND,
            ),
        );

        printer.clear_with_background(Screen::Bottom, COLOR_BACKGROUND);

        printer.goto_last_line(Screen::Bottom);
        printer.println(
            Screen::Bottom,
            ColorString::centered_full_line(
                "Press START to exit ⌂",
                Screen::Bottom,
                COLOR_BACKGROUND,
            ),
        );
    }

    fn handle_input(&mut self, hid: &Hid) -> Message {
        let input = hid.keys_down();
        if input.contains(KeyPad::START) {
            return Message::Exit;
        }
        if input.contains(KeyPad::B) {
            return Message::Goto(Route::Home(HomePage::new()), String::new());
        }
        if input.contains(KeyPad::A) {
            match &self.status {
                None => {
                    self.status = Some(self.save());
                    return Message::NeedRedraw;
                }
                Some(Err(_)) if self.uuid.is_some() => {
                    // Config was valid but saving failed
                    self.status = Some(self.save());
                    return Message::NeedRedraw;
                }
                Some(Err(_)) => {
                    // Invalid config
                    return Message::Goto(
                        Route::ImportConfig(ImportConfigPage::new()),
                        String::new(),
                    );
                }
                Some(Ok(())) => {}
            }
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
