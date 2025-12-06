use crate::widgets::chat_dashboard::{Active, ChatDashboardState as WidgetState};
use crossterm::event::{KeyCode, KeyEvent};

#[derive(Default)]
pub struct ChatDashboardState {
    /// Which part of the dashboard is currently active. This determines how input is handled.
    active: Option<Active>,
    state: WidgetState,
}

impl ChatDashboardState {
    pub fn state_mut(&mut self) -> &mut WidgetState {
        &mut self.state
    }

    pub fn active_widget(&self) -> Option<Active> {
        self.active
    }

    pub fn process_event(&mut self, input: KeyEvent) -> Option<super::ActiveWidget> {
        if input.code == KeyCode::Esc {
            *self = ChatDashboardState::default();
            return Some(super::ActiveWidget::Dashboard);
        }
        match self.active {
            Some(Active::Connections) => match input.code {
                KeyCode::Up => {
                    self.state.select_previous_connection();
                }
                KeyCode::Down => {
                    self.state.select_next_connection();
                }
                KeyCode::Right => {
                    self.active = Some(Active::Input);
                }
                _ => {}
            },
            Some(Active::Chat) => match input.code {
                KeyCode::Up => {
                    self.state.scroll_messages_up();
                }
                KeyCode::Down => {
                    if self.state.first_visible_message() == 0 {
                        self.active = Some(Active::Input)
                    } else {
                        self.state.scroll_messages_down();
                    }
                }
                KeyCode::Left => {
                    self.active = Some(Active::Connections);
                }
                _ => {}
            },
            Some(Active::Input) => match input.code {
                KeyCode::Char(c) => {
                    self.state.push_input(c);
                }
                KeyCode::Backspace => {
                    self.state.pop_input();
                }
                KeyCode::Left => self.active = Some(Active::Connections),
                KeyCode::Enter => {
                    let _ = self.state.input();
                    self.state.clear_input();
                }
                KeyCode::Up => {
                    self.active = Some(Active::Chat);
                }
                _ => {}
            },
            None => {}
        }
        None
    }

    pub fn activate(&mut self) {
        self.state.select_next_connection();
        self.active = Some(Active::Connections)
    }
}
