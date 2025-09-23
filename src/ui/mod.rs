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

    // TODO: 重构，update不应该全部在这里处理
    pub fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::Resize(direction) => {
                window::latest().and_then(move |id| window::drag_resize(id, direction))
            }

            Message::Home(home_msg) => {
                let _ = self.home.update(home_msg);
                Task::none()
            }

            Message::Navbar(nav) => {
                use widgets::navbar::NavbarMessage;
                match nav {
                    NavbarMessage::DragWindow => window::latest().and_then(window::drag),
                    NavbarMessage::Minimize => {
                        window::latest().and_then(|id| window::minimize(id, true))
                    }
                    NavbarMessage::Close => window::latest().and_then(window::close),
                }
            }
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
