//! This module contains implementation of a card and its corresponding
//! attributes.
//!
//! This module does not provide Joker as a card. Jokers are
//! provided in their own module and are not expected to be used with cards. A
//! deck can still be created by enum composition that can contain these
//! variants.

use std::{
    cmp::{Ordering, Reverse},
    collections::HashMap,
    fmt::{Display, Formatter, Result as FmtResult},
    ops::{Add, Sub},
    result::Result as StdResult,
    str::FromStr,
};

use color_eyre::{
    eyre::{OptionExt, Report},
    Result,
};
use itertools::Itertools;
use strum::{Display as EnumDisplay, EnumCount, EnumIter, EnumProperty, EnumString, IntoStaticStr};
use unicode_segmentation::UnicodeSegmentation;

/// Represents the suit of a card.
///
/// There are conversion methods for creating a [`Suit`] instance from unicode
/// and first letter notation.
///
/// ```
/// # use std::str::FromStr;
/// # use balatro_tui_core::card::Suit;
/// let parsed_suits = ["♣", "♦", "♥", "♠"].map(|suit| Suit::from_str(suit).unwrap());
/// let expected_suits = [Suit::Club, Suit::Diamond, Suit::Heart, Suit::Spade];
///
/// assert_eq!(parsed_suits, expected_suits);
/// ```
///
/// ```
/// # use std::str::FromStr;
/// # use balatro_tui_core::card::Suit;
/// let parsed_suits = ["C", "D", "H", "S"].map(|suit| Suit::from_str(suit).unwrap());
/// let expected_suits = [Suit::Club, Suit::Diamond, Suit::Heart, Suit::Spade];
///
/// assert_eq!(parsed_suits, expected_suits);
/// ```
///
/// Suit provides [`Suit::iter()`] method that can be used to create an iterator
/// over suit values.
///
/// ```
/// # use strum::IntoEnumIterator;
/// # use balatro_tui_core::card::Suit;
/// assert_eq!(Suit::iter().collect::<Vec<Suit>>(), vec![
///     Suit::Club,
///     Suit::Diamond,
///     Suit::Heart,
///     Suit::Spade
/// ]);
/// ```
#[derive(
    Clone,
    Copy,
    Debug,
    EnumDisplay,
    EnumCount,
    EnumIter,
    EnumString,
    Eq,
    Hash,
    IntoStaticStr,
    Ord,
    PartialEq,
    PartialOrd,
)]
pub enum Suit {
    /// Club suit (♣/C)
    #[strum(serialize = "\u{2663}", serialize = "C")]
    Club,
    /// Diamond suit (♦/D)
    #[strum(serialize = "\u{2666}", serialize = "D")]
    Diamond,
    /// Heart suit (♥/H)
    #[strum(serialize = "\u{2665}", serialize = "H")]
    Heart,
    /// Spade suit (♠/S)
    #[strum(serialize = "\u{2660}", serialize = "S")]
    Spade,
}

