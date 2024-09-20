use std::{
    collections::HashSet,
    sync::{Arc, Mutex},
};

use balatro_tui_core::card::Card;
use color_eyre::{
    eyre::{bail, OptionExt},
    Result,
};
use itertools::Itertools;
use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Layout, Offset, Rect},
    symbols::border,
    widgets::StatefulWidget,
};

use super::CardWidget;

/// Provide bidirectional circular cursor for iterators with ability to select
/// items.
///
/// By default the list is unfocussed. To move the cursor to the first element,
/// call [`SelectableList::move_next()`].
pub trait SelectableList {
    /// Move the cursor to next item. If next item doesn't exist, cycle back to
    /// the first item.
    fn move_next(&mut self) -> Result<()>;
    /// Move the cursor to previous item. If previous item doesn't exist, cycle
    /// to the last item.
    fn move_prev(&mut self) -> Result<()>;
    /// Add item at current cursor position to the selection list. No-op if the
    /// index is already selected.
    fn select(&mut self) -> Result<bool>;
    /// Remove item at current cursor position from the selection list. No-op if
    /// the index is already deselected.
    fn deselect(&mut self) -> Result<bool>;
    /// Unfocus (blur) the list. This method removes the cursor from the list.
    fn blur(&mut self);
}

// TODO: Move unchanging items out of state for widgets.
// TODO: Create macros for creating widgets.

/// Render state for [`CardListWidget`].
///
/// Holds a atomic mutable reference to a [`Vec<Card>`]. Tracks the current
/// cursor position and selected [`Card`] set.
///
/// [`CardListWidget`] can be created out of a [`Vec<Card>`] reference using the
/// [`Self::from()`] implementation.
///
/// ```
/// # use std::sync::{Arc, Mutex};
/// # use balatro_tui_core::card::{Card, Rank, Suit};
/// # use balatro_tui_widgets::CardListWidgetState;
/// let cards = vec![
///     Card {
///         rank: Rank::Ace,
///         suit: Suit::Diamond,
///     },
///     Card {
///         rank: Rank::Ten,
///         suit: Suit::Heart,
///     },
/// ];
///
/// let list_state = CardListWidgetState::from(Arc::from(Mutex::from(cards)));
/// ```
#[derive(Debug)]
pub struct CardListWidgetState {
    /// Holds a atomic mutable reference to a [`Vec<Card>`].
    pub cards: Arc<Mutex<Vec<Card>>>,
    /// Cursor position over the [`Self::cards`].
    pub pos: Option<usize>,
    // TODO: Use bit-mask for selected value over usize
    /// A cache of selected card indices.
    pub selected: HashSet<usize>,
    /// Optional limit defines the maximum cards that can be selected.
    pub selection_limit: Option<usize>,
}

impl CardListWidgetState {
    /// Update the [`Self::selection_limit`] and return the
    /// [`CardListWidgetState`] instance.
    #[must_use = "Card list widget state builder returned instance must be used."]
    #[inline]
    pub fn selection_limit(mut self, selection_limit: Option<usize>) -> Result<Self> {
        if selection_limit.is_some_and(|limit| limit < self.selected.len()) {
            bail!("Cannot reduce selection limit if number of selected cards is more than it.")
        }

        self.selection_limit = selection_limit;
        Ok(self)
    }

    /// Updates the [`Self::cards`] and reset selected and focussed positions.
    #[inline]
    pub fn set_cards(&mut self, cards: Arc<Mutex<Vec<Card>>>) {
        self.cards = cards;
        self.pos = None;
        self.selected.clear();
    }
}

impl From<Arc<Mutex<Vec<Card>>>> for CardListWidgetState {
    fn from(value: Arc<Mutex<Vec<Card>>>) -> Self {
        Self {
            cards: value,
            pos: None,
            selected: HashSet::new(),
            selection_limit: None,
        }
    }
}

