use crate::model::message::Message;
use crate::model::printer::Printer;
use crate::views::home_page::HomePage;
use crate::views::page::Page;
use ctru::prelude::{Console, Gfx, Hid};
use ctru::services::gfx::Swap;
use enum_dispatch::enum_dispatch;

#[enum_dispatch(Page)]
pub enum State {
    Home(HomePage),
}

pub struct App<'a> {
    state: State,
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
            state: State::Home(HomePage::new()),
            printer: Printer::new(top_screen, bottom_screen),
            hid,
        };
        new_self.swap_buffers();
        new_self.state.render(&mut new_self.printer);
        new_self.swap_buffers();
        new_self
    }

    fn swap_buffers(&mut self) {
        self.printer.top_screen.swap_buffers();
        self.printer.bottom_screen.swap_buffers();
    }

    pub fn app_loop(&mut self) -> bool {
        self.hid.scan_input();
        match self.state.handle_input(&self.hid) {
            Message::NeedRedraw => {
                self.state.render(&mut self.printer);
                self.swap_buffers();
                false
            }
            Message::Exit => true,
            Message::None => false,
        }
    }
}
