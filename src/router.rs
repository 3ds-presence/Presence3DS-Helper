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

use crate::ui::printer::Printer;
use crate::views::confirm_scan_page::ConfirmScanPage;
use crate::views::download_presence3ds_page::DownloadPresence3DSPage;
use crate::views::home_page::HomePage;
use crate::views::import_config_page::ImportConfigPage;
use ctru::services::hid::Hid;
use enum_dispatch::enum_dispatch;

#[enum_dispatch]
pub trait Page {
    fn render(&self, print: &mut Printer<'_>);
    fn handle_input(&mut self, hid: &Hid) -> Message;
    fn update(&mut self) -> Message {
        Message::None
    }

    fn on_goto(&mut self, _payload: String) {}

    fn draws_top_screen_manually(&self) -> bool {
        false
    }

    fn blocks_home(&self) -> bool {
        false
    }
}

#[enum_dispatch(Page)]
pub enum Route {
    Home(HomePage),
    DownloadPresence3DS(DownloadPresence3DSPage),
    ImportConfig(ImportConfigPage),
    ConfirmScan(ConfirmScanPage),
}

pub enum Message {
    None,
    NeedRedraw,
    Goto(Route, String),
    Exit,
}
