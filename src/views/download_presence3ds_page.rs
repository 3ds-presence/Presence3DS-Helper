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
use crate::ui::constants::{COLOR_BACKGROUND, COLOR_GREEN, COLOR_RED};
use crate::ui::printer::{Printer, Screen};
use crate::ui::{style::ColorString, widgets};
use crate::utils::hash::verify_file_sha256;
use crate::utils::power;
use crate::utils::sd_file::{delete_file, move_file};
use crate::utils::{DownloadJob, JobStatus, SdCardInfo};
use crate::views::HomePage;
use ctru::services::hid::{Hid, KeyPad};

const URL: &str = "https://3ds-presence.top/dyn/boot.firm";
const HASH_URL: &str = "https://3ds-presence.top/dyn/boot.firm.sha256";

const STAGED_FILE: &str = "boot.firm.new";
const TARGET_FILE: &str = "boot.firm";
const BACKUP_FILE: &str = "boot.firm.old";

pub struct DownloadPresence3DSPage {
    job: DownloadJob,
    sd_card: Option<SdCardInfo>,
    verifying: bool,
}

impl Page for DownloadPresence3DSPage {
    fn render(&self, printer: &mut Printer<'_>) {
        printer.clear_with_background(Screen::Top, COLOR_BACKGROUND);
        printer.println(Screen::Top, widgets::title_bar(Screen::Top));
        printer.println(
            Screen::Top,
            ColorString::centered_full_line(
                "\nDownload/Update Presence 3DS",
                Screen::Top,
                COLOR_BACKGROUND,
            ),
        );
        printer.goto_line(Screen::Top, 8);
        match self.job.status() {
            JobStatus::Idle => printer.println(
                Screen::Top,
                ColorString::centered_full_line(
                    "Press A to download.",
                    Screen::Top,
                    COLOR_BACKGROUND,
                ),
            ),
            JobStatus::Connecting => printer.println(
                Screen::Top,
                ColorString::centered_full_line("Connecting...", Screen::Top, COLOR_BACKGROUND),
            ),
            JobStatus::Downloading {
                current,
                total,
                percentage,
            } => {
                widgets::download_section(printer, TARGET_FILE, *current, *total, *percentage);

                printer.println(
                    Screen::Top,
                    ColorString::centered_full_line(
                        "\nDo not turn off your 3DS or remove the SD card.",
                        Screen::Top,
                        COLOR_BACKGROUND,
                    ),
                );
            }
            JobStatus::Done => {
                if self.verifying {
                    printer.println(
                        Screen::Top,
                        ColorString::centered_full_line(
                            "Downloaded, verifying SHA256...\n\
                             Do not turn off your 3DS.",
                            Screen::Top,
                            COLOR_BACKGROUND,
                        ),
                    );
                } else {
                    printer.println(
                        Screen::Top,
                        ColorString::centered_full_line("Done!", Screen::Top, COLOR_BACKGROUND)
                            .with_fg_color(COLOR_GREEN),
                    );
                    printer.println(
                        Screen::Top,
                        ColorString::centered_full_line(
                            &format!(
                                "{TARGET_FILE} installed\n\
                                 \n\
                                 Press A to reboot.\n\
                                 Press B to return to the menu"
                            ),
                            Screen::Top,
                            COLOR_BACKGROUND,
                        ),
                    );
                }
            }
            JobStatus::Failed(error) => {
                let trace_color = [145, 115, 115];
                let details = error.split(": ").collect::<Vec<_>>().join("\n");

                printer.println(
                    Screen::Top,
                    ColorString::centered_full_line("Failed!", Screen::Top, COLOR_BACKGROUND)
                        .with_fg_color(COLOR_RED),
                );
                printer.println(
                    Screen::Top,
                    ColorString::centered_full_line(
                        &format!(
                            "\n\nError details:\n\
                             {details}"
                        ),
                        Screen::Top,
                        COLOR_BACKGROUND,
                    )
                    .with_fg_color(trace_color),
                );
                printer.println(
                    Screen::Top,
                    ColorString::centered_full_line(
                        "\n\nPress A to retry.",
                        Screen::Top,
                        COLOR_BACKGROUND,
                    ),
                );
            }
        }
        if self.job.is_running() {
            printer.goto_last_line(Screen::Top);
            printer.println(
                Screen::Top,
                ColorString::centered_full_line(
                    "Press B to cancel.",
                    Screen::Top,
                    COLOR_BACKGROUND,
                ),
            );
        }

        printer.clear_with_background(Screen::Bottom, COLOR_BACKGROUND);

        if !self.job.is_running() {
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
    }

    fn handle_input(&mut self, hid: &Hid) -> Message {
        if self.verifying {
            return Message::None;
        }

        let input = hid.keys_down();
        if input.contains(KeyPad::START) && !self.job.is_running() {
            return Message::Exit;
        }

        if input.contains(KeyPad::B) && self.job.is_running() {
            self.job.cancel();
            return Message::NeedRedraw;
        }

        if input.contains(KeyPad::B) && !self.job.is_running() {
            return Message::Goto(Route::Home(HomePage::new()), String::new());
        }

        if input.contains(KeyPad::A)
            && (self.job.status().is_idle() || self.job.status().is_failed())
        {
            self.job.start();
            return Message::NeedRedraw;
        }

        if input.contains(KeyPad::A) && self.job.status().is_done() {
            power::reboot(); // Well, no need to return a message here, the system will reboot anyway
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
            self.verifying = true;
            message = Message::NeedRedraw;
        } else if self.verifying && matches!(self.job.status(), JobStatus::Done) {
            match verify_file_sha256(STAGED_FILE, HASH_URL) {
                Ok(_) => {
                    move_file(TARGET_FILE, BACKUP_FILE).ok();
                    if let Err(error) = move_file(STAGED_FILE, TARGET_FILE) {
                        self.job.fail(error.to_string());
                    }
                }
                Err(error) => {
                    delete_file(STAGED_FILE).ok();
                    self.job.fail(error);
                }
            }
            self.verifying = false;
            message = Message::NeedRedraw;
        }
        message
    }

    fn blocks_home(&self) -> bool {
        self.job.is_running() || self.verifying
    }
}

impl DownloadPresence3DSPage {
    pub fn new() -> Self {
        Self {
            job: DownloadJob::new(URL, STAGED_FILE),
            sd_card: SdCardInfo::query(),
            verifying: false,
        }
    }
}
