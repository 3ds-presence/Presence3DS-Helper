use ctru::services::hid::{Hid, KeyPad};

use crate::model::{Message, Printer, Screen};
use crate::utils::color_string::ColorString;
use crate::views::{HomePage, Page, Route};

pub struct SecondExamplePage {
    counter: u32,
}

impl Page for SecondExamplePage {
    fn render(&self, printer: &mut Printer<'_>) {
        printer.clear_with_background(Screen::Top, [30, 34, 45]);
        printer.println(Screen::Top, "Second example");
        printer.println(
            Screen::Top,
            ColorString::new(&format!("Counter: {}", self.counter)).with_fg_color([255, 0, 0]),
        );
        printer.clear_with_background(Screen::Bottom, [30, 34, 45]);
        printer.println(Screen::Bottom, "Press A to increment | START: exit");
    }

    fn handle_input(&mut self, hid: &Hid) -> Message {
        let input = hid.keys_down();
        if input.contains(KeyPad::START) {
            return Message::Exit;
        }

        if input.contains(KeyPad::B) {
            return Message::Goto(Route::Home(HomePage::new()));
        }

        if input.contains(KeyPad::A) {
            self.counter += 1;
            return Message::NeedRedraw;
        }
        Message::None
    }
}

impl SecondExamplePage {
    pub const fn new() -> Self {
        Self { counter: 0 }
    }
}
