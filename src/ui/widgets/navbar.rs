use iced::widget::{button, mouse_area, row, text};
use iced::{Element, Length, Padding};

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
}
