use ctru::services::hid::{Hid, KeyPad};
use ctru_sys::svcGetSystemTick;

use crate::model::{Message, Printer, Screen};
use crate::utils::camera::blit_to_top_screen;
use crate::utils::constant::{COLOR_BACKGROUND, COLOR_RED};
use crate::utils::{Camera, ColorString};
use crate::views::{ConfirmScanPage, HomePage, Page, Route};

const SCAN_INTERVAL_TICKS: u64 = 268_111_856 * 3 / 10;

pub struct ImportConfigPage {
    camera: Result<Camera, String>,
    next_scan_tick: u64,
}

impl Page for ImportConfigPage {
    fn render(&self, printer: &mut Printer<'_>) {
        printer.clear_with_background(Screen::Bottom, COLOR_BACKGROUND);
        match &self.camera {
            Ok(_) => printer.println(Screen::Bottom, "Point the QR code at the camera."),
            Err(error) => printer.println(
                Screen::Bottom,
                ColorString::new(error).with_fg_color(COLOR_RED),
            ),
        }
        printer.println(Screen::Bottom, "B: back | START: exit");
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

    fn update(&mut self) -> Message {
        let Ok(camera) = &mut self.camera else {
            return Message::None;
        };

        if camera.poll() {
            let (width, height) = camera.size();
            blit_to_top_screen(camera.frame(), width, height);

            let now = unsafe { svcGetSystemTick() };
            if now >= self.next_scan_tick {
                self.next_scan_tick = now + SCAN_INTERVAL_TICKS;
                if let Some(content) = camera.scan_qr() {
                    return Message::Goto(Route::ConfirmScan(ConfirmScanPage::new()), content);
                }
            }
        }

        Message::None
    }

    fn draws_top_screen_manually(&self) -> bool {
        true
    }
}

impl ImportConfigPage {
    pub fn new() -> Self {
        Self {
            camera: Camera::new().map_err(|error| error.to_string()),
            next_scan_tick: 0,
        }
    }
}
