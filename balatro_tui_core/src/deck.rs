//! This module provides [`Deck`] as a primitive alias.
//!
//! This module also provides deck management methods and card tracking for UI
//! states. To utilize methods described on [`Deck`],
//! [`DeckConstExt`] and [`DeckExt`] traits must be brought into scope.

use std::sync::LazyLock;

use itertools::Itertools;
use rand::{seq::SliceRandom, thread_rng};
use strum::IntoEnumIterator;

use super::card::{Card, Rank, Suit};
use crate::error::{ArithmeticError, CoreError};

/// Lazy initializer for default deck.
///
/// More decks can be added using lazy initialization with use of
/// [`super::card::SuitIter`] and [`super::card::RankIter`].
pub static DEFAULT_DECK: LazyLock<Deck> = LazyLock::new(|| {
    Rank::iter()
        .cartesian_product(Suit::iter())
        .map(|(rank, suit)| Card { rank, suit })
        .collect()
});

/// [`Deck`] type is re-exported as an alias to [`Vec<Card>`] for better
/// contextual understanding.
pub type Deck = Vec<Card>;

/// Constructor extension trait for [`Deck`].
///
/// This trait only consists of construction function for various decks. Since,
/// this trait is implemented directly on foreign type [`Vec<Card>`], it gains
/// static methods to create decks on import.
pub trait DeckConstExt {
    /// Return a standard 52 card deck.
    #[must_use = "Created deck must be used."]
    fn standard() -> Deck;
}

/// Extension methods for [`Deck`], directly implemented on top of
/// [`Vec<Card>`].
pub trait DeckExt {
    /// In-place shuffle a deck based on thread/seed rng.
    fn shuffle(&mut self);
    /// Draw random cards from the deck and return new deck.
    #[must_use = "Drawn cards must be used."]
    fn draw_random(&mut self, draw_size: usize) -> Result<Deck, CoreError>;
}

impl DeckConstExt for Deck {
    #[inline]
    fn standard() -> Self {
        DEFAULT_DECK.to_vec()
    }
}

impl DeckExt for Deck {
    #[inline]
    fn shuffle(&mut self) {
        self.as_mut_slice().shuffle(&mut thread_rng());
    }

    fn draw_random(&mut self, draw_size: usize) -> Result<Deck, CoreError> {
        if draw_size > self.len() {
            return Err(CoreError::HandsExhaustedError);
        }
        self.shuffle();

        let drain_size = self
            .len()
            .checked_sub(draw_size)
            .ok_or(ArithmeticError::Overflow("subtraction"))?;
        let drawn_cards = self.drain(drain_size..).collect();

        Ok(drawn_cards)
    }
}
