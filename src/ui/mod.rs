pub mod config;
pub mod router;
pub mod theme;
pub mod views;
pub mod widgets;

use crate::ui::router::{Message as RouterMessage, Router};
use crate::ui::theme::{ThemeConfig, ThemeMode, make_theme};
use iced::{
    Color, Element, Subscription, Task, Theme,
    theme::{Mode, Style},
    window,
};
use window::Direction;

#[derive(Debug)]
pub struct Application {
    router: Router,
    theme_config: ThemeConfig,
    system_mode: Mode,
}

#[derive(Clone, Debug)]
pub enum Message {
    Resize(Direction),
    Router(RouterMessage),
    Navbar(widgets::navbar::NavbarMessage),
    SystemThemeChanged(Mode),
}

impl Application {
    pub fn new() -> (Self, Task<Message>) {
        let app = Self {
            router: Router::new(),
            theme_config: ThemeConfig::default(),
            system_mode: Mode::None,
        };
        let task = iced::system::theme().map(Message::SystemThemeChanged);
        (app, task)
    }

    pub fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::Resize(direction) => widgets::window_frame::WindowFrame::command(direction),
            Message::Router(msg) => {
                if let crate::ui::router::Message::Settings(sm) = &msg {
                    match sm {
                        crate::ui::views::settings::Message::PrimaryChanged(hex) => {
                            self.theme_config.primary_hex = hex.clone();
                        }
                        crate::ui::views::settings::Message::ModeAuto => {
                            self.theme_config.mode = ThemeMode::Auto;
                        }
                        crate::ui::views::settings::Message::ModeLight => {
                            self.theme_config.mode = ThemeMode::Light;
                        }
                        crate::ui::views::settings::Message::ModeDark => {
                            self.theme_config.mode = ThemeMode::Dark;
                        }
                    }
                }
                self.router.update(msg);
                Task::none()
            }
            Message::Navbar(msg) => widgets::navbar::Navbar::command(msg),
            Message::SystemThemeChanged(mode) => {
                self.system_mode = mode;
                Task::none()
            }
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

    pub fn theme(&self) -> Theme {
        make_theme(&self.theme_config, self.system_mode)
    }

    pub fn subscription(&self) -> Subscription<Message> {
        iced::system::theme_changes().map(Message::SystemThemeChanged)
    }
}
