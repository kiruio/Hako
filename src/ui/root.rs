use iced::{
    Color, Element, Task, Theme,
    theme::Style,
    window::{self, Direction},
};

#[derive(Default, Debug)]
pub struct Application {
    content: String,
}

#[derive(Clone, Debug)]
pub enum Message {
    Close,
    Drag,
    Minimize,
    Resize(window::Direction),

    ContentChanged(String),
}

impl Application {
    pub fn new() -> (Self, Task<Message>) {
        (Self::default(), Task::none())
    }

    pub fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::Close => window::latest().and_then(window::close),
            Message::Drag => window::latest().and_then(window::drag),
            Message::Minimize => window::latest().and_then(|id| window::minimize(id, true)),
            Message::Resize(direction) => {
                window::latest().and_then(move |id| window::drag_resize(id, direction))
            }

            Message::ContentChanged(content) => {
                self.content = content;
                Task::none()
            }
        }
    }

    pub fn view(&self) -> Element<'_, Message> {
        use iced::{
            Background, Length, Padding, border,
            mouse::Interaction,
            widget::{Space, button, column, container, mouse_area, row, text, text_input},
        };

        let navbar = row![
            text("Hako").size(24),
            Space::with_width(Length::Fill),
            row![button("启动"), button("下载"), button("设置"),].height(40),
            Space::with_width(Length::Fill),
            row![
                button("_").on_press(Message::Minimize),
                button("×").on_press(Message::Close),
            ]
        ]
        .align_y(iced::Alignment::Center)
        .padding(Padding::from([5, 12]))
        .height(50);

        let content = container(column![
            mouse_area(navbar).on_press(Message::Drag),
            column![
                text("English"),
                text("Deutsch"),
                text("Français"),
                text("Italiano"),
                text("中文"),
                text("日本語"),
                text("한국어"),
                text_input("Type something here...", &self.content)
                    .on_input(Message::ContentChanged)
                    .size(20)
            ]
            .height(Length::Fill)
        ])
        .style(|_theme: &Theme| container::Style {
            background: Some(Background::Color(_theme.palette().background)),
            border: border::Border {
                width: 0.0,
                radius: 8.0.into(),
                color: Color::TRANSPARENT,
            },
            ..Default::default()
        })
        .clip(true)
        .width(Length::Fill)
        .height(Length::Fill);

        let resize_handle = |direction| {
            let interaction = match direction {
                Direction::NorthWest | Direction::SouthEast => Interaction::ResizingDiagonallyDown,
                Direction::NorthEast | Direction::SouthWest => Interaction::ResizingDiagonallyUp,
                _ => Interaction::default(),
            };
            mouse_area(Space::with_width(Length::Fixed(4.)).height(Length::Fixed(4.)))
                .interaction(interaction)
                .on_press(Message::Resize(direction))
        };

        let resize_fill_handle = |direction| {
            let (width, height) = match direction {
                Direction::East | Direction::West => (Length::Fixed(4.0), Length::Fill),
                _ => (Length::Fill, Length::Fixed(4.0)),
            };
            mouse_area(Space::with_width(width).height(height))
                .interaction(match direction {
                    Direction::North | Direction::South => Interaction::ResizingVertically,
                    Direction::East | Direction::West => Interaction::ResizingHorizontally,
                    _ => Interaction::default(),
                })
                .on_press(Message::Resize(direction))
        };

        column![
            row![
                resize_handle(Direction::NorthWest),
                resize_fill_handle(Direction::North),
                resize_handle(Direction::NorthEast),
            ],
            row![
                resize_fill_handle(Direction::West),
                content,
                resize_fill_handle(Direction::East),
            ],
            row![
                resize_handle(Direction::SouthWest),
                resize_fill_handle(Direction::South),
                resize_handle(Direction::SouthEast),
            ],
        ]
        .into()
    }

    pub fn style(&self, theme: &Theme) -> Style {
        Style {
            background_color: Color::TRANSPARENT,
            text_color: theme.palette().text,
        }
    }
}
