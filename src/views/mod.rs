pub mod confirm_scan_page;
pub mod download_presence3ds_page;
pub mod home_page;
pub mod page;
pub mod import_config_page;

pub use confirm_scan_page::ConfirmScanPage;
pub use download_presence3ds_page::DownloadPresence3DSPage;
pub use home_page::HomePage;
pub use page::Page;
pub use import_config_page::ImportConfigPage;

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
