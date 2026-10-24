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

use crate::model::Screen;
use std::fmt::{Display, Formatter, Result, Write};
use std::string::String;

use crate::utils::constant::{CHAR_PER_LINE_BOTTOM, CHAR_PER_LINE_TOP};

pub enum Attribute {
    Bold,
    Underline,
    Reverse,
}

pub struct ColorString {
    text: String,
    bg_color: Option<[u8; 3]>,
    fg_color: Option<[u8; 3]>,
    position: Option<[u8; 2]>,
    apply_with_middle_position: bool,
    on_screen: Screen,
    entire_line: bool,
    attributes: Option<Attribute>,
}

impl Display for ColorString {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        write!(f, "{}", self.build())
    }
}

impl ColorString {
    pub fn new(text: &str, screen: Screen) -> Self {
        let string_text = text.to_string();
        Self::new_from_string(string_text, screen)
    }

    pub const fn new_from_string(text: String, screen: Screen) -> Self {
        Self {
            text,
            bg_color: None,
            fg_color: None,
            position: None,
            apply_with_middle_position: false,
            on_screen: screen,
            entire_line: false,
            attributes: None,
        }
    }

    pub fn centered_full_line(text: &str, screen: Screen, bg_color: [u8; 3]) -> Self {
        Self::new(text, screen)
            .with_bg_color(bg_color)
            .with_middle_position()
            .color_the_entire_line()
    }

    pub fn with_text(mut self, text: String) -> Self {
        self.text = text;
        self
    }

    pub const fn with_bg_color(mut self, color: [u8; 3]) -> Self {
        self.bg_color = Some(color);
        self
    }

    pub const fn with_fg_color(mut self, color: [u8; 3]) -> Self {
        self.fg_color = Some(color);
        self
    }

    pub const fn with_attribute(mut self, attribute: Attribute) -> Self {
        self.attributes = Some(attribute);
        self
    }

    pub const fn with_middle_position(mut self) -> Self {
        self.apply_with_middle_position = true;
        self
    }

    const fn width(&self) -> usize {
        match self.on_screen {
            Screen::Top => CHAR_PER_LINE_TOP as usize,
            Screen::Bottom => CHAR_PER_LINE_BOTTOM as usize,
        }
    }

    fn calc_middle_position(&self, text: &str) -> usize {
        let text_length = text.chars().count();
        self.width().saturating_sub(text_length) / 2
    }

    const fn calc_end_line(&self, text_lenght: usize) -> usize {
        self.width().saturating_sub(text_lenght)
    }

    /// Must use println, print renders incorrectly.
    pub const fn color_the_entire_line(mut self) -> Self {
        self.entire_line = true;
        self
    }

    fn wrap_segment(segment: &str, width: usize) -> Vec<String> {
        let width = width.max(1);
        let chars: Vec<char> = segment.chars().collect();
        if chars.is_empty() || chars.iter().all(|&c| c == ' ') {
            return vec![segment.to_string()];
        }
        let mut lines = Vec::new();
        let mut start = 0;
        while start < chars.len() {
            while start < chars.len() && chars[start] == ' ' {
                start += 1;
            }
            if start >= chars.len() {
                break;
            }
            let remaining = chars.len() - start;
            if remaining <= width {
                lines.push(chars[start..].iter().collect());
                break;
            }
            let cut = start + width;
            lines.push(chars[start..cut].iter().collect());
            if chars[cut] == ' ' {
                start = cut + 1;
            } else {
                start = cut;
            }
        }
        if lines.is_empty() {
            lines.push(String::new());
        }
        lines
    }

    fn visual_lines(&self) -> Vec<String> {
        let width = self.width().max(1);
        let mut lines = Vec::new();
        for segment in self.text.split('\n') {
            lines.extend(Self::wrap_segment(segment, width));
        }
        lines
    }

    pub fn build(&self) -> String {
        let mut result = String::new();

        if let Some([position_x, position_y]) = self.position {
            let _ = write!(result, "\x1b[{};{}H", position_y + 1, position_x + 1);
        }

        let mut style_prefix = String::new();
        if let Some([bg_color_red, bg_color_green, bg_color_blue, ..]) = self.bg_color {
            let _ = write!(
                style_prefix,
                "\x1b[48;2;{bg_color_red};{bg_color_green};{bg_color_blue}m"
            );
        }
        if let Some([fg_color_red, fg_color_green, fg_color_blue, ..]) = self.fg_color {
            let _ = write!(
                style_prefix,
                "\x1b[38;2;{fg_color_red};{fg_color_green};{fg_color_blue}m"
            );
        }
        if let Some(attribute) = &self.attributes {
            match attribute {
                Attribute::Bold => style_prefix.push_str("\x1b[1m"),
                Attribute::Underline => style_prefix.push_str("\x1b[4m"),
                Attribute::Reverse => style_prefix.push_str("\x1b[7m"),
            }
        }

        let lines = self.visual_lines();
        for (index, line) in lines.iter().enumerate() {
            let left = if self.apply_with_middle_position {
                self.calc_middle_position(line.as_str())
            } else {
                0
            };
            let right = if self.entire_line {
                self.calc_end_line(left + line.chars().count())
            } else {
                0
            };

            if !self.entire_line && left > 0 {
                result.push_str(&" ".repeat(left));
            }
            result.push_str(&style_prefix);
            if self.entire_line && left > 0 {
                result.push_str(&" ".repeat(left));
            }
            result.push_str(line);
            if self.entire_line && right > 0 {
                result.push_str(&" ".repeat(right));
            }
            result.push_str("\x1b[0m");
            if index + 1 < lines.len() {
                result.push('\n');
            }
        }

        result
    }
}