/// Represents the rank of the card.
///
/// There are conversion method to create a rank instance from serialized
/// representation.
///
/// ```
/// # use std::str::FromStr;
/// # use balatro_tui_core::card::Rank;
/// let parsed_ranks = ["A", "3", "10", "J", "Q", "K"].map(|rank| Rank::from_str(rank).unwrap());
/// let expected_ranks = [
///     Rank::Ace,
///     Rank::Three,
///     Rank::Ten,
///     Rank::Jack,
///     Rank::Queen,
///     Rank::King,
/// ];
///
/// assert_eq!(parsed_ranks, expected_ranks);
/// ```
///
/// There are different ordering and properties attached that can be used for
/// comparing, sorting and scoring.
///
/// ## Ordering and Representation
/// The ordinal representation represents the underlying index of the rank. In
/// this context, [`Rank::Ace`] has ordinal `1`, [`Rank::Jack`] has ordinal
/// `11`, [`Rank::Queen`] has ordinal `12` and [`Rank::King`] has ordinal `13`.
///
/// This ordinal is only used for representation purposes. For comparison and
/// sorting, a custom comparison is implemented that keeps [`Rank::Ace`] greater
/// than [`Rank::King`].
///
/// This is useful for sorting in most games. Although scoring must implement
/// custom checks for wrap-around instances (like straights).
///
/// ```
/// # use balatro_tui_core::card::Rank;
/// let mut unsorted_ranks = [Rank::Seven, Rank::King, Rank::Two, Rank::Ace];
/// let sorted_ranks = [Rank::Ace, Rank::King, Rank::Seven, Rank::Two];
///
/// unsorted_ranks.sort();
/// unsorted_ranks.reverse();
///
/// assert_eq!(unsorted_ranks, sorted_ranks);
/// ```
///
/// Since the scoring is independent of the ordinal, the rank carries additional
/// property of score that equates to ordinal value of a card from 2 through 10.
/// Ace and all face cards score for 10 points. The score for a rank can be
/// fetched using [`Self::get_score()`].
///
/// Since, the cards are comparable values, [`Add`] and [`Sub`] implementations
/// are provided for rank using their ordinal representation. `High Ace` must be
/// considered by scoring implementation as it won't be wrapping.
#[derive(
    Clone,
    Copy,
    Debug,
    EnumDisplay,
    EnumCount,
    EnumIter,
    EnumProperty,
    EnumString,
    Eq,
    Hash,
    IntoStaticStr,
    PartialEq,
)]
pub enum Rank {
    /// Ace rank (A)
    #[strum(serialize = "A", serialize = "1", props(score = "10"))]
    Ace = 1,
    /// Two rank (2)
    #[strum(serialize = "2", props(score = "2"))]
    Two,
    /// Three rank (3)
    #[strum(serialize = "3", props(score = "3"))]
    Three,
    /// Four rank (4)
    #[strum(serialize = "4", props(score = "4"))]
    Four,
    /// Five rank (5)
    #[strum(serialize = "5", props(score = "5"))]
    Five,
    /// Six rank (6)
    #[strum(serialize = "6", props(score = "6"))]
    Six,
    /// Seven rank (7)
    #[strum(serialize = "7", props(score = "7"))]
    Seven,
    /// Eight rank (8)
    #[strum(serialize = "8", props(score = "8"))]
    Eight,
    /// Nine rank (9)
    #[strum(serialize = "9", props(score = "9"))]
    Nine,
    /// Ten rank (10)
    #[strum(serialize = "10", props(score = "10"))]
    Ten,
    /// Jack rank (11)
    #[strum(serialize = "J", serialize = "11", props(score = "10"))]
    Jack,
    /// Queen rank (12)
    #[strum(serialize = "Q", serialize = "12", props(score = "10"))]
    Queen,
    /// King rank (13)
    #[strum(serialize = "K", serialize = "13", props(score = "10"))]
    King,
}

impl Rank {
    /// Returns score for given rank to be used in card scoring.
    #[inline]
    pub fn get_score(&self) -> Result<usize> {
        // TODO: Use get_int() when stabilized
        Ok(str::parse(self.get_str("score").ok_or_eyre(format!(
            "Score could not be fetched for rank: {self}."
        ))?)?)
    }
}

impl PartialOrd for Rank {
    #[inline]
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Rank {
    #[inline]
    fn cmp(&self, other: &Self) -> Ordering {
        if self == &Self::Ace {
            return Ordering::Greater;
        }
        if other == &Self::Ace {
            return Ordering::Less;
        }
        (*self as usize).cmp(&(*other as usize))
    }
}

impl Add for Rank {
    type Output = usize;

    #[inline]
    fn add(self, rhs: Self) -> Self::Output {
        (self as usize)
            .checked_add(rhs as usize)
            .ok_or_eyre("Add operation overflowed")
            .unwrap()
    }
}

impl Sub for Rank {
    type Output = isize;

