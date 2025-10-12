pub mod config;
pub mod router;
pub mod views;
pub mod widgets;

use crate::ui::router::{Message as RouterMessage, Router};
use iced::{Color, Element, Task, Theme, theme::Style, window};
use window::Direction;

#[derive(Default, Debug)]
pub struct Application {
    router: Router,
}

#[derive(Clone, Debug)]
pub enum Message {
    Resize(Direction),
    Router(RouterMessage),
    Navbar(widgets::navbar::NavbarMessage),
}

impl Application {
    pub fn new() -> (Self, Task<Message>) {
        (
            Self {
                router: Router::new(),
            },
            Task::none(),
        )
    }

    pub fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::Resize(direction) => widgets::window_frame::WindowFrame::command(direction),
            Message::Router(msg) => {
                self.router.update(msg);
                Task::none()
            }
            Message::Navbar(msg) => widgets::navbar::Navbar::command(msg),
        }
    }

    pub fn view(&self) -> Element<'_, Message> {
        use crate::ui::widgets::{navbar::Navbar, window_frame::WindowFrame};
        use iced::widget::column;

        let nav_state = self.router.navbar_state();
        let navbar = Navbar::view(nav_state);
        let content = self.router.view().map(Message::Router);

        WindowFrame::view(column![navbar, content].into(), Message::Resize)
    }

    pub fn style(&self, theme: &Theme) -> Style {
        Style {
            background_color: Color::TRANSPARENT,
            text_color: theme.palette().text,
        }
    }
}
