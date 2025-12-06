use std::cmp::min;

use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::Line,
    widgets::{Block, ListState, Paragraph, StatefulWidget, Tabs, Widget, block::Title},
};

/// The state type for the DashBoard widget.
pub type DashBoardState = ListState;

/// The DashBoard widget that acts as the top-level container.
///
/// Contains three sections:
/// 1. The Tabs at the top for navigation with an optional title ([Self::title]).
/// 2. The main content area in the middle ([Self::inner]).
/// 3. A help footer at the bottom showing available command ([Self::help]).
#[derive(Default)]
pub struct DashBoard<'a> {
    /// The title for the dashboard.
    title: Title<'a>,
    /// The available tab titles.
    tabs: Vec<Line<'a>>,
    /// An optional footer.
    footer: Option<(Paragraph<'a>, Constraint)>,
}

impl<'a> DashBoard<'a> {
    /// Creates a new DashBoard with the given tab titles
    pub fn new<Iter>(titles: Iter) -> Self
    where
        Iter: IntoIterator,
        Iter::Item: Into<Line<'a>>,
    {
        Self {
            tabs: titles.into_iter().map(Into::into).collect(),
            ..Default::default()
        }
    }

    /// Sets the tab titles for the DashBoard.
    pub fn titles<Iter>(mut self, titles: Iter) -> Self
    where
        Iter: IntoIterator,
        Iter::Item: Into<Line<'a>>,
    {
        self.tabs = titles.into_iter().map(Into::into).collect();
        self
    }

    /// Sets the title for the DashBoard.
    pub fn title(mut self, title: impl Into<Title<'a>>) -> Self {
        self.title = title.into();
        self
    }

    /// Create a footer at the bottom of the dashboard
    pub fn footer(mut self, footer: Paragraph<'a>, constraint: Constraint) -> Self {
        self.footer = Some((footer, constraint));
        self
    }

    pub fn inner(&self, area: Rect) -> Rect {
        self.chunks(area).0[1]
    }

    /// Returns the layout chunks for the dashboard
    fn chunks(&self, area: Rect) -> ([Rect; 2], Option<Rect>) {
        let area = Layout::new(
            Direction::Horizontal,
            [
                Constraint::Fill(1),
                Constraint::Max(100),
                Constraint::Fill(1),
            ],
        )
        .split(area)[1];

        let mut constraints = vec![Constraint::Length(3), Constraint::Min(0)];
        if let Some((_, constraint)) = &self.footer {
            constraints.push(*constraint);
        }

        let areas = Layout::new(Direction::Vertical, constraints)
            .margin(1)
            .split(area);

        let cmd_area = if self.footer.is_some() {
            Some(areas[2])
        } else {
            None
        };

        ([areas[0], areas[1]], cmd_area)
    }
}

impl<'a> StatefulWidget for DashBoard<'a> {
    type State = DashBoardState;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        // Split into 3 rows: Tabs, content, Commands
        let ([tab_area, _], cmd_area) = self.chunks(area);

        // Fix up the selected tab index if it is out of bounds
        state.select(min(
            Some(self.tabs.len().saturating_sub(1)),
            state.selected(),
        ));

        Tabs::new(self.tabs)
            .block(Block::bordered().title(self.title))
            .select(state.selected())
            .style(Style::default().fg(Color::Cyan))
            .highlight_style(Style::default().add_modifier(Modifier::BOLD))
            .render(tab_area, buf);

        if let (Some(cmd_area), Some((footer, _))) = (cmd_area, self.footer) {
            footer.render(cmd_area, buf);
        }
    }
}
