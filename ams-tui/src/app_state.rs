use crossterm::event::{KeyCode, KeyEvent};

mod chat_dashboard;
mod dashboard;

#[derive(Default)]
enum ActiveWidget {
    #[default]
    Dashboard,
    ChatDashboard,
}

impl ActiveWidget {
    pub fn from_idx(idx: Option<usize>) -> Option<Self> {
        match idx {
            Some(0) => Some(ActiveWidget::ChatDashboard),
            _ => None,
        }
    }
}

pub struct AppState {
    display_help: bool,
    active_widget: ActiveWidget,
    chat_dashboard: chat_dashboard::ChatDashboardState,
    dashboard: dashboard::DashboardState,
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            display_help: true,
            active_widget: Default::default(),
            chat_dashboard: Default::default(),
            dashboard: Default::default(),
        }
    }
}

impl AppState {
    pub fn toggle_help(&mut self) {
        self.display_help = !self.display_help;
    }

    pub fn display_help(&self) -> bool {
        self.display_help
    }

    pub fn selected(&self) -> Option<usize> {
        self.dashboard.selected()
    }

    pub fn chat_dashboard_state_mut(
        &mut self,
    ) -> &mut crate::widgets::chat_dashboard::ChatDashboardState {
        self.chat_dashboard.state_mut()
    }

    pub fn chat_dashboard_active_widget(&self) -> Option<crate::widgets::chat_dashboard::Active> {
        self.chat_dashboard.active_widget()
    }

    pub fn dashboard_state_mut(&mut self) -> &mut crate::widgets::dashboard::DashBoardState {
        self.dashboard.state_mut()
    }

    pub fn process_event(&mut self, input: KeyEvent) {
        // Toggle the help dashboard from anywhere, even though it is a part of the dashboard widget.
        if input.code == KeyCode::Char('h') {
            self.toggle_help();
            return;
        }

        let active_widget = match self.active_widget {
            ActiveWidget::Dashboard => self.dashboard.process_event(input),
            ActiveWidget::ChatDashboard => self.chat_dashboard.process_event(input),
        };

        if let Some(widget) = active_widget {
            match widget {
                ActiveWidget::Dashboard => {
                    self.dashboard.activate();
                    self.active_widget = ActiveWidget::Dashboard;
                }
                ActiveWidget::ChatDashboard => {
                    self.chat_dashboard.activate();
                    self.active_widget = ActiveWidget::ChatDashboard;
                }
            }
        }
    }
}