    fn sub(self, rhs: Self) -> Self::Output {
        (self as isize)
            .checked_sub(rhs as isize)
            .ok_or_eyre("Subtract operation overflowed")
            .unwrap()
    }
}

/// Represents a card unit. Card is made of a [`Rank`] and a [`Suit`].
///
/// A standard pack of 52 cards can be expressed using this representation.
///
/// Card can also be created by parsing from unicode or representational string.
///
/// ```
/// # use std::str::FromStr;
/// # use balatro_tui_core::card::{Card, Rank, Suit};
/// assert_eq!(Card::from_str("J♣").unwrap(), Card { rank: Rank::Jack, suit: Suit::Club });
/// assert_eq!(Card::from_str("10♥").unwrap(), Card { rank: Rank::Ten, suit: Suit::Heart });
/// assert_eq!(Card::from_str("12♣").unwrap(), Card { rank: Rank::Queen, suit: Suit::Club });
/// assert_eq!(Card::from_str("5H").unwrap(), Card { rank: Rank::Five, suit: Suit::Heart });
/// assert_eq!(Card::from_str("7S").unwrap(), Card { rank: Rank::Seven, suit: Suit::Spade });
/// assert_eq!(Card::from_str("11D").unwrap(), Card { rank: Rank::Jack, suit: Suit::Diamond });
/// ```
#[derive(Clone, Copy, Debug, Ord, PartialOrd, PartialEq, Eq, Hash)]
pub struct Card {
    /// Rank of the card
    pub rank: Rank,
    /// Suit of the card
    pub suit: Suit,
}

impl Display for Card {
    #[inline]
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        write!(f, "{}{}", self.suit, self.rank)
    }
}

impl FromStr for Card {
    type Err = Report;

    fn from_str(s: &str) -> StdResult<Self, Self::Err> {
        let mut chars = s.graphemes(true).collect::<Vec<_>>();
        let suit_str = chars
            .pop()
            .ok_or_eyre("Could not unpack suit from provided string when parsing card.")?;
        Ok(Self {
            rank: Rank::from_str(&chars.join(""))?,
            suit: Suit::from_str(suit_str)?,
        })
    }
}

/// Trait that defines sorting methods for cards. This trait is implemented over
/// a slice of cards and thus methods can be used over [`\[Card;N\]`],
/// [`&\[Card\]`] and [`Vec<Card>`]
pub trait Sortable {
    /// In-place sorts the cards by [`Suit`] first and then by descending order
    /// of [`Rank`].
    fn sort_by_suit(&mut self);
    /// In-place sorts the cards by descending order of [`Rank`] first and then
    /// by [`Suit`].
    fn sort_by_rank(&mut self);
    /// Creates a new sorted [`Vec<Card>`] using the rules from
    /// [`Sortable::sort_by_suit()`].
    #[must_use = "Sorted cards must be used."]
    fn sorted_by_suit(&self) -> Vec<Card>;
    /// Creates a new sorted [`Vec<Card>`] using the rules from
    /// [`Sortable::sort_by_rank()`].
    #[must_use = "Sorted cards must be used."]
    fn sorted_by_rank(&self) -> Vec<Card>;
    /// Groups the played cards by their [`Suit`], gets the count of each group,
    /// sorts them in descending order based on the count, and returns the
    /// [`Vec<(Suit, usize)`]
    #[must_use = "Grouped suits must be used."]
    fn grouped_by_suit(&self) -> Vec<(Suit, usize)>;
    /// Groups the played cards by their [`Rank`], gets the count of each group,
    /// sorts them in descending order based on the count, and returns the
    /// [`Vec<(Rank, usize)`]
    #[must_use = "Grouped ranks must be used."]
    fn grouped_by_rank(&self) -> Vec<(Rank, usize)>;
}

impl Sortable for [Card] {
    #[inline]
    fn sort_by_suit(&mut self) {
        self.sort_by_key(|card| (card.suit, Reverse(card.rank)));
    }

    #[inline]
    fn sort_by_rank(&mut self) {
        self.sort_by_key(|card| (Reverse(card.rank), card.suit));
    }

    #[inline]
    fn sorted_by_suit(&self) -> Vec<Card> {
        let mut cards = self.to_vec();
        cards.sort_by_suit();
        cards
    }

