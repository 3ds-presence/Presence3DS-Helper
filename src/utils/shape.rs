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

use crate::{
    model::Screen,
    utils::{ColorString, constant::COLOR_ACCENT},
};

pub fn progress_bar(percentage: u8, width: usize) -> String {
    let filled = (percentage as usize) * width / 100;
    let mut bar = String::with_capacity(width);
    bar.extend(std::iter::repeat_n('#', filled));
    bar.extend(std::iter::repeat_n('-', width - filled));
    bar
}

pub fn title_bar(screen: Screen) -> ColorString {
    ColorString::new("Presence 3DS Helper")
        .with_middle_position(screen)
        .color_the_entire_line(screen)
        .with_bg_color(COLOR_ACCENT)
}
