use iced::Element;
use iced::widget::{button, column, row, text, text_input};

#[derive(Debug, Clone)]
pub struct State {
    pub primary_hex: String,
}

#[derive(Clone, Debug)]
pub enum Message {
    PrimaryChanged(String),
    ModeAuto,
    ModeLight,
    ModeDark,
}

// just for demo :)
impl State {
    pub fn update(&mut self, msg: &Message) {
        if let Message::PrimaryChanged(hex) = msg {
            self.primary_hex = hex.clone();
        }
    }
}

impl Default for State {
    fn default() -> Self {
        Self {
            primary_hex: String::from("#5865F2"),
        }
    }
}

pub fn view<'a>(state: &State) -> Element<'a, Message> {
    let input = text_input("Primary HEX", &state.primary_hex).on_input(Message::PrimaryChanged);
    let controls = row![
        button("Auto").on_press(Message::ModeAuto),
        button("Light").on_press(Message::ModeLight),
        button("Dark").on_press(Message::ModeDark),
    ]
    .spacing(8);

    let content = column![text("Settings"), input, controls].spacing(12);

    content.into()
}
