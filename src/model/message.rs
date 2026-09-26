use crate::views::Route;

pub enum Message {
    None,
    NeedRedraw,
    Goto(Route, String),
    Exit,
}
