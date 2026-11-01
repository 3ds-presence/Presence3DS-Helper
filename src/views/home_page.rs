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

use crate::router::{Message, Page, Route};
use crate::ui::constants::{COLOR_BACKGROUND, COLOR_RED, MAX_LINES};
use crate::ui::printer::{Printer, Screen};
use crate::ui::{style::ColorString, widgets};
use crate::utils::network::get_latest_presence_version;
use crate::utils::{PresenceVersion, presence};
use crate::views::{DownloadPresence3DSPage, ImportConfigPage};
use ctru::services::hid::{Hid, KeyPad};

struct RouteName {
    name: &'static str,
    route: fn() -> Route,
}

pub struct HomePage {
    current_selection: usize,
    router: Box<[RouteName]>,
    presence_ver: Option<PresenceVersion>,
}

impl Page for HomePage {
    fn render(&self, printer: &mut Printer<'_>) {
        printer.clear_with_background(Screen::Top, COLOR_BACKGROUND);
        printer.println(Screen::Top, widgets::title_bar(Screen::Top));
        printer.println(Screen::Top, "");

        for (i, route) in self.router.iter().enumerate() {
            if i == self.current_selection {
                printer.println(
                    Screen::Top,
                    ColorString::new(route.name, Screen::Top)
                        .with_fg_color(COLOR_RED)
                        .with_bg_color(COLOR_BACKGROUND),
                );
            } else {
                printer.println(
                    Screen::Top,
                    ColorString::new(route.name, Screen::Top).with_bg_color(COLOR_BACKGROUND),
                );
            }
        }
        printer.clear_with_background(Screen::Bottom, COLOR_BACKGROUND);

        let nb_line = 10;
        printer.goto_line(Screen::Bottom, (MAX_LINES - nb_line) / 2);
        printer.println(
            Screen::Bottom,
            ColorString::centered_full_line(
                "\
                        3DS Presence\n\
                        Discord RPC for the Nintendo 3DS\n\
                        \n\
                        Use this app to download Presence3DS\n\
                        and connect to your 3DS Presence account\n\
                        \n\
                        \n\
                        Presence3DS status:\n\
                    ",
                Screen::Bottom,
                COLOR_BACKGROUND,
            ),
        );

        let installed_version = self
            .presence_ver
            .as_ref()
            .map_or_else(|| "Not installed".into(), std::string::ToString::to_string);

        printer.println(
            Screen::Bottom,
            ColorString::new(
                &format!("\tInstalled version: {installed_version}"),
                Screen::Bottom,
            )
            .with_bg_color(COLOR_BACKGROUND),
        );

        let latest_version = get_latest_presence_version().unwrap_or_else(|_| "Unknown".into());
        printer.println(
            Screen::Bottom,
            ColorString::new(&format!("\tLatest version: {latest_version}"), Screen::Bottom)
                .with_bg_color(COLOR_BACKGROUND),
        );

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
        if input.intersects(KeyPad::UP) {
            self.current_selection = self.current_selection.saturating_sub(1);
            return Message::NeedRedraw;
        }
        if input.intersects(KeyPad::DOWN) {
            let max_index = self.router.len().saturating_sub(1);
            self.current_selection = (self.current_selection + 1).min(max_index);
            return Message::NeedRedraw;
        }

        if input.contains(KeyPad::A) {
            return Message::Goto((self.router[self.current_selection].route)(), String::new());
        }

        Message::None
    }
}

impl HomePage {
    pub fn new() -> Self {
        let router: [RouteName; 2] = [
            RouteName {
                name: "Download Presence 3DS",
                route: || Route::DownloadPresence3DS(DownloadPresence3DSPage::new()),
            },
            RouteName {
                name: "Import config from QR",
                route: || Route::ImportConfig(ImportConfigPage::new()),
            },
        ];
        Self {
            current_selection: 0,
            router: router.into(),
            presence_ver: presence::version().ok(),
        }
    }
}
