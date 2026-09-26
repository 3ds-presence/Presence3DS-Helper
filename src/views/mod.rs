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

pub mod confirm_scan_page;
pub mod download_presence3ds_page;
pub mod home_page;
pub mod import_config_page;
pub mod page;

pub use confirm_scan_page::ConfirmScanPage;
pub use download_presence3ds_page::DownloadPresence3DSPage;
pub use home_page::HomePage;
pub use import_config_page::ImportConfigPage;
pub use page::Page;

use crate::model::{Message, Printer};
use ctru::prelude::Hid;
use enum_dispatch::enum_dispatch;

#[enum_dispatch(Page)]
pub enum Route {
    Home(HomePage),
    DownloadPresence3DS(DownloadPresence3DSPage),
    ImportConfig(ImportConfigPage),
    ConfirmScan(ConfirmScanPage),
}
