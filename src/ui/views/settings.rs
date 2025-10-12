use iced::widget::{column, text};
use iced::{Element, Length};

#[derive(Default, Debug, Clone)]
pub struct Settings;

impl Settings {
    pub fn view<'a>() -> Element<'a, crate::ui::router::Message> {
        column![text("Settings")].height(Length::Fill).into()
    }
}
