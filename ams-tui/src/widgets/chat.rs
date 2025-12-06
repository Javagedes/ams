use std::borrow::Cow;

use ratatui::{
    buffer::Buffer,
    layout::Rect,
    prelude::BlockExt,
    style::{Style, Stylize},
    text::{Line, Span, Text},
    widgets::{Block, List, ListDirection, ListState, StatefulWidget, Widget},
};

/// Styling to pick the side of the chat box the message is on.
#[derive(Clone)]
pub enum Side {
    Left,
    Right,
}

/// A chat message.
#[derive(Clone)]
pub struct Message<'a> {
    content: Cow<'a, str>,
    side: Side,
}

impl<'a> Message<'a> {
    /// Creates a new message that will be displayed on the given side.
    pub fn new(content: impl Into<Cow<'a, str>>, side: Side) -> Self {
        Self {
            content: content.into(),
            side,
        }
    }

    /// Converts the message into a Text object, formatted appropriately for its side.
    fn to_text(&self) -> Text<'a> {
        match self.side {
            Side::Left => left(&self.content, 30),
            Side::Right => right(&self.content, 30),
        }
    }
}

/// The state associated with the Chat widget.
#[derive(Default)]
pub struct ChatState(ListState);

impl ChatState {
    /// Returns the newest message index visible
    pub fn offset(&self) -> usize {
        self.0.offset()
    }

    /// Selects a message by its index.
    pub fn select(&mut self, index: Option<usize>) {
        self.0.select(index);
    }

    /// Scrolls down in the message list (towards newer messages)
    pub fn scroll_down(&mut self) {
        if self.0.offset() != 0 {
            *self.0.offset_mut() -= 1;
        }
    }

    /// Scrolls up in the message list (towards older messages)
    pub fn scroll_up(&mut self) {
        *self.0.offset_mut() += 1;
    }
}

/// The Chat widget that displays providing a scrollable view of chat messages.
///
/// The [ChatState] manages the state that determines which messages are currently visible.
#[derive(Default)]
pub struct Chat<'a> {
    messages: Vec<Text<'a>>,
    block: Option<Block<'a>>,
}

impl<'a> Chat<'a> {
    /// Creates a new Chat widget with the given messages.
    pub fn new<Iter>(messages: Iter) -> Self
    where
        Iter: IntoIterator<Item = Message<'a>>,
        Iter::Item: Into<Message<'a>>,
    {
        Self {
            messages: messages.into_iter().map(|msg| msg.to_text()).collect(),
            block: None,
        }
    }

    /// Sets the messages for the Chat widget.
    pub fn messages<Iter>(mut self, messages: Iter) -> Self
    where
        Iter: IntoIterator,
        Iter::Item: Into<Message<'a>>,
    {
        self.messages = messages
            .into_iter()
            .flat_map(|item| [item.into().to_text(), Text::raw("")])
            .collect();
        self
    }

    /// Sets the block for the Chat widget.
    pub fn block(mut self, block: Block<'a>) -> Self {
        self.block = Some(block);
        self
    }
}

impl StatefulWidget for Chat<'_> {
    type State = ChatState;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        let inner = self.block.inner_if_some(area);
        self.block.render(area, buf);

        let list = self
            .messages
            .into_iter()
            .flat_map(|txt| [txt, Text::from("")]);

        // TODO: Add a debug mode. highlight symbol would be a great toggle for that. .highlight_symbol("-")
        StatefulWidget::render(
            List::new(list)
                .highlight_symbol("-")
                .direction(ListDirection::BottomToTop),
            inner,
            buf,
            &mut state.0,
        );
    }
}

/// A helper method to turn a string into multiple lines wrapped at the given max width.
fn wrap_text(s: &str, max_width: usize) -> Vec<String> {
    let mut lines = Vec::new();
    let mut current = String::with_capacity(max_width);

    for word in s.split_whitespace() {
        if current.is_empty() {
            // First word in the line
            current.push_str(word);
        } else if current.len() + 1 + word.len() <= max_width {
            // Add to current line
            current.push(' ');
            current.push_str(word);
        } else {
            // Start a new line
            // This is purposeful to keep the capacity of the original vec.
            #[allow(clippy::drain_collect)]
            lines.push(current.drain(..).collect());
            current.push_str(word);
        }
    }

    if !current.is_empty() {
        lines.push(current);
    }

    let longest = lines.iter().map(|l| l.len()).max().unwrap_or(0);

    for line in &mut lines {
        if line.len() < longest {
            line.push_str(&" ".repeat(longest - line.len()));
        }
    }

    lines
}

// A helper method to format a message on the left side.
fn left<'a>(s: &str, width: usize) -> Text<'a> {
    let lines = wrap_text(s, width);

    let lines = lines.into_iter().map(|line| {
        Line::from(vec![
            Span::raw(" "),
            Span::styled(format!(" {line} "), Style::new().on_gray()),
        ])
        .left_aligned()
    });

    Text::from(lines.collect::<Vec<_>>())
}

// A helper method to format a message on the right side.
fn right<'a>(s: &str, width: usize) -> Text<'a> {
    let lines = wrap_text(s, width);

    let lines = lines.into_iter().map(|line| {
        Line::from(vec![
            Span::styled(format!(" {line} "), Style::new().on_blue()),
            Span::raw(" "),
        ])
        .right_aligned()
    });

    Text::from(lines.collect::<Vec<_>>())
}
