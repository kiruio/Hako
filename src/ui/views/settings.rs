use iced::widget::{column, text};
use iced::{Element, Length};

#[derive(Default, Debug, Clone)]
pub struct State {}

pub fn view<'a>(_state: &State) -> Element<'a, crate::ui::router::Message> {
    column![text("Settings")].height(Length::Fill).into()
}
