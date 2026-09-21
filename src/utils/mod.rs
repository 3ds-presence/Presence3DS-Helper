pub mod color_string;
pub mod constant;
pub mod downloader;
pub mod sd_card;

pub use color_string::ColorString;
pub use downloader::{DownloadState, DownloadTask};
pub use sd_card::SdCardInfo;

pub fn progress_bar(percentage: u8, width: usize) -> String {
    let filled = (percentage as usize) * width / 100;
    let mut bar = String::with_capacity(width);
    bar.extend(std::iter::repeat_n('#', filled));
    bar.extend(std::iter::repeat_n('-', width - filled));
    bar
}
