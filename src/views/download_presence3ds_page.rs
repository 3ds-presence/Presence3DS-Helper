use crate::model::message::Message;
use crate::model::printer::{Printer, Screen};
use crate::utils::constant::{COLOR_BACKGROUND, COLOR_CYAN, COLOR_GREEN, COLOR_RED};
use crate::utils::sd_file::move_file;
use crate::utils::shape::progress_bar;
use crate::utils::{ColorString, DownloadJob, JobStatus, SdCardInfo};
use crate::views::page::Page;
use crate::views::{HomePage, Route};
use ctru::services::hid::{Hid, KeyPad};

const URL: &str = "https://3ds-presence.top/dyn/boot.firm";

const STAGED_FILE: &str = "boot.firm.new";
const TARGET_FILE: &str = "boot.firm";
const BACKUP_FILE: &str = "boot.firm.old";

pub struct DownloadPresence3DSPage {
    job: DownloadJob,
    sd_card: Option<SdCardInfo>,
}

impl Page for DownloadPresence3DSPage {
    fn render(&self, printer: &mut Printer<'_>) {
        printer.clear_with_background(Screen::Top, COLOR_BACKGROUND);
        printer.println(Screen::Top, "Download example");
        match self.job.status() {
            JobStatus::Idle => printer.println(Screen::Top, "Press A to download."),
            JobStatus::Connecting => printer.println(Screen::Top, "Connecting…"),
            JobStatus::Downloading {
                current,
                total,
                percentage,
            } => {
                let bar = progress_bar(*percentage, 30);
                printer.println(
                    Screen::Top,
                    ColorString::new(&format!("[{bar}] {percentage}%")).with_fg_color(COLOR_CYAN),
                );
                if *total == 0 {
                    printer.println(Screen::Top, format!("{current} bytes (size unknown)"));
                } else {
                    printer.println(Screen::Top, format!("{current} / {total} bytes"));
                }
                printer.println(
                    Screen::Top,
                    "Do not turn off your 3DS or remove the SD card.",
                );
            }
            JobStatus::Done => {
                printer.println(
                    Screen::Top,
                    ColorString::new("Done!").with_fg_color(COLOR_GREEN),
                );
                printer.println(Screen::Top, format!("{TARGET_FILE} installed"));
            }
            JobStatus::Failed(error) => printer.println(
                Screen::Top,
                ColorString::new(&format!("Failed: {error}")).with_fg_color(COLOR_RED),
            ),
        }
        if self.job.is_running() {
            printer.println(Screen::Top, "Press B to cancel.");
        }

        printer.clear_with_background(Screen::Bottom, COLOR_BACKGROUND);
        printer.println(Screen::Bottom, "A: download | B: cancel | START: exit");
        printer.print(Screen::Bottom, "SD card: ");
        let info = self.sd_card.map_or_else(
            || "not found".to_owned(),
            |info| {
                format!(
                    "{} bytes free, {} bytes total, {} sectors",
                    info.free_size(),
                    info.total_size(),
                    info.total_clusters
                )
            },
        );
        printer.println(Screen::Bottom, info);
    }

    fn handle_input(&mut self, hid: &Hid) -> Message {
        let input = hid.keys_down();
        if input.contains(KeyPad::START) && !self.job.is_running() {
            return Message::Exit;
        }

        if input.contains(KeyPad::B) && self.job.is_running() {
            self.job.cancel();
            return Message::NeedRedraw;
        }

        if input.contains(KeyPad::B) && !self.job.is_running() {
            return Message::Goto(Route::Home(HomePage::new()));
        }

        if input.contains(KeyPad::A) && !self.job.is_running() {
            self.job.start();
            return Message::NeedRedraw;
        }

        Message::None
    }

    fn update(&mut self) -> Message {
        let progressed = self.job.update();
        let mut message = if progressed {
            Message::NeedRedraw
        } else {
            Message::None
        };

        if self.job.take_finished() {
            move_file(TARGET_FILE, BACKUP_FILE).ok();
            if let Err(error) = move_file(STAGED_FILE, TARGET_FILE) {
                self.job.fail(error.to_string());
                message = Message::NeedRedraw;
            }
        }

        message
    }

    fn blocks_home(&self) -> bool {
        self.job.is_running()
    }
}

impl DownloadPresence3DSPage {
    pub fn new() -> Self {
        Self {
            job: DownloadJob::new(URL, STAGED_FILE),
            sd_card: SdCardInfo::query(),
        }
    }
}
