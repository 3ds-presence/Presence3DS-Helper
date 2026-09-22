pub mod download_presence3ds_page;
pub mod home_page;
pub mod page;
pub mod second_example;

pub use download_presence3ds_page::DownloadPresence3DSPage;
pub use home_page::HomePage;
pub use page::Page;
pub use second_example::SecondExamplePage;

use crate::model::{Message, Printer};
use ctru::prelude::Hid;
use enum_dispatch::enum_dispatch;

#[enum_dispatch(Page)]
pub enum Route {
    Home(HomePage),
    DownloadPresence3DS(DownloadPresence3DSPage),
    SecondExample(SecondExamplePage),
}
