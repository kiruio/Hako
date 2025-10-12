use iced::widget::{button, mouse_area, row, text};
use iced::window;
use iced::{Element, Length, Padding, Task};

#[derive(Clone, Debug)]
pub enum NavbarMessage {
    DragWindow,
    Minimize,
    Close,
}

pub struct Navbar;

impl Navbar {
    pub fn view(state: crate::ui::router::NavbarState) -> Element<'static, crate::ui::Message> {
        let title = state.title.clone().unwrap_or_default();

        let header: iced::Element<'_, crate::ui::Message> = if state.stack_active {
            row![
                button("<").on_press(crate::ui::Message::Router(
                    crate::ui::router::Message::Home(crate::ui::views::home::Message::Pop),
                )),
                text(title),
            ]
            .into()
        } else {
            text("Hako").size(24).into()
        };

        let navigation: iced::Element<'_, crate::ui::Message> = if state.stack_active {
            iced::widget::Space::with_width(Length::Fill).into()
        } else {
            row![
                button("Home").on_press(crate::ui::Message::Router(
                    crate::ui::router::Message::SwitchTop(crate::ui::router::TopTab::Home),
                )),
                button("Settings").on_press(crate::ui::Message::Router(
                    crate::ui::router::Message::SwitchTop(crate::ui::router::TopTab::Settings),
                )),
            ]
            .into()
        };

        let controls = row![
            button("_").on_press(crate::ui::Message::Navbar(NavbarMessage::Minimize)),
            button("×").on_press(crate::ui::Message::Navbar(NavbarMessage::Close)),
        ];

        mouse_area(
            row![
                header,
                iced::widget::Space::with_width(Length::Fill),
                navigation,
                iced::widget::Space::with_width(Length::Fill),
                controls
            ]
            .align_y(iced::Alignment::Center)
            .padding(Padding::from([5, 12]))
            .height(50),
        )
        .on_press(crate::ui::Message::Navbar(NavbarMessage::DragWindow))
        .into()
    }

    pub fn command(msg: NavbarMessage) -> Task<crate::ui::Message> {
        use crate::ui::Message;
        match msg {
            NavbarMessage::DragWindow => window::latest()
                .and_then(window::drag)
                .map(|_: Option<()>| Message::Navbar(NavbarMessage::DragWindow)),
            NavbarMessage::Minimize => window::latest()
                .and_then(|id| window::minimize(id, true))
                .map(|_: Option<()>| Message::Navbar(NavbarMessage::Minimize)),
            NavbarMessage::Close => window::latest()
                .and_then(window::close)
                .map(|_: Option<()>| Message::Navbar(NavbarMessage::Close)),
        }
    }
}
