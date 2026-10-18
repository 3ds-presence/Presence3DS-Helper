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
    model::{Printer, Screen},
    utils::{
        ColorString,
        constant::{COLOR_ACCENT, COLOR_BACKGROUND, COLOR_BACKGROUND_INVERTED},
    },
};

pub fn progress_bar(percentage: u8, width: usize) -> String {
    let filled = (percentage as usize) * width / 100;
    let mut bar = String::with_capacity(width);
    bar.extend(std::iter::repeat_n('#', filled));
    bar.extend(std::iter::repeat_n('-', width - filled));
    bar
}

pub fn download_section(
    printer: &mut Printer<'_>,
    filename: &str,
    current: usize,
    total: usize,
    percentage: u8,
) {
let print_here = |printer: &mut Printer<'_>, txt: &str| {
        printer.print(Screen::Top, ColorString::new(txt).with_bg_color(COLOR_BACKGROUND));
    };

    printer.println(
        Screen::Top,
        ColorString::centered_full_line(filename, Screen::Top, COLOR_BACKGROUND),
    );

    print_here(printer, " ".repeat(10).as_str());
    print_here(printer, &format!("╔{}╗", "═".repeat(28)));
    print_here(printer, " ".repeat(10).as_str());
    printer.println(Screen::Top, "");

    print_here(printer, " ".repeat(10).as_str());
    print_here(printer, "║");
    print_here(printer, &progress_bar(percentage, 25));
    print_here(
        printer,
        format!("{:02}%", percentage.min(99)).as_str(),
    );
    print_here(printer, "║");
    print_here(printer, " ".repeat(10).as_str());
    printer.println(Screen::Top, "");

    print_here(printer, " ".repeat(10).as_str());
    print_here(printer, &format!("╚{}╝", "═".repeat(28)));
    print_here(printer, " ".repeat(10).as_str());
    printer.println(Screen::Top, "");

    if total == 0 {
        printer.println(
            Screen::Top,
            ColorString::centered_full_line(
                &format!("{current} bytes (size unknown)"),
                Screen::Top,
                COLOR_BACKGROUND,
            ),
        );
    } else {
        printer.println(
            Screen::Top,
            ColorString::centered_full_line(
                &format!("{current} / {total} bytes"),
                Screen::Top,
                COLOR_BACKGROUND,
            ),
        );
    }
}

pub fn title_bar(screen: Screen) -> ColorString {
    ColorString::centered_full_line("Presence 3DS Helper", screen, COLOR_ACCENT)
}
