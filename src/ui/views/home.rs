use iced::widget::{button, column, row, text, text_input};
use iced::{Element, Length};

#[derive(Default, Debug, Clone)]
pub struct State {
    pub sub: Sub,
    pub stack: Vec<Page>,
    pub content: String,
}

#[derive(Default, Debug, Clone, PartialEq, Eq)]
pub enum Sub {
    #[default]
    List,
    Stats,
}

#[derive(Debug, Clone)]
pub enum Page {
    Detail(String),
}

#[derive(Clone, Debug)]
pub enum Message {
    SwitchSub(Sub),
    PushDetail(String),
    Pop,
    ContentChanged(String),
}

pub fn update(state: &mut State, message: Message) {
    match message {
        Message::SwitchSub(s) => state.sub = s,
        Message::PushDetail(t) => state.stack.push(Page::Detail(t)),
        Message::Pop => {
            state.stack.pop();
        }
        Message::ContentChanged(c) => state.content = c,
    }
}

pub fn view(state: &State) -> Element<'_, Message> {
    let side = column![
        button("List").on_press(Message::SwitchSub(Sub::List)),
        button("Stats").on_press(Message::SwitchSub(Sub::Stats)),
    ];

    let main: iced::Element<'_, Message> = if let Some(top) = state.stack.last() {
        match top {
            Page::Detail(t) => iced::widget::Column::new()
                .push(text(format!("Detail: {} (depth {})", t, state.stack.len())))
                .push(button("Open Next").on_press(Message::PushDetail(format!("{}-next", t))))
                .push(button("Back").on_press(Message::Pop))
                .into(),
        }
    } else {
        match state.sub {
            Sub::List => column![
                text("Stack test..."),
                row![
                    button("Open A").on_press(Message::PushDetail("A".into())),
                    button("Open B").on_press(Message::PushDetail("B".into())),
                    button("Open C").on_press(Message::PushDetail("C".into())),
                ],
                text_input("Type...", &state.content)
                    .on_input(Message::ContentChanged)
                    .size(16),
            ]
            .into(),
            Sub::Stats => column![text("Stats View")].into(),
        }
    };

    if state.stack.is_empty() {
        row![side, main].spacing(16).height(Length::Fill).into()
    } else {
        row![main].height(Length::Fill).into()
    }
}
