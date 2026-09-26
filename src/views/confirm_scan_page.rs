use ctru::services::hid::{Hid, KeyPad};

use crate::model::{Message, Printer, Screen};
use crate::utils::constant::COLOR_BACKGROUND;
use crate::views::{HomePage, Page, Route};

pub struct ConfirmScanPage {
    content: String,
}

impl Page for ConfirmScanPage {
    fn render(&self, printer: &mut Printer<'_>) {
        printer.clear_with_background(Screen::Top, COLOR_BACKGROUND);
        printer.println(Screen::Top, "Scanned QR code:");
        printer.println(Screen::Top, self.content.as_str());

        printer.clear_with_background(Screen::Bottom, COLOR_BACKGROUND);
        printer.println(Screen::Bottom, "B: back to home | START: exit");
    }

    fn handle_input(&mut self, hid: &Hid) -> Message {
        let input = hid.keys_down();
        if input.contains(KeyPad::START) {
            return Message::Exit;
        }
        if input.contains(KeyPad::B) {
            return Message::Goto(Route::Home(HomePage::new()), String::new());
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
        }
    }
}
