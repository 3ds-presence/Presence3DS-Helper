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
    on_screen: Option<Screen>,
    entire_line: bool,
    attributes: Option<Attribute>,
}

impl Display for ColorString {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        write!(f, "{}", self.build())
    }
}

impl ColorString {
    pub fn new(text: &str) -> Self {
        let string_text = text.to_string();
        Self::new_from_string(string_text)
    }

    pub const fn new_from_string(text: String) -> Self {
        Self {
            text,
            bg_color: None,
            fg_color: None,
            position: None,
            apply_with_middle_position: false,
            on_screen: None,
            entire_line: false,
            attributes: None,
        }
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

    pub const fn with_position(mut self, x: u8, y: u8) -> Self {
        self.position = Some([x, y]);
        self
    }

    pub const fn with_middle_position(mut self, screen: Screen) -> Self {
        self.apply_with_middle_position = true;
        self.on_screen = Some(screen);
        self
    }

    fn calc_middle_position(&self, text: &str) -> usize {
        let text_length = text.chars().count();
        let total_width = match self.on_screen.unwrap() {
            Screen::Top => CHAR_PER_LINE_TOP as usize,
            Screen::Bottom => CHAR_PER_LINE_BOTTOM as usize,
        };
        (total_width - text_length) / 2
    }

    const fn calc_end_line(&self, text_lenght: usize) -> usize {
        let total_width = match self.on_screen.unwrap() {
            Screen::Top => CHAR_PER_LINE_TOP as usize,
            Screen::Bottom => CHAR_PER_LINE_BOTTOM as usize,
        };
        
        total_width - text_lenght
    }

    /// Must println, print will have a bad behavior
    pub const fn color_the_entire_line(mut self, screen:Screen) -> Self {
        self.entire_line = true;
        self.on_screen = Some(screen);
        self
    }

    pub fn build(&self) -> String {
        let mut result = String::new();

        let padding = if self.apply_with_middle_position {
            let middle = self.calc_middle_position(self.text.as_str());
            " ".repeat(middle)
        } else {
            String::new()
        };

        if !self.entire_line {
            result.push_str(&padding);
        }

        if let Some([bg_color_red, bg_color_green, bg_color_blue, ..]) = self.bg_color {
            let _ = write!(
                result,
                "\x1b[48;2;{bg_color_red};{bg_color_green};{bg_color_blue}m"
            );
        }

        if let Some([fg_color_red, fg_color_green, fg_color_blue, ..]) = self.fg_color {
            let _ = write!(
                result,
                "\x1b[38;2;{fg_color_red};{fg_color_green};{fg_color_blue}m"
            );
        }

        if let Some([position_x, position_y]) = self.position {
            let _ = write!(result, "\x1b[{};{}H", position_y + 1, position_x + 1);
        }

        if let Some(attribute) = &self.attributes {
            match attribute {
                Attribute::Bold => result.push_str("\x1b[1m"),
                Attribute::Underline => result.push_str("\x1b[4m"),
                Attribute::Reverse => result.push_str("\x1b[7m"),
            }
        }

        if self.entire_line {
            result.push_str(&padding);
        }
        result.push_str(&self.text);
        if self.entire_line {
            let end_line = self.calc_end_line(padding.chars().count() + self.text.chars().count());
            result.push_str(&" ".repeat(end_line));
        }
        result.push_str("\x1b[0m");

        result
    }
}
