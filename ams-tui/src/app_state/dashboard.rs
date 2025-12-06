use crate::widgets::dashboard::DashBoardState as WidgetState;
use crossterm::event::{KeyCode, KeyEvent};
pub struct DashboardState {
    state: WidgetState,
}

impl Default for DashboardState {
    fn default() -> Self {
        let mut state = WidgetState::default();
        state.select_first();
        Self { state }
    }
}

impl DashboardState {
    pub fn state_mut(&mut self) -> &mut WidgetState {
        &mut self.state
    }

    pub fn selected(&self) -> Option<usize> {
        self.state.selected()
    }

    pub fn process_event(&mut self, input: KeyEvent) -> Option<super::ActiveWidget> {
        match input.code {
            KeyCode::Left => {
                self.state.select_previous();
            }
            KeyCode::Right => {
                self.state.select_next();
            }
            KeyCode::Down => {
                return super::ActiveWidget::from_idx(self.state.selected());
            }
            _ => {}
        }
        None
    }

    pub fn activate(&mut self) {}
}
