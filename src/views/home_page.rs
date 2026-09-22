use crate::model::{Message, Printer, Screen};
use crate::utils::ColorString;
use crate::views::Route;
use crate::views::{DownloadExamplePage, Page};
use ctru::services::hid::{Hid, KeyPad};

struct RouteName {
    name: &'static str,
    route: fn() -> Route,
}

pub struct HomePage {
    current_selection: usize,
    router: Box<[RouteName]>,
}

impl Page for HomePage {
    fn render(&self, printer: &mut Printer<'_>) {
        let default_background_color = [30, 34, 45];
        printer.clear_with_background(Screen::Top, default_background_color);
        printer.println(
            Screen::Top,
            ColorString::new("Presence 3DS Helper")
                .with_middle_position(Screen::Top)
                .color_the_entire_line(Screen::Top)
                .with_bg_color([88, 101, 242]),
        );
        printer.println(Screen::Top, "");

        for (i, route) in self.router.iter().enumerate() {
            if i == self.current_selection {
                printer.println(
                    Screen::Top,
                    ColorString::new(route.name)
                        .with_fg_color([255, 0, 0])
                        .with_bg_color(default_background_color),
                );
            } else {
                printer.println(
                    Screen::Top,
                    ColorString::new(route.name).with_bg_color(default_background_color),
                );
            }
        }
        printer.clear_with_background(Screen::Bottom, default_background_color);
        printer.println(Screen::Bottom, "Press START to exit.");
    }

    fn handle_input(&mut self, hid: &Hid) -> Message {
        let input = hid.keys_down();
        if input.contains(KeyPad::START) {
            return Message::Exit;
        }
        if input.intersects(KeyPad::UP) {
            self.current_selection = self.current_selection.saturating_sub(1);
            return Message::NeedRedraw;
        }
        if input.intersects(KeyPad::DOWN) {
            let max_index = self.router.len().saturating_sub(1);
            self.current_selection = (self.current_selection + 1).min(max_index);
            return Message::NeedRedraw;
        }

        if input.contains(KeyPad::A) {
            return Message::Goto((self.router[self.current_selection].route)());
        }

        Message::None
    }
}

impl HomePage {
    pub fn new() -> Self {
        let router: [RouteName; 2] = [
            RouteName {
                name: "Download Example",
                route: || Route::DownloadExample(DownloadExamplePage::new()),
            },
            RouteName {
                name: "Second Example",
                route: || {
                    Route::SecondExample(crate::views::second_example::SecondExamplePage::new())
                },
            },
        ];
        Self {
            current_selection: 0,
            router: router.into(),
        }
    }
}