impl SelectableList for CardListWidgetState {
    #[expect(
        clippy::arithmetic_side_effects,
        clippy::integer_division_remainder_used,
        reason = "Intended: Modulo arithmetic is pre-checked."
    )]
    fn move_next(&mut self) -> Result<()> {
        if let Some(pos) = self.pos {
            let max_length = self
                .cards
                .lock()
                .or_else(|err| bail!("Could not attain lock for cards: {err}."))?
                .len();

            self.pos = Some(
                (pos.checked_add(max_length)
                    .ok_or_eyre("Addition operation overflowed")?
                    .checked_add(1)
                    .ok_or_eyre("Addition operation overflowed")?)
                    % max_length,
            );
        } else {
            self.pos = Some(0);
        }

        Ok(())
    }

    #[expect(
        clippy::arithmetic_side_effects,
        clippy::integer_division_remainder_used,
        reason = "Intended: Modulo arithmetic is pre-checked."
    )]
    fn move_prev(&mut self) -> Result<()> {
        if let Some(pos) = self.pos {
            let max_length = self
                .cards
                .lock()
                .or_else(|err| bail!("Could not attain lock for cards: {err}."))?
                .len();

            self.pos = Some(
                (pos.checked_add(max_length)
                    .ok_or_eyre("Addition operation overflowed")?
                    .checked_sub(1)
                    .ok_or_eyre("Addition operation overflowed")?)
                    % max_length,
            );
        } else {
            self.pos = Some(0);
        }

        Ok(())
    }

    #[inline]
    fn select(&mut self) -> Result<bool> {
        if self
            .selection_limit
            .is_some_and(|limit| limit < self.selected.len())
        {
            return Ok(false);
        }

        if let Some(pos) = self.pos {
            return Ok(self.selected.insert(pos));
        }

        Ok(false)
    }

    #[inline]
    fn deselect(&mut self) -> Result<bool> {
        if let Some(pos) = self.pos {
            return Ok(self.selected.remove(&pos));
        }

        Ok(false)
    }

    #[inline]
    fn blur(&mut self) {
        self.pos = None;
    }
}

/// [`StatefulWidget`] to display a list of [`Card`].
///
/// Widget construction uses builder pattern which can be started using the
/// [`Self::new()`] method.
///
/// ```
/// # use std::sync::{Arc, Mutex};
/// # use ratatui::{buffer::Buffer, layout::Rect, prelude::*};
/// # use balatro_tui_core::card::{Card, Rank, Suit};
/// # use balatro_tui_widgets::{CardListWidget, CardListWidgetState};
/// let area = Rect::new(0, 0, 100, 100);
/// let mut buffer = Buffer::empty(area);
/// let mut card_list = CardListWidgetState::from(
///     Arc::from(Mutex::from(vec![
///         Card {
///             rank: Rank::Ace,
///             suit: Suit::Club,
///         },
///         Card {
///             rank: Rank::Two,
///             suit: Suit::Heart,
///         },
///         Card {
///             rank: Rank::Ten,
///             suit: Suit::Diamond,
///         },
///     ]))
/// );
///
/// CardListWidget::new().render(area, &mut buffer, &mut card_list);
/// ```
#[derive(Clone, Copy, Debug, Default)]
pub struct CardListWidget;

impl CardListWidget {
    /// Create new instance of [`CardListWidget`].
    #[must_use]
    #[inline]
    pub const fn new() -> Self {
        Self {}
    }
}

// TODO: Add pub(crate) qualifications
// TODO: Remove unused pub qualifications
// TODO: Optimize card widget construction

impl StatefulWidget for CardListWidget {
    type State = CardListWidgetState;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        // Cards
        let cards = state.cards.lock().unwrap();

        // Prepare areas
        let deck_areas = Layout::horizontal(vec![Constraint::Fill(1); cards.len()]).split(area);

        // Render widgets
        cards
            .clone()
            .into_iter()
            .zip_eq(deck_areas.iter().copied())
            .enumerate()
            .for_each(|(idx, (mut card, mut card_area))| {
                if state.selected.contains(&idx) {
                    card_area = card_area.offset(Offset { x: 0, y: -5 });
                }

                CardWidget::bordered(
                    if state.pos == Some(idx) {
                        border::THICK
                    } else {
                        border::ROUNDED
                    },
                )
                .render(card_area, buf, &mut card);
            });
    }
}
