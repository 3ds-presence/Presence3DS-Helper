use crate::model::message::Message;
use crate::model::printer::{Printer, Screen};
use crate::utils::{
    DownloadState, DownloadTask, SdCardInfo, color_string::ColorString, shape::progress_bar,
};
use crate::views::page::Page;
use ctru::services::hid::{Hid, KeyPad};

const URL: &str = "https://httpbin.org/bytes/100000";

enum Status {
    Idle,
    Done,
    Failed(String),
}

pub struct DownloadExamplePage {
    sd_card: Option<SdCardInfo>,
    task: Option<DownloadTask<'static>>,
    progress: Option<(usize, usize, u8)>, // (current, total, %)
    status: Status,
}

impl Page for DownloadExamplePage {
    fn render(&self, printer: &mut Printer<'_>) {
        printer.clear_with_background(Screen::Top, [30, 34, 45]);
        printer.println(Screen::Top, "Download example");

        if self.task.is_some() {
            match self.progress {
                Some((current, total, percentage)) => {
                    let bar = progress_bar(percentage, 30);
                    printer.println(
                        Screen::Top,
                        ColorString::new(&format!("[{bar}] {percentage}%"))
                            .with_fg_color([0, 200, 255]),
                    );
                    printer.println(Screen::Top, format!("{current} / {total} bytes"));
                }
                None => printer.println(Screen::Top, "Connecting…"),
            }
            printer.println(Screen::Top, "Press B to cancel.");
        } else {
            match &self.status {
                Status::Idle => printer.println(Screen::Top, "Press A to download."),
                Status::Done => printer.println(
                    Screen::Top,
                    ColorString::new("Done!").with_fg_color([0, 255, 0]),
                ),
                Status::Failed(err) => printer.println(
                    Screen::Top,
                    ColorString::new(&format!("Failed: {err}")).with_fg_color([255, 0, 0]),
                ),
            }
        }

        printer.clear_with_background(Screen::Bottom, [30, 34, 45]);
        printer.println(Screen::Bottom, "A: download | B: cancel | START: exit");
    }

    fn handle_input(&mut self, hid: &Hid) -> Message {
        let input = hid.keys_down();
        if input.contains(KeyPad::START) {
            return Message::Exit;
        }

        if input.contains(KeyPad::B) && self.task.take().is_some() {
            self.progress = None;
            self.status = Status::Idle;
            return Message::NeedRedraw;
        }

        if input.contains(KeyPad::A) && self.task.is_none() {
            match DownloadTask::start(URL, self.sd_card.unwrap().sector_size as usize) {
                Ok(task) => self.task = Some(task),
                Err(e) => self.status = Status::Failed(e.to_string()),
            }
            return Message::NeedRedraw;
        }

        Message::None
    }

    fn update(&mut self) -> Message {
        let Some(task) = self.task.as_mut() else {
            return Message::None;
        };

        let outcome = match task.poll() {
            DownloadState::InProgress {
                current,
                total,
                percentage,
                current_chunk,
            } => {
                self.progress = Some((current, total, percentage));
                return Message::NeedRedraw;
            }
            DownloadState::Done => Status::Done,
            DownloadState::Failed(err) => Status::Failed(err),
        };

        self.task = None;
        self.progress = None;
        self.status = outcome;
        Message::NeedRedraw
    }
}

impl DownloadExamplePage {
    pub fn new() -> Self {
        Self {
            sd_card: SdCardInfo::query(),
            task: None,
            progress: None,
            status: Status::Idle,
        }
    }
}
