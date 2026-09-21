pub mod download_example_page;
pub mod home_page;
pub mod page;
pub mod second_example;

pub use download_example_page::DownloadExamplePage;
pub use home_page::HomePage;
pub use page::Page;
pub use second_example::SecondExamplePage;

use crate::model::{Message, Printer};
use ctru::prelude::Hid;
use enum_dispatch::enum_dispatch;

#[enum_dispatch(Page)]
pub enum Route {
    Home(HomePage),
    DownloadExample(DownloadExamplePage),
    SecondExample(SecondExamplePage),
}
