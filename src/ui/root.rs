use iced::Element;
use iced::widget::{button, column, text};

#[derive(Default, Debug)]
pub struct Application {}

#[derive(Clone, Debug)]
pub enum Message {
    Crash,
}

impl Application {
    pub fn update(&mut self, _message: Message) {
        match _message {
            Message::Crash => panic!("Crash!"),
        }
    }

    pub fn view(&self) -> Element<'_, Message> {
        column![
            text("Hello World!"),
            text("你好世界！"),
            text("こんにちは世界！"),
            text("Español ¡Hola Mundo!"),
            text("Здравствуй, мир!"),
            text("🤤❗🤮🤗🎶😜💖"),
            button("Don't touch me!").on_press(Message::Crash)
        ]
        .into()
    }
}
