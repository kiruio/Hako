use iced::widget::{column, text, text_input};
use iced::{Element, Length, Task};

#[derive(Default, Debug)]
pub struct Home {
    pub content: String,
}

#[derive(Clone, Debug)]
pub enum HomeMessage {
    ContentChanged(String),
}

impl Home {
    pub fn update(&mut self, message: HomeMessage) -> Task<HomeMessage> {
        match message {
            HomeMessage::ContentChanged(content) => {
                self.content = content;
                Task::none()
            }
        }
    }

    pub fn view(&self) -> Element<'_, HomeMessage> {
        column![
            text("English"),
            text("Deutsch"),
            text("Français"),
            text("Italiano"),
            text("中文"),
            text("日本語"),
            text("한국어"),
            text_input("Type something here...", &self.content)
                .on_input(HomeMessage::ContentChanged)
                .size(20)
        ]
        .height(Length::Fill)
        .into()
    }
}
