use std::default::Default;

use balatro_tui_core::card::Card;
use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Layout, Margin, Rect},
    symbols::border::{self, Set},
    text::Line,
    widgets::{Block, Paragraph, StatefulWidget, Widget},
};

use super::text_box::TextBoxWidget;

/// Content width for [`CardWidget`].
pub const CARD_CONTENT_WIDTH: u16 = 12;
/// Content height for [`CardWidget`].
pub const CARD_CONTENT_HEIGHT: u16 = 9;

// TODO: Convert to references with lifetimes and implement Copy on structs
// wherever possible.
// TODO: Implement Rc or Arc implementation based on multithreading feature
// TODO: Implement std or tokio implementation based on tokio feature
// TODO: Remove unneeded usages of clone()
// TODO: Explore blanket trait implementation for From over mutability.

/// [`Widget`] to display a [`Card`].
///
/// Widget construction uses builder pattern which can be started using the
/// [`Self::new()`] method.
///
/// ```
/// # use ratatui::{buffer::Buffer, layout::Rect, prelude::StatefulWidget, style::Color, symbols::border, text::Line};
/// # use balatro_tui_core::card::{Card, Rank, Suit};
/// # use balatro_tui_widgets::CardWidget;
/// let area = Rect::new(0, 0, 100, 100);
/// let mut buffer = Buffer::empty(area);
/// let mut card = Card {
///     rank: Rank::Ace,
///     suit: Suit::Club,
/// };
///
/// CardWidget::bordered(border::THICK).render(area, &mut buffer, &mut card);
/// ```
///
/// A hovered card is represented with border as [`border::THICK`], otherwise
/// border is set to [`border::ROUNDED`].
#[derive(Clone, Copy, Debug, Default)]
pub struct CardWidget {
    border_set: Set,
}

impl CardWidget {
    /// Create new instance of [`CardWidget`].
    #[must_use = "Created card widget state instance must be used."]
    #[inline]
    pub const fn new() -> Self {
        Self {
            border_set: border::ROUNDED,
        }
    }

    /// Create new instance of [`CardWidget`] with specified border set.
    #[must_use = "Card widget builder returned instance must be used."]
    #[inline]
    pub const fn bordered(border_set: Set) -> Self {
        Self { border_set }
    }

    /// Update the border set of the card and return the [`CardWidget`]
    /// instance.
    #[must_use = "Card widget builder returned instance must be used."]
    #[inline]
    pub const fn border(mut self, border_set: Set) -> Self {
        self.border_set = border_set;
        self
    }
}

impl StatefulWidget for CardWidget {
    type State = Card;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        // Prepare areas
        let mut inner_area =
            Layout::vertical([Constraint::Length(CARD_CONTENT_HEIGHT)]).areas::<1>(area)[0];
        inner_area =
            Layout::horizontal([Constraint::Length(CARD_CONTENT_WIDTH)]).areas::<1>(inner_area)[0];
        let [top_area, middle_area, bottom_area] = Layout::vertical([
            Constraint::Length(2),
            Constraint::Fill(1),
            Constraint::Length(2),
        ])
        .areas(inner_area.inner(Margin::new(1, 1)));

        // Render containers
        Block::bordered()
            .border_set(self.border_set)
            .render(inner_area, buf);

        // Render widgets
        Paragraph::new(format!("{}\r\n{}", state.rank, state.suit))
            .left_aligned()
            .render(top_area, buf);
        // TODO: Mimic actual card suit layout
        TextBoxWidget::new([Line::from(format!("{}{}", state.rank, state.suit)).centered()])
            .render(middle_area, buf);
        Paragraph::new(format!("{}\r\n{}", state.suit, state.rank))
            .right_aligned()
            .render(bottom_area, buf);
    }
}
