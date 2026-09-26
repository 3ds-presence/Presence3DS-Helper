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
