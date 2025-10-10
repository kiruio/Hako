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
    pub fn view() -> Element<'static, NavbarMessage> {
        let bar = row![
            text("Hako").size(24),
            iced::widget::Space::with_width(Length::Fill),
            row![button("启动"), button("下载"), button("设置"),].height(40),
            iced::widget::Space::with_width(Length::Fill),
            row![
                button("_").on_press(NavbarMessage::Minimize),
                button("×").on_press(NavbarMessage::Close),
            ]
        ]
        .align_y(iced::Alignment::Center)
        .padding(Padding::from([5, 12]))
        .height(50);

        mouse_area(bar).on_press(NavbarMessage::DragWindow).into()
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
