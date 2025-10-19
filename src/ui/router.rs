use crate::ui::views::{
    home::{self, Message as HomeMessage, State as HomeState},
    settings,
};
use iced::Element;

#[derive(Copy, Clone, Debug, PartialEq, Eq, Default)]
pub enum TopTab {
    #[default]
    Home,
    Settings,
}

#[derive(Debug, Clone)]
pub enum Message {
    SwitchTop(TopTab),
    Home(HomeMessage),
    Settings(settings::Message),
}

#[derive(Debug, Clone)]
pub struct NavbarState {
    pub stack_active: bool,
    pub title: Option<String>,
}

#[derive(Debug, Default)]
pub struct Router {
    pub top: TopTab,
    pub home: HomeState,
    pub settings: settings::State,
}

impl Router {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn update(&mut self, msg: Message) {
        match msg {
            Message::SwitchTop(t) => self.top = t,
            Message::Home(m) => home::update(&mut self.home, m),
            Message::Settings(m) => self.settings.update(&m),
        }
    }

    pub fn view<'a>(&'a self) -> Element<'a, Message> {
        match self.top {
            TopTab::Home => home::view(&self.home).map(Message::Home),
            TopTab::Settings => settings::view(&self.settings).map(Message::Settings),
        }
    }

    pub fn navbar_state(&self) -> NavbarState {
        let stack_active = !self.home.stack.is_empty();
        let title = self.home.stack.last().map(|p| match p {
            crate::ui::views::home::Page::Detail(t) => t.clone(),
        });

        NavbarState {
            stack_active,
            title,
        }
    }
}
