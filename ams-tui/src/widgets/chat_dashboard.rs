use std::borrow::Cow;

use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout},
    style::{Style, Stylize},
    text::Line,
    widgets::{Block, BorderType, List, ListState, StatefulWidget, Widget},
};

use crate::{
    lang::{CHAT_STR, CONNECTION_INFO_STR, CONNECTIONS_STR},
    widgets::chat::{Chat, ChatState},
};

pub use crate::widgets::chat::Message;

#[derive(Clone)]
pub struct Connection<'a> {
    /// The nickname of the connection.
    name: Cow<'a, str>,
    /// The chat messages associated with this connection.
    chat: Vec<Message<'a>>,
}

#[derive(PartialEq, Clone, Copy)]
pub enum Active {
    Connections,
    Chat,
    Input,
}

impl<'a> Connection<'a> {
    /// Creates a new Connection with the given name.
    pub fn new(name: impl Into<Cow<'a, str>>) -> Self {
        Self {
            name: name.into(),
            chat: Vec::new(),
        }
    }

    /// Sets the name of the connection.
    pub fn name(self, name: Cow<'a, str>) -> Self {
        Self { name, ..self }
    }

    /// Sets the chat messages for the connection.
    ///
    /// Note: Messages are expected to be in reverse chronological order (newest first).
    pub fn messages(self, messages: Vec<Message<'a>>) -> Self {
        Self {
            chat: messages,
            ..self
        }
    }

    /// Adds a message to the connection's chat.
    pub fn add_message(&mut self, message: Message<'a>) {
        self.chat.insert(0, message);
    }
}

/// The state associated with the ChatDashboard widget.
#[derive(Default)]
pub struct ChatDashboardState {
    connections_list_state: ListState,
    chat_state: ChatState,
    chat_input: String,
}

impl ChatDashboardState {
    /// Returns the currently selected connection index.
    pub fn selected_connection(&self) -> Option<usize> {
        self.connections_list_state.selected()
    }

    /// Selects the next input.
    ///
    /// Resets the chat message selection and input.
    pub fn select_next_connection(&mut self) {
        self.connections_list_state.select_next();
        self.clear_input();
        self.chat_state.select(None);
    }

    /// Selects the previous input.
    ///
    /// Resets the chat message selection and input.
    pub fn select_previous_connection(&mut self) {
        self.connections_list_state.select_previous();
        self.clear_input();
        self.chat_state.select(None);
    }

    pub fn first_visible_message(&self) -> usize {
        self.chat_state.offset()
    }

    /// Selects the next message in the chat.
    pub fn scroll_messages_down(&mut self) {
        self.chat_state.scroll_down();
    }

    /// Selects the previous message in the chat.
    pub fn scroll_messages_up(&mut self) {
        self.chat_state.scroll_up();
    }

    /// Returns the current input string.
    pub fn input(&self) -> &str {
        &self.chat_input
    }

    // Clears the chat input.
    pub fn clear_input(&mut self) {
        self.chat_input.clear();
    }

    /// Appends a character to the current input.
    pub fn push_input(&mut self, ch: char) {
        self.chat_input.push(ch);
    }

    /// Removes the last character from the current input.
    pub fn pop_input(&mut self) {
        self.chat_input.pop();
    }
}

/// The chat dashboard widget that displays chats with connections.
///
/// The chat dashboard widget consumes a list of [Connection]s. The associate [ChatDashboardState] is responsible for
/// managing all the state associated with the widget. The widget has three main areas, all of which have some state:
///
/// 1. The connection list on the left side, with state to determine the currently selected connection.
/// 2. The chat area in the middle/right, which displays the messages for the currently selected connection. The
///    displayable messages are generally a subset of all messages for the connection, and the state manages which
///    messages are currently visible.
/// 3. The text input area at the bottom, which displays user input, which is also managed by the state.
#[derive(Default)]
pub struct ChatDashboard<'a> {
    /// The List for the available connections.
    list: Vec<Connection<'a>>,
    active: Option<Active>,
}

impl<'a> ChatDashboard<'a> {
    /// Create a new ChatDashboard with the given chats.
    pub fn new<Iter>(connections: Iter) -> Self
    where
        Iter: IntoIterator<Item = Connection<'a>>,
    {
        let list = connections.into_iter().collect();

        ChatDashboard { list, active: None }
    }

    pub fn select(mut self, selected: Option<Active>) -> Self {
        self.active = selected;
        self
    }

    /// Helper function to split the area into the connection list and the rest of the dashboard.
    fn chunks(&self, area: ratatui::layout::Rect) -> [ratatui::layout::Rect; 2] {
        Layout::new(
            Direction::Horizontal,
            [Constraint::Max(30), Constraint::Min(30)],
        )
        .areas(area)
    }

    fn block(&self, widget: Active) -> Block {
        if self.active == Some(widget) {
            Block::bordered().border_type(BorderType::Double)
        } else {
            Block::bordered()
        }
    }
}

impl<'a> StatefulWidget for ChatDashboard<'a> {
    type State = ChatDashboardState;

    fn render(
        self,
        area: ratatui::layout::Rect,
        buf: &mut ratatui::buffer::Buffer,
        state: &mut Self::State,
    ) {
        let [list_area, inner] = self.chunks(area);

        Block::bordered().title(CHAT_STR).render(area, buf);

        let list = List::new(self.list.iter().map(|conn| Line::from(conn.name.as_ref())))
            .block(self.block(Active::Connections).title(CONNECTIONS_STR))
            .highlight_style(
                Style::new()
                    .bg(ratatui::style::Color::Gray)
                    .fg(ratatui::style::Color::Black),
            );

        StatefulWidget::render(list, list_area, buf, &mut state.connections_list_state);

        let [conn_info, chat_area, text_input_area] = Layout::new(
            Direction::Vertical,
            [
                Constraint::Length(3),
                Constraint::Min(3),
                Constraint::Length(3),
            ],
        )
        .areas(inner);

        Block::bordered()
            .title(CONNECTION_INFO_STR)
            .title_alignment(Alignment::Right)
            .render(conn_info, buf);

        let block = self.block(Active::Input);
        if state.input().is_empty() {
            Line::from(" Type your message here...")
                .gray()
                .style(Style::new().dim().italic().slow_blink())
                .left_aligned()
                .render(block.inner(text_input_area), buf);
        } else {
            Line::from(format!(" {}", state.input())).render(block.inner(text_input_area), buf);
        }
        block.render(text_input_area, buf);

        let chat = if let Some(idx) = state.selected_connection() {
            Chat::new(self.list[idx].chat.iter().cloned())
        } else {
            Chat::default()
        };

        chat.block(
            self.block(Active::Chat)
                .title(CHAT_STR)
                .title_alignment(Alignment::Left),
        )
        .render(chat_area, buf, &mut state.chat_state);
    }
}
