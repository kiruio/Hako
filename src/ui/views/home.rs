use iced::widget::{button, column, row, text, text_input};
use iced::{Element, Length};

#[derive(Default, Debug, Clone)]
pub struct Home {
    pub sub: HomeSub,
    pub stack: Vec<Page>,
    pub content: String,
}

#[derive(Default, Debug, Clone, PartialEq, Eq)]
pub enum HomeSub {
    #[default]
    List,
    Stats,
}

#[derive(Debug, Clone)]
pub enum Page {
    Detail(String),
}

#[derive(Clone, Debug)]
pub enum HomeMessage {
    SwitchSub(HomeSub),
    PushDetail(String),
    Pop,
    ContentChanged(String),
}

impl Home {
    pub fn update(&mut self, message: HomeMessage) {
        match message {
            HomeMessage::SwitchSub(s) => self.sub = s,
            HomeMessage::PushDetail(t) => self.stack.push(Page::Detail(t)),
            HomeMessage::Pop => {
                self.stack.pop();
            }
            HomeMessage::ContentChanged(c) => self.content = c,
        }
    }

    pub fn view(&self) -> Element<'_, HomeMessage> {
        let side = column![
            button("List").on_press(HomeMessage::SwitchSub(HomeSub::List)),
            button("Stats").on_press(HomeMessage::SwitchSub(HomeSub::Stats)),
        ];

        let main: iced::Element<'_, HomeMessage> = if let Some(top) = self.stack.last() {
            match top {
                Page::Detail(t) => iced::widget::Column::new()
                    .push(text(format!("Detail: {} (depth {})", t, self.stack.len())))
                    .push(
                        button("Open Next")
                            .on_press(HomeMessage::PushDetail(format!("{}-next", t))),
                    )
                    .push(button("Back").on_press(HomeMessage::Pop))
                    .into(),
            }
        } else {
            match self.sub {
                HomeSub::List => column![
                    text("Stack test..."),
                    row![
                        button("Open A").on_press(HomeMessage::PushDetail("A".into())),
                        button("Open B").on_press(HomeMessage::PushDetail("B".into())),
                        button("Open C").on_press(HomeMessage::PushDetail("C".into())),
                    ],
                    text_input("Type...", &self.content)
                        .on_input(HomeMessage::ContentChanged)
                        .size(16)
                ]
                .into(),
                HomeSub::Stats => column![text("Stats View")].into(),
            }
        };

        if self.stack.is_empty() {
            row![side, main].spacing(16).height(Length::Fill).into()
        } else {
            row![main].height(Length::Fill).into()
        }
    }
}