    #[inline]
    fn sorted_by_rank(&self) -> Vec<Card> {
        let mut cards = self.to_vec();
        cards.sort_by_rank();
        cards
    }

    fn grouped_by_suit(&self) -> Vec<(Suit, usize)> {
        let group = self
            .iter()
            .fold(HashMap::new(), |mut groups, card| {
                _ = groups
                    .entry(card.suit)
                    .and_modify(|entry: &mut usize| {
                        *entry = entry
                            .checked_add(1)
                            .ok_or_eyre("Add operation overflowed")
                            .unwrap();
                    })
                    .or_insert(1);
                groups
            })
            .into_iter()
            .sorted_by_key(|group| group.1)
            .rev()
            .collect();
        group
    }

    fn grouped_by_rank(&self) -> Vec<(Rank, usize)> {
        let group = self
            .iter()
            .fold(HashMap::new(), |mut groups, card| {
                _ = groups
                    .entry(card.rank)
                    .and_modify(|entry: &mut usize| {
                        *entry = entry
                            .checked_add(1)
                            .ok_or_eyre("Add operation overflowed")
                            .unwrap();
                    })
                    .or_insert(1);
                groups
            })
            .into_iter()
            .sorted_by_key(|group| group.1)
            .rev()
            .collect();
        group
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn suit_from_unicode() {
        let parsed_suits = ["♣", "♦", "♥", "♠"].map(|suit| Suit::from_str(suit).unwrap());

        let expected_suits = [Suit::Club, Suit::Diamond, Suit::Heart, Suit::Spade];

        assert_eq!(parsed_suits, expected_suits);
    }

    #[test]
    fn suit_from_str() {
        let parsed_suits = ["C", "D", "H", "S"].map(|suit| Suit::from_str(suit).unwrap());

        let expected_suits = [Suit::Club, Suit::Diamond, Suit::Heart, Suit::Spade];

        assert_eq!(parsed_suits, expected_suits);
    }

    #[test]
    fn rank_from_str() {
        let parsed_ranks =
            ["A", "2", "3", "4", "5", "6", "7", "8", "9", "10", "J", "Q", "K"].map(|rank| Rank::from_str(rank).unwrap());

        let expected_ranks = [
            Rank::Ace,
            Rank::Two,
            Rank::Three,
            Rank::Four,
            Rank::Five,
            Rank::Six,
            Rank::Seven,
            Rank::Eight,
            Rank::Nine,
            Rank::Ten,
            Rank::Jack,
            Rank::Queen,
            Rank::King,
        ];

        assert_eq!(parsed_ranks, expected_ranks);
    }

    #[test]
    fn rank_from_repr() {
        let parsed_ranks =
            ["1", "2", "3", "4", "5", "6", "7", "8", "9", "10", "11", "12", "13"].map(|rank| Rank::from_str(rank).unwrap());

        let expected_ranks = [
            Rank::Ace,
            Rank::Two,
            Rank::Three,
            Rank::Four,
            Rank::Five,
            Rank::Six,
            Rank::Seven,
            Rank::Eight,
            Rank::Nine,
            Rank::Ten,
            Rank::Jack,
            Rank::Queen,
            Rank::King,
        ];

        assert_eq!(parsed_ranks, expected_ranks);
    }

    #[test]
    fn card_from_repr_rank_str_suit() {
        let parsed_cards = [
            "1C", "2D", "3H", "4S", "5C", "6D", "7H", "8S", "9C", "10D", "11H", "12S", "13C",
        ]
        .map(|card| Card::from_str(card).unwrap());

        let expected_cards = [
            Card {
                rank: Rank::Ace,
                suit: Suit::Club,
            },
            Card {
                rank: Rank::Two,
                suit: Suit::Diamond,
            },
            Card {
                rank: Rank::Three,
                suit: Suit::Heart,
            },
            Card {
                rank: Rank::Four,
                suit: Suit::Spade,
            },
            Card {
                rank: Rank::Five,
                suit: Suit::Club,
            },
            Card {
                rank: Rank::Six,
                suit: Suit::Diamond,
            },
            Card {
                rank: Rank::Seven,
                suit: Suit::Heart,
            },
            Card {
                rank: Rank::Eight,
                suit: Suit::Spade,
            },
            Card {
                rank: Rank::Nine,
                suit: Suit::Club,
            },
            Card {
                rank: Rank::Ten,
                suit: Suit::Diamond,
            },
            Card {
                rank: Rank::Jack,
                suit: Suit::Heart,
            },
            Card {
                rank: Rank::Queen,
                suit: Suit::Spade,
            },
            Card {
                rank: Rank::King,
                suit: Suit::Club,
            },
        ];

        assert_eq!(parsed_cards, expected_cards);
    }

    #[test]
    fn card_from_repr_rank_unicode_suit() {
        let parsed_cards = [
            "1♣", "2♦", "3♥", "4♠", "5♣", "6♦", "7♥", "8♠", "9♣", "10♦", "11♥", "12♠", "13♣",
        ]
        .map(|card| Card::from_str(card).unwrap());

        let expected_cards = [
            Card {
                rank: Rank::Ace,
                suit: Suit::Club,
            },
            Card {
                rank: Rank::Two,
                suit: Suit::Diamond,
            },
            Card {
                rank: Rank::Three,
                suit: Suit::Heart,
            },
            Card {
                rank: Rank::Four,
                suit: Suit::Spade,
            },
            Card {
                rank: Rank::Five,
                suit: Suit::Club,
            },
            Card {
                rank: Rank::Six,
                suit: Suit::Diamond,
            },
            Card {
                rank: Rank::Seven,
                suit: Suit::Heart,
            },
            Card {
                rank: Rank::Eight,
                suit: Suit::Spade,
            },
            Card {
                rank: Rank::Nine,
                suit: Suit::Club,
            },
            Card {
                rank: Rank::Ten,
                suit: Suit::Diamond,
            },
            Card {
                rank: Rank::Jack,
                suit: Suit::Heart,
            },
            Card {
                rank: Rank::Queen,
                suit: Suit::Spade,
            },
            Card {
                rank: Rank::King,
                suit: Suit::Club,
            },
        ];

        assert_eq!(parsed_cards, expected_cards);
    }

    #[test]
    fn card_from_str_rank_and_str_suit() {
        let parsed_cards = [
            "AC", "JD", "QH", "KS"
        ]
        .map(|card| Card::from_str(card).unwrap());

        let expected_cards = [
            Card {
                rank: Rank::Ace,
                suit: Suit::Club,
            },
            Card {
                rank: Rank::Jack,
                suit: Suit::Diamond,
            },
            Card {
                rank: Rank::Queen,
                suit: Suit::Heart,
            },
            Card {
                rank: Rank::King,
                suit: Suit::Spade,
            },
        ];

        assert_eq!(parsed_cards, expected_cards);
    }

    #[test]
    fn card_from_str_rank_and_unicode_suit() {
        let parsed_cards = [
            "A♣", "J♦", "Q♥", "K♠",
        ]
        .map(|card| Card::from_str(card).unwrap());

        let expected_cards = [
            Card {
                rank: Rank::Ace,
                suit: Suit::Club,
            },
            Card {
                rank: Rank::Jack,
                suit: Suit::Diamond,
            },
            Card {
                rank: Rank::Queen,
                suit: Suit::Heart,
            },
            Card {
                rank: Rank::King,
                suit: Suit::Spade,
            },
        ];

        assert_eq!(parsed_cards, expected_cards);
    }

    #[test]
    fn sort_ranks() {
        let mut unsorted_ranks = [Rank::Seven, Rank::King, Rank::Two, Rank::Ace];

        let sorted_ranks = [Rank::Ace, Rank::King, Rank::Seven, Rank::Two];

        unsorted_ranks.sort();
        unsorted_ranks.reverse();

        assert_eq!(unsorted_ranks, sorted_ranks);
    }
}
