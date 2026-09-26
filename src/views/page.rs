use crate::model::{Message, Printer};
use ctru::services::hid::Hid;
use enum_dispatch::enum_dispatch;

#[enum_dispatch]
pub trait Page {
    fn render(&self, print: &mut Printer<'_>);
    fn handle_input(&mut self, hid: &Hid) -> Message;
    fn update(&mut self) -> Message {
        Message::None
    }

    fn draws_top_screen_manually(&self) -> bool {
        false
    }

    fn blocks_home(&self) -> bool {
        false
    }
}
