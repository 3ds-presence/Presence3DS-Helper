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
use std::io::Write;

use crate::ui::constants::MAX_LINES;
use crate::utils::formatter::encode_cp437;
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
    current_top_line: u8,
    current_bottom_line: u8,
}

impl<'a> Printer<'a> {
    pub const fn new(top_screen: Console<'a>, bottom_screen: Console<'a>) -> Self {
        Self {
            top_screen,
            bottom_screen,
            last_used: None,
            current_top_line: 0,
            current_bottom_line: 0,
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

        let mut out = std::io::stdout();
        let _ = write!(
            out,
            "\x1b[0m\x1b[48;2;{};{};{}m\x1b[2J",
            color[0], color[1], color[2]
        );
        let _ = out.flush();

        *self.line_mut(screen) = 0;
    }

    pub const fn current_line(&self, screen: Screen) -> u8 {
        self.line(screen)
    }

    pub fn new_line(&mut self, screen: Screen, number: u8) {
        self.goto_line(screen, self.line(screen).saturating_add(number));
    }

    pub fn goto_line(&mut self, screen: Screen, line: u8) {
        let target = line.min(MAX_LINES);
        while self.line(screen) < target {
            self.println(screen, "");
            if self.line(screen) >= MAX_LINES {
                break;
            }
        }
    }

    pub fn goto_last_line(&mut self, screen: Screen) {
        self.goto_line(screen, MAX_LINES - 1);
    }

    fn do_print(&mut self, screen: Screen, text: impl Display, newline: bool) {
        if self.line(screen) >= MAX_LINES {
            return;
        }
        let formatted_initial = format!("{text}");
        let mut formatted_vec = encode_cp437(&formatted_initial);
        if newline {
            formatted_vec.push(b'\n');
        }

        self.select_screen(screen);

        let mut out = std::io::stdout();
        let _ = out.write_all(&formatted_vec);
        let _ = out.flush();

        #[allow(clippy::cast_possible_truncation)]
        let newlines = formatted_initial
            .bytes()
            .filter(|&b| b == b'\n')
            .count()
            .min(MAX_LINES as usize) as u8;
        self.advance(screen, newlines.saturating_add(u8::from(newline)));
    }

    pub fn print(&mut self, screen: Screen, text: impl Display) {
        self.do_print(screen, text, false);
    }

    pub fn println<T: Display>(&mut self, screen: Screen, text: T) {
        self.do_print(screen, text, true);
    }

    const fn line(&self, screen: Screen) -> u8 {
        match screen {
            Screen::Top => self.current_top_line,
            Screen::Bottom => self.current_bottom_line,
        }
    }

    const fn line_mut(&mut self, screen: Screen) -> &mut u8 {
        match screen {
            Screen::Top => &mut self.current_top_line,
            Screen::Bottom => &mut self.current_bottom_line,
        }
    }

    fn advance(&mut self, screen: Screen, count: u8) {
        let line = self.line_mut(screen);
        *line = line.saturating_add(count).min(MAX_LINES);
    }
}
