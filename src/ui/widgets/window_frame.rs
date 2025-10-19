use iced::mouse::Interaction;
use iced::widget::{Space, column, container, mouse_area, row};
use iced::window::Direction;
use iced::{Background, border};
use iced::{Element, Length, Task, Theme};

#[derive(Debug)]
pub struct WindowFrame;

impl WindowFrame {
    pub fn view<'a, Message: 'a + Clone>(
        content: Element<'a, Message>,
        on_resize: impl Fn(Direction) -> Message + 'static + Copy,
    ) -> Element<'a, Message> {
        let handle = |direction: Direction| -> Element<'a, Message> {
            let (width, height) = match direction {
                Direction::East | Direction::West => (Length::Fixed(4.), Length::Fill),
                Direction::North | Direction::South => (Length::Fill, Length::Fixed(4.)),
                _ => (Length::Fixed(4.), Length::Fixed(4.)),
            };

            let interaction = match direction {
                Direction::NorthWest | Direction::SouthEast => Interaction::ResizingDiagonallyDown,
                Direction::NorthEast | Direction::SouthWest => Interaction::ResizingDiagonallyUp,
                Direction::North | Direction::South => Interaction::ResizingVertically,
                Direction::East | Direction::West => Interaction::ResizingHorizontally,
            };

            mouse_area(Space::with_width(width).height(height))
                .interaction(interaction)
                .on_press(on_resize(direction))
                .into()
        };

        column![
            row![
                handle(Direction::NorthWest),
                handle(Direction::North),
                handle(Direction::NorthEast),
            ],
            row![
                handle(Direction::West),
                container(content)
                    .style(|theme: &Theme| container::Style {
                        background: Some(Background::Color(
                            theme.extended_palette().background.base.color
                        )),
                        border: border::Border {
                            width: 0.0,
                            radius: 8.0.into(),
                            color: iced::Color::TRANSPARENT,
                        },
                        ..Default::default()
                    })
                    .clip(true)
                    .width(Length::Fill)
                    .height(Length::Fill),
                handle(Direction::East),
            ],
            row![
                handle(Direction::SouthWest),
                handle(Direction::South),
                handle(Direction::SouthEast),
            ],
        ]
        .into()
    }

    pub fn command(direction: Direction) -> Task<crate::ui::Message> {
        use iced::window;
        window::latest()
            .and_then(move |id| window::drag_resize(id, direction))
            .map(move |_: Option<()>| crate::ui::Message::Resize(direction))
    }
}
