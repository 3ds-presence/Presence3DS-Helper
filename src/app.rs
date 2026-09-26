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

use crate::model::message::Message;
use crate::model::printer::Printer;
use crate::views::Route;
use crate::views::home_page::HomePage;
use crate::views::page::Page;
use ctru::prelude::{Console, Gfx, Hid};
use ctru::services::gfx::Swap;

pub struct App<'a> {
    state: Route,
    printer: Printer<'a>,
    hid: Hid,
}

impl<'a> App<'a> {
    pub fn new(gfx: &'a Gfx) -> Self {
        let hid = Hid::new().unwrap();
        let mut top_screen = Console::new(gfx.top_screen.borrow_mut());
        top_screen.set_double_buffering(true);
        let mut bottom_screen = Console::new(gfx.bottom_screen.borrow_mut());
        bottom_screen.set_double_buffering(true);
        let mut new_self = Self {
            state: Route::Home(HomePage::new()),
            printer: Printer::new(top_screen, bottom_screen),
            hid,
        };
        new_self.swap_buffers();
        new_self.state.render(&mut new_self.printer);
        new_self.swap_buffers();
        new_self
    }

    fn swap_buffers(&mut self) {
        if !self.state.draws_top_screen_manually() {
            self.printer.top_screen.swap_buffers();
        }
        self.printer.bottom_screen.swap_buffers();
    }

    fn render(&mut self) {
        self.state.render(&mut self.printer);
        self.swap_buffers();
    }

    fn process_message(&mut self, message: Message) -> bool {
        match message {
            Message::NeedRedraw => {
                self.render();
                false
            }
            Message::Goto(route, payload) => {
                self.state = route;
                self.state.on_goto(payload);
                self.swap_buffers();
                self.render();
                false
            }
            Message::Exit => true,
            Message::None => false,
        }
    }

    pub fn blocks_home(&self) -> bool {
        self.state.blocks_home()
    }

    pub fn app_loop(&mut self) -> bool {
        self.hid.scan_input();

        let message = self.state.handle_input(&self.hid);
        if self.process_message(message) {
            return true;
        }

        let message = self.state.update();
        self.process_message(message)
    }
}
