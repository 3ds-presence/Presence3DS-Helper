use crate::app::State;
use crate::model::message::Message;
use crate::model::printer::Printer;
use crate::views::home_page::HomePage;
use ctru::services::hid::Hid;
use enum_dispatch::enum_dispatch;

#[enum_dispatch]
pub trait Page {
    fn render(&self, print: &mut Printer<'_>);
    fn handle_input(&mut self, hid: &Hid) -> Message;
}
