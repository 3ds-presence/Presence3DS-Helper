use ctru::services::hid::{Hid, KeyPad};

use crate::model::{Message, Printer, Screen};
use crate::utils::constant::{COLOR_BACKGROUND, COLOR_GREEN, COLOR_RED};
use crate::utils::sd_file::{ensure_dir, write_file};
use crate::utils::ColorString;
use crate::views::{HomePage, Page, Route};

pub struct ConfirmScanPage {
    content: String,
    status: Option<Result<(), String>>,
}

impl Page for ConfirmScanPage {
    fn render(&self, printer: &mut Printer<'_>) {
        printer.clear_with_background(Screen::Top, COLOR_BACKGROUND);
        printer.println(Screen::Top, "Scanned QR code:");
        printer.println(Screen::Top, self.content.as_str());

        printer.clear_with_background(Screen::Bottom, COLOR_BACKGROUND);
        match &self.status {
            Some(Ok(())) => printer.println(
                Screen::Bottom,
                ColorString::new("Saved to sdmc:/presence3ds/discord_rpc.conf")
                    .with_fg_color(COLOR_GREEN),
            ),
            Some(Err(error)) => printer.println(
                Screen::Bottom,
                ColorString::new(error).with_fg_color(COLOR_RED),
            ),
            None => printer.println(Screen::Bottom, "A: save | B: back to home | START: exit"),
        }
    }

    fn handle_input(&mut self, hid: &Hid) -> Message {
        let input = hid.keys_down();
        if input.contains(KeyPad::START) {
            return Message::Exit;
        }
        if input.contains(KeyPad::B) {
            return Message::Goto(Route::Home(HomePage::new()), String::new());
        }
        if input.contains(KeyPad::A) {
            self.status = Some(self.save());
            return Message::NeedRedraw;
        }

        Message::None
    }

    fn on_goto(&mut self, payload: String) {
        self.content = payload;
    }
}

impl ConfirmScanPage {
    pub const fn new() -> Self {
        Self {
            content: String::new(),
            status: None,
        }
    }

    fn save(&self) -> Result<(), String> {
        ensure_dir("/presence3ds").map_err(|error| error.to_string())?;
        write_file("/presence3ds/discord_rpc.conf", self.content.as_bytes()).map_err(|error| error.to_string())
    }
}
