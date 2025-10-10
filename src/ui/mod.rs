pub mod config;
pub mod views;
pub mod widgets;

use iced::{Color, Element, Task, Theme, theme::Style, window};
use window::Direction;

#[derive(Default, Debug)]
pub struct Application {
    home: views::home::Home,
}

#[derive(Clone, Debug)]
pub enum Message {
    Resize(Direction),
    Home(views::home::HomeMessage),
    Navbar(widgets::navbar::NavbarMessage),
}

impl Application {
    pub fn new() -> (Self, Task<Message>) {
        (Self::default(), Task::none())
    }

    pub fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::Resize(direction) => widgets::window_frame::WindowFrame::command(direction),
            Message::Home(msg) => self.home.update(msg).map(Message::Home),
            Message::Navbar(msg) => widgets::navbar::Navbar::command(msg),
        }
    }

    pub fn view(&self) -> Element<'_, Message> {
        use crate::ui::widgets::{navbar::Navbar, window_frame::WindowFrame};
        use iced::widget::column;

        let navbar = Navbar::view().map(Message::Navbar);
        let content = self.home.view().map(Message::Home);

        WindowFrame::view(column![navbar, content].into(), Message::Resize)
    }

    pub fn style(&self, theme: &Theme) -> Style {
        Style {
            background_color: Color::TRANSPARENT,
            text_color: theme.palette().text,
        }
    }
}
