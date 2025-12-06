use ams_tui::app_state::AppState;

use std::time::Duration;

use ams_tui::widgets::{
    chat::{Message, Side},
    chat_dashboard::{ChatDashboard, Connection},
    dashboard::DashBoard,
};
use ratatui::{
    DefaultTerminal,
    buffer::Buffer,
    layout::{Constraint, Rect},
    style::Stylize,
    widgets::{Block, Paragraph, StatefulWidget, Widget},
};
use tokio_stream::StreamExt;

use crossterm::event::{Event, KeyCode, KeyEvent, KeyModifiers, EventStream};

#[derive(Default)]
struct AppData<'a> {
    connections: Vec<Connection<'a>>,
}

impl<'a> AppData<'a> {
    pub fn test_init() -> Self {
        Self {
            connections: vec![
                Connection::new("Test Connection 0").messages(vec![
                    Message::new("Hello from Test Connection 0", Side::Left),
                    Message::new("T123.", Side::Right),
                    Message::new("T234", Side::Right),
                    Message::new("55e.", Side::Right),
                    Message::new("Th5test message.", Side::Right),
                    Message::new("This is a test message.", Side::Right),
                    Message::new("This is a 666xwmessage.", Side::Right),
                ]),
                Connection::new("Test Connection 1").messages(vec![
                    Message::new("Hello from Test Connection 1", Side::Left),
                    Message::new("This is a test message.", Side::Right),
                ]),
            ],
        }
    }
}

struct App<'a> {
    _ams: ams::Ams,
    app_data: AppData<'a>,
    app_state: AppState,
}

impl<'a> App<'a> {
    const FRAMES_PER_SECOND: u64 = 60;

    async fn new(addr: String) -> Self {
        let _ams = ams::Ams::bind(addr).await.unwrap();
        Self {
            _ams,
            app_data: AppData::test_init(),
            app_state: AppState::default(),
        }
    }

    // Renders the ever-present dashboard and returns the inner area for other widgets
    fn render_dashboard(&mut self, area: Rect, buf: &mut Buffer) -> Rect {
        let mut dashboard =
            DashBoard::new(["Chats", "Servers", "Settings"]).title("AMS - Unsecured");

        if self.app_state.display_help() {
            dashboard = dashboard.footer(
                Paragraph::new("Hide Help: 'h'")
                    .block(Block::bordered().light_green().title("Help")),
                Constraint::Length(3),
            );
        }

        let inner = dashboard.inner(area);

        dashboard.render(area, buf, self.app_state.dashboard_state_mut());
        inner
    }

    fn render_chat_dashboard(&mut self, area: Rect, buf: &mut Buffer) {
        ChatDashboard::new(self.app_data.connections.iter().cloned())
            .select(self.app_state.chat_dashboard_active_widget())
            .render(area, buf, self.app_state.chat_dashboard_state_mut())
    }

    fn render_settings(&mut self, area: Rect, buf: &mut Buffer) {
        Block::bordered().title("Settings").render(area, buf);
    }

    fn render_server_dashboard(&mut self, area: Rect, buf: &mut Buffer) {
        Block::bordered().title("Servers").render(area, buf)
    }

    fn render(&mut self, area: Rect, buf: &mut Buffer) {
        let area = self.render_dashboard(area, buf);

        match self.app_state.selected() {
            Some(1) => self.render_server_dashboard(area, buf),
            Some(2) => self.render_settings(area, buf),
            // Fallback to always render the chat dashboard
            _ => self.render_chat_dashboard(area, buf),
        }
    }

    fn process_event(&mut self, event: KeyEvent) {
        self.app_state.process_event(event);
    }

    async fn run(&mut self, mut terminal: DefaultTerminal) -> std::io::Result<()> {
        let period = Duration::from_secs_f32(1.0 / Self::FRAMES_PER_SECOND as f32);
        let mut interval = tokio::time::interval(period);
        let mut events = EventStream::new();

        loop {
            tokio::select! {
                _ = interval.tick() => { terminal.draw(|frame| {
                    let area = frame.area();
                    let buf = frame.buffer_mut();
                    self.render(area, buf);
                })?; },
                Some(Ok(Event::Key(event))) = events.next() => {
                    if event.code == KeyCode::Char('c') && event.modifiers.contains(KeyModifiers::CONTROL) {
                        return Ok(())
                    }
                    self.process_event(event);
                },
            }
        }
    }
}

#[tokio::main]
async fn main() {
    let addr = std::env::args().nth(1).unwrap_or_else(|| "9999".into());

    let terminal = ratatui::init();
    let mut app = App::new(format!("127.0.0.1:{}", addr)).await;
    let _ = app.run(terminal).await;
    ratatui::restore();
}
