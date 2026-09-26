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

use std::fmt::Display;

use ctru::console::Console;

#[derive(Copy, Clone, PartialEq, Eq)]
pub enum Screen {
    Top,
    Bottom,
}

pub struct Printer<'a> {
    pub top_screen: Console<'a>,
    pub bottom_screen: Console<'a>,
    last_used: Option<Screen>,
}

impl<'a> Printer<'a> {
    pub const fn new(top_screen: Console<'a>, bottom_screen: Console<'a>) -> Self {
        Self {
            top_screen,
            bottom_screen,
            last_used: None,
        }
    }

    fn select_screen(&mut self, screen: Screen) {
        match screen {
            Screen::Top => {
                if self.last_used != Some(screen) {
                    self.top_screen.select();
                }
            }
            Screen::Bottom => {
                if self.last_used != Some(screen) {
                    self.bottom_screen.select();
                }
            }
        }
        self.last_used = Some(screen);
    }

    pub fn clear_with_background(&mut self, screen: Screen, color: [u8; 3]) {
        self.select_screen(screen);
        print!("\x1b[0m\x1b[48;2;{};{};{}m", color[0], color[1], color[2]);
        print!("\x1b[2J");
    }
    pub fn print<T: Display>(&mut self, screen: Screen, text: T) {
        self.select_screen(screen);
        print!("{text}");
    }

    pub fn println<T: Display>(&mut self, screen: Screen, text: T) {
        self.select_screen(screen);
        println!("{text}");
    }
}
