//! Scorer provides generic scoring mechanism for a set of cards played.
//!
//! It extends the normal deck scoring to scoring hands available in biased
//! decks as well with [`ScoringHand::FlushFive`], [`ScoringHand::FlushHouse`]
//! and [`ScoringHand::FiveOfAKind`].

use color_eyre::{eyre::OptionExt, Result};
use strum::{Display, EnumCount, EnumIter, EnumProperty, EnumString, IntoStaticStr};

use super::card::{Card, Rank, Sortable};

/// [`ScoringHand`] represents which kind of hand is made when playing a set of
/// cards.
///
/// A scoring hand has associated values of base `chips` and `multiplier` to be
/// used when scoring the hand.
///
/// [`ScoringHand`] also implements conversion from string representation.
///
/// ```
/// # use std::str::FromStr;
/// # use balatro_tui_core::scorer::ScoringHand;
/// assert_eq!(ScoringHand::from_str("Flush").unwrap(), ScoringHand::Flush);
/// assert_eq!(
///     ScoringHand::from_str("Four of a Kind").unwrap(),
///     ScoringHand::FourOfAKind
/// );
/// assert_eq!(
///     ScoringHand::from_str("Two Pair").unwrap(),
///     ScoringHand::TwoPair,
/// );
/// ```
///
/// The scoring hands are provided in order of scoring precedence (reverse in
/// ordinal).
///
/// When scoring, the order of cards doesn't matter. It is internally sorted by
/// [`Rank`] and [`super::card::Suit`] as necessary.
#[derive(
    Clone,
    Copy,
    Debug,
    Display,
    EnumCount,
    EnumIter,
    EnumProperty,
    EnumString,
    Eq,
    Hash,
    IntoStaticStr,
    Ord,
    PartialEq,
    PartialOrd,
)]
pub enum ScoringHand {
    /// [`ScoringHand::FlushFive`] is scored when played cards have five cards
    /// of the same [`Rank`] and same [`super::card::Suit`].
    ///
    /// ## Examples
    /// - A♥, A♥, A♥, A♥, A♥
    /// - 9♣, 9♣, 9♣, 9♣, 9♣
    #[strum(serialize = "Flush Five", props(chips = "160", multiplier = "16"))]
    FlushFive,
    /// [`ScoringHand::FlushHouse`] is scored when played cards have five cards
    /// of the same [`super::card::Suit`] which have two of same [`Rank`] and
    /// three of same [`Rank`].
    ///
    /// ## Examples
    /// - K♥, K♥, K♥, 10♥, 10♥
    /// - 5♣, 7♣, 5♣, 7♣, 5♣
    #[strum(serialize = "Flush House", props(chips = "140", multiplier = "14"))]
    FlushHouse,
    /// [`ScoringHand::FiveOfAKind`] is scored when played cards have five cards
    /// of the same [`Rank`] regardless of the [`super::card::Suit`].
    ///
    /// ## Examples
    /// - Q♥, Q♣, Q♦, Q♠, Q♥
    /// - 6♣, 6♣, 6♣, 6♣, 6♣
    #[strum(serialize = "Five of a Kind", props(chips = "120", multiplier = "12"))]
    FiveOfAKind,
    /// [`ScoringHand::RoyalFlush`] is scored when played cards have five cards
    /// of the same [`super::card::Suit`] and they form a straight with a high
    /// ace.
    ///
    /// ## Examples
    /// - A♥, K♥, Q♥, J♥, 10♥
    #[strum(serialize = "Royal Flush", props(chips = "100", multiplier = "8"))]
    RoyalFlush,
    /// [`ScoringHand::StraightFlush`] is scored when played cards have five
    /// cards of the same [`super::card::Suit`] and they form a straight.
    ///
    /// ## Examples
    /// - 7♣, 6♣, 8♣, 5♣, 4♣
    /// - K♥, Q♥, J♥, 10♥, 9♥
    #[strum(serialize = "Straight Flush", props(chips = "60", multiplier = "7"))]
    StraightFlush,
    /// [`ScoringHand::FourOfAKind`] is scored when played cards have four cards
    /// of the same [`Rank`]. The remaining card isn't scored.
    ///
    /// ## Examples
    /// - 7♣, 7♥, 7♦, 7♣, 4♦
    /// - 6♦, 6♥, 5♦, 6♣, 6♥
    #[strum(serialize = "Four of a Kind", props(chips = "40", multiplier = "4"))]
    FourOfAKind,
    /// [`ScoringHand::FullHouse`] is scored when played cards have two cards of
    /// the same [`Rank`] and another three of the same [`Rank`].
    ///
    /// ## Examples
    /// 6♣, 5♥, 5♦, 6♣, 5♦
    /// 3♥, A♥, A♦, A♣, 3♣
    #[strum(serialize = "Full House", props(chips = "35", multiplier = "4"))]
    FullHouse,
    /// [`ScoringHand::Flush`] is scored when played cards have five cards of
    /// the same [`super::card::Suit`] regardless of their [`Rank`].
    ///
    /// ## Examples
    /// A♦, 3♦, 5♦, 8♦, 10♦
    #[strum(serialize = "Flush", props(chips = "30", multiplier = "4"))]
    Flush,
    /// [`ScoringHand::Straight`] is scored when played cards have five cards
    /// that form a sequence of consecutive [`Rank`] regardless of their
    /// [`super::card::Suit`].
    ///
    /// ## Examples
    /// 4♣, 6♥, 5♦, 3♣, 7♦
    #[strum(serialize = "Straight", props(chips = "30", multiplier = "3"))]
    Straight,
    #[strum(serialize = "Three of a Kind", props(chips = "20", multiplier = "2"))]
    /// [`ScoringHand::ThreeOfAKind`] is scored when played cards have three
    /// cards that have the same [`Rank`]. Rest of the cards are not scored.
    ///
    /// ## Examples
    /// K♥, 6♣, 6♦, 6♥, 10♥
    ThreeOfAKind,
    #[strum(serialize = "Two Pair", props(chips = "20", multiplier = "2"))]
    /// [`ScoringHand::TwoPair`] is scored when played cards have two cards of
    /// the same [`Rank`] and another two of the same [`Rank`]. Remaining card
    /// isn't scored.
    ///
    /// ## Examples
    /// 9♣, 9♥, 5♦, J♣, J♦
    TwoPair,
    /// [`ScoringHand::Pair`] is scored when played cards have two cards of the
    /// same [`Rank`]. Remaining cards are not scored.
    ///
    /// ## Examples
    /// 6♣, 6♥, 5♦, 8♣, K♦
    #[strum(serialize = "Pair", props(chips = "10", multiplier = "2"))]
    Pair,
    /// [`ScoringHand::HighCard`] is scored when played cards does not satisfy
    /// any other scoring criteria. Only the card with highest [`Rank`] is
    /// scored. In this case, [`Rank::Ace`] is always scored as a high ace.
    ///
    /// ## Examples
    /// 2♥, 8♣, 7♦, K♥, 4♥
    #[strum(serialize = "High Card", props(chips = "5", multiplier = "1"))]
    HighCard,
}

/// Holds information regarding testing for a straight in the played hand.
#[derive(Clone, Debug)]
struct StraightTestReport {
    /// A optional boolean that can indicate the following:
    /// - [`None`]: No ace was counted in the the straight scoring.
    /// - [`Some(false)`]: Ace was counted in the the straight scoring and was a
    ///   low ace, ie, straight was made from ranks `5, 4, 3, 2, A`
    /// - [`Some(true)`]: Ace was counted in the the straight scoring and was a
    ///   high ace, ie, straight was made from ranks `A, K, Q, J, 10`
    pub high_ace: Option<bool>,
    /// Collection of scored ranks.
    pub scored_ranks: Vec<Rank>,
}

/// Container for static scoring methods.
///
/// [`Scorer::score_cards`] is a wrapper that handles scoring for cards. It
/// should satisfy most requirements.
#[derive(Clone, Copy, Debug, Ord, PartialOrd, Eq, Hash, PartialEq)]
pub struct Scorer;

impl Scorer {
    /// Returns chips and multiplier for a [`ScoringHand`].
    #[inline]
    pub fn get_chips_and_multiplier(scoring_hand: ScoringHand) -> Result<(usize, usize)> {
        Ok((
            str::parse(scoring_hand.get_str("chips").ok_or_eyre(format!(
                "Chips could not be fetched for scoring hand: {scoring_hand}."
            ))?)?,
            str::parse(scoring_hand.get_str("multiplier").ok_or_eyre(format!(
                "Multiplier could not be fetched for scoring hand: {scoring_hand}."
            ))?)?,
        ))
    }

    /// Tests for a straight in a slice of [`Card`] and returns a
    /// [`StraightTestReport`]
    fn test_straight(cards: &[Card]) -> Option<StraightTestReport> {
        let has_ace = cards.iter().any(|card| card.rank == Rank::Ace);
        let mut ranks_without_ace = cards
            .iter()
            .map(|card| card.rank)
            .filter(|rank| rank != &Rank::Ace)
            .collect::<Vec<_>>();

        #[expect(
            clippy::arithmetic_side_effects,
            reason = "False positive: Rank implements safe subtraction."
        )]
        let comparator = ranks_without_ace
            .iter()
            .map(|&rank| {
                ranks_without_ace
                    .first()
                    .map(|&first_rank| first_rank - rank)
            })
            .collect::<Option<Vec<_>>>()?;

        if comparator.len() < 4 {
            return None;
        }

        // TODO: Convert to bitwise scoring that will hold true for more than five
        // cards.
        if comparator.eq(&[0, 1, 2, 3, 4]) {
            return Some(StraightTestReport {
                high_ace: None,
                scored_ranks: ranks_without_ace,
            });
        }

        let is_partial_straight = comparator.eq(&[0, 1, 2, 3]);
        let is_low_straight =
            has_ace && is_partial_straight && cards.iter().any(|card| card.rank == Rank::Two);
        let is_high_straight =
            has_ace && is_partial_straight && cards.iter().any(|card| card.rank == Rank::King);

        if is_low_straight {
            ranks_without_ace.push(Rank::Ace);
            return Some(StraightTestReport {
                high_ace: Some(false),
                scored_ranks: ranks_without_ace,
            });
        }

        if is_high_straight {
            ranks_without_ace.insert(0, Rank::Ace);
            return Some(StraightTestReport {
                high_ace: Some(true),
                scored_ranks: ranks_without_ace,
            });
        }

        None
    }

    /// Returns [`ScoringHand`] for played cards.
    #[expect(
        clippy::indexing_slicing,
        reason = "Refactor: Current implementation guarantees index accesses are safe, but this can be refactored."
    )]
    pub fn get_scoring_hand(cards: &[Card]) -> Result<(Option<ScoringHand>, Vec<Rank>)> {
        let sorted_cards = cards.sorted_by_rank();
        let suit_groups = sorted_cards.grouped_by_suit();
        let rank_groups = sorted_cards.grouped_by_rank();
        let straight_test_result = Self::test_straight(&sorted_cards);

        if suit_groups.is_empty() || rank_groups.is_empty() {
            return Ok((None, vec![]));
        }

        if suit_groups[0].1 == 5 && rank_groups[0].1 == 5 {
            return Ok((Some(ScoringHand::FlushFive), vec![
                rank_groups[0].0;
                rank_groups[0].1
            ]));
        }

        if rank_groups.len() >= 2
            && suit_groups[0].1 == 5
            && rank_groups[0].1 == 3
            && rank_groups[1].1 == 2
        {
            let mut played_ranks = vec![];
            played_ranks.append(&mut vec![rank_groups[0].0; rank_groups[0].1]);
            played_ranks.append(&mut vec![rank_groups[1].0; rank_groups[1].1]);
            return Ok((Some(ScoringHand::FlushHouse), played_ranks));
        }

        if rank_groups[0].1 == 5 {
            return Ok((Some(ScoringHand::FiveOfAKind), vec![
                rank_groups[0].0;
                rank_groups[0].1
            ]));
        }

        if suit_groups[0].1 == 5 {
            if let Some(result) = straight_test_result {
                if result.high_ace.unwrap_or(false) {
                    return Ok((Some(ScoringHand::RoyalFlush), result.scored_ranks));
                }

                return Ok((Some(ScoringHand::StraightFlush), result.scored_ranks));
            }
        }

        if rank_groups[0].1 == 4 {
            return Ok((Some(ScoringHand::FourOfAKind), vec![
                rank_groups[0].0;
                rank_groups[0].1
            ]));
        }

        if rank_groups.len() >= 2 && rank_groups[0].1 == 3 && rank_groups[1].1 == 2 {
            let mut played_ranks = vec![];
            played_ranks.append(&mut vec![rank_groups[0].0; rank_groups[0].1]);
            played_ranks.append(&mut vec![rank_groups[1].0; rank_groups[1].1]);
            return Ok((Some(ScoringHand::FullHouse), played_ranks));
        }

        if suit_groups[0].1 == 5 {
            return Ok((
                Some(ScoringHand::Flush),
                cards.iter().map(|card| card.rank).collect(),
            ));
        }

        if let Some(result) = straight_test_result {
            return Ok((Some(ScoringHand::Straight), result.scored_ranks));
        }

        if rank_groups[0].1 == 3 {
            return Ok((Some(ScoringHand::ThreeOfAKind), vec![
                rank_groups[0].0;
                rank_groups[0].1
            ]));
        }

        if rank_groups.len() >= 2 && rank_groups[0].1 == 2 && rank_groups[1].1 == 2 {
            let mut played_ranks = vec![];
            played_ranks.append(&mut vec![rank_groups[0].0; rank_groups[0].1]);
            played_ranks.append(&mut vec![rank_groups[1].0; rank_groups[1].1]);
            return Ok((Some(ScoringHand::TwoPair), played_ranks));
        }

        if rank_groups[0].1 == 2 {
            return Ok((Some(ScoringHand::Pair), vec![
                rank_groups[0].0;
                rank_groups[0].1
            ]));
        }

        Ok((Some(ScoringHand::HighCard), vec![
            rank_groups[0].0;
            rank_groups[0].1
        ]))
    }

    /// Score played cards and return the computed score.
    pub fn score_cards(cards: &[Card]) -> Result<usize> {
        let (scoring_hand, scored_ranks) = Self::get_scoring_hand(cards)?;
        let (base_chips, multiplier) = Self::get_chips_and_multiplier(
            scoring_hand.ok_or_eyre("Attempted to score with no cards.")?,
        )?;
        let chips_increment = Self::score_chips_from_ranks(&scored_ranks)?;
        (base_chips
            .checked_add(chips_increment)
            .ok_or_eyre("Add operation overflowed")?)
        .checked_mul(multiplier)
        .ok_or_eyre("Multiplication operation overflowed")
    }

    /// Return total score from [`Rank`] from cards.
    #[inline]
    fn score_chips_from_ranks(ranks: &[Rank]) -> Result<usize> {
        ranks.iter().try_fold(0, |acc, rank| {
            let score = rank.get_score()?;
            score
                .checked_add(acc)
                .ok_or_eyre("Add operation overflowed")
        })
    }
}

// TODO: Add more tests for core functionality

#[cfg(test)]
mod tests {
    use super::*;
    use crate::card::Suit;

    #[test]
    fn score_flush_five() {
        let test_cards = [
            Card {
                rank: Rank::Ten,
                suit: Suit::Club,
            },
            Card {
                rank: Rank::Ten,
                suit: Suit::Club,
            },
            Card {
                rank: Rank::Ten,
                suit: Suit::Club,
            },
            Card {
                rank: Rank::Ten,
                suit: Suit::Club,
            },
            Card {
                rank: Rank::Ten,
                suit: Suit::Club,
            },
        ];

        assert_eq!(
            Scorer::get_scoring_hand(&test_cards).unwrap().0.unwrap(),
            ScoringHand::FlushFive
        );
    }

    #[test]
    fn score_flush_house() {
        let test_cards = vec![
            Card {
                rank: Rank::Eight,
                suit: Suit::Club,
            },
            Card {
                rank: Rank::Eight,
                suit: Suit::Club,
            },
            Card {
                rank: Rank::Eight,
                suit: Suit::Club,
            },
            Card {
                rank: Rank::Three,
                suit: Suit::Club,
            },
            Card {
                rank: Rank::Three,
                suit: Suit::Club,
            },
        ];

        assert_eq!(
            Scorer::get_scoring_hand(&test_cards).unwrap().0.unwrap(),
            ScoringHand::FlushHouse
        );
    }

    #[test]
    fn score_five_of_a_kind() {
        let test_cards = vec![
            Card {
                rank: Rank::Ten,
                suit: Suit::Club,
            },
            Card {
                rank: Rank::Ten,
                suit: Suit::Heart,
            },
            Card {
                rank: Rank::Ten,
                suit: Suit::Diamond,
            },
            Card {
                rank: Rank::Ten,
                suit: Suit::Spade,
            },
            Card {
                rank: Rank::Ten,
                suit: Suit::Club,
            },
        ];

        assert_eq!(
            Scorer::get_scoring_hand(&test_cards).unwrap().0.unwrap(),
            ScoringHand::FiveOfAKind
        );
    }

    #[test]
    fn score_royal_flush() {
        let test_cards = vec![
            Card {
                rank: Rank::Queen,
                suit: Suit::Club,
            },
            Card {
                rank: Rank::Ten,
                suit: Suit::Club,
            },
            Card {
                rank: Rank::Ace,
                suit: Suit::Club,
            },
            Card {
                rank: Rank::Jack,
                suit: Suit::Club,
            },
            Card {
                rank: Rank::King,
                suit: Suit::Club,
            },
        ];

        assert_eq!(
            Scorer::get_scoring_hand(&test_cards).unwrap().0.unwrap(),
            ScoringHand::RoyalFlush
        );
    }

    #[test]
    fn score_straight_flush() {
        let test_cards = vec![
            Card {
                rank: Rank::Eight,
                suit: Suit::Club,
            },
            Card {
                rank: Rank::Five,
                suit: Suit::Club,
            },
            Card {
                rank: Rank::Four,
                suit: Suit::Club,
            },
            Card {
                rank: Rank::Six,
                suit: Suit::Club,
            },
            Card {
                rank: Rank::Seven,
                suit: Suit::Club,
            },
        ];

        assert_eq!(
            Scorer::get_scoring_hand(&test_cards).unwrap().0.unwrap(),
            ScoringHand::StraightFlush
        );
    }

    #[test]
    fn score_four_of_a_kind() {
        let test_cards = vec![
            Card {
                rank: Rank::Seven,
                suit: Suit::Club,
            },
            Card {
                rank: Rank::Seven,
                suit: Suit::Heart,
            },
            Card {
                rank: Rank::Seven,
                suit: Suit::Diamond,
            },
            Card {
                rank: Rank::Seven,
                suit: Suit::Spade,
            },
            Card {
                rank: Rank::Three,
                suit: Suit::Club,
            },
        ];

        assert_eq!(
            Scorer::get_scoring_hand(&test_cards).unwrap().0.unwrap(),
            ScoringHand::FourOfAKind
        );
    }

    #[test]
    fn score_full_house() {
        let test_cards = vec![
            Card {
                rank: Rank::Eight,
                suit: Suit::Club,
            },
            Card {
                rank: Rank::Eight,
                suit: Suit::Club,
            },
            Card {
                rank: Rank::Eight,
                suit: Suit::Club,
            },
            Card {
                rank: Rank::Three,
                suit: Suit::Diamond,
            },
            Card {
                rank: Rank::Three,
                suit: Suit::Diamond,
            },
        ];

        assert_eq!(
            Scorer::get_scoring_hand(&test_cards).unwrap().0.unwrap(),
            ScoringHand::FullHouse
        );
    }

    #[test]
    fn score_flush() {
        let test_cards = vec![
            Card {
                rank: Rank::Eight,
                suit: Suit::Club,
            },
            Card {
                rank: Rank::Five,
                suit: Suit::Club,
            },
            Card {
                rank: Rank::Jack,
                suit: Suit::Club,
            },
            Card {
                rank: Rank::Seven,
                suit: Suit::Club,
            },
            Card {
                rank: Rank::Three,
                suit: Suit::Club,
            },
        ];

        assert_eq!(
            Scorer::get_scoring_hand(&test_cards).unwrap().0.unwrap(),
            ScoringHand::Flush
        );
    }

    #[test]
    fn score_non_ace_straight() {
        let test_cards = vec![
            Card {
                rank: Rank::Eight,
                suit: Suit::Diamond,
            },
            Card {
                rank: Rank::Five,
                suit: Suit::Club,
            },
            Card {
                rank: Rank::Four,
                suit: Suit::Spade,
            },
            Card {
                rank: Rank::Six,
                suit: Suit::Heart,
            },
            Card {
                rank: Rank::Seven,
                suit: Suit::Club,
            },
        ];

        assert_eq!(
            Scorer::get_scoring_hand(&test_cards).unwrap().0.unwrap(),
            ScoringHand::Straight
        );
    }

    #[test]
    fn score_low_ace_straight() {
        let test_cards = vec![
            Card {
                rank: Rank::Four,
                suit: Suit::Diamond,
            },
            Card {
                rank: Rank::Three,
                suit: Suit::Club,
            },
            Card {
                rank: Rank::Ace,
                suit: Suit::Spade,
            },
            Card {
                rank: Rank::Two,
                suit: Suit::Heart,
            },
            Card {
                rank: Rank::Five,
                suit: Suit::Club,
            },
        ];

        assert_eq!(
            Scorer::get_scoring_hand(&test_cards).unwrap().0.unwrap(),
            ScoringHand::Straight
        );
    }

    #[test]
    fn score_high_ace_straight() {
        let test_cards = vec![
            Card {
                rank: Rank::Ten,
                suit: Suit::Diamond,
            },
            Card {
                rank: Rank::Queen,
                suit: Suit::Club,
            },
            Card {
                rank: Rank::Ace,
                suit: Suit::Spade,
            },
            Card {
                rank: Rank::King,
                suit: Suit::Heart,
            },
            Card {
                rank: Rank::Jack,
                suit: Suit::Club,
            },
        ];

        assert_eq!(
            Scorer::get_scoring_hand(&test_cards).unwrap().0.unwrap(),
            ScoringHand::Straight
        );
    }

    #[test]
    fn score_mid_ace_straight_false_positive() {
        let test_cards = vec![
            Card {
                rank: Rank::Two,
                suit: Suit::Diamond,
            },
            Card {
                rank: Rank::Ace,
                suit: Suit::Club,
            },
            Card {
                rank: Rank::Three,
                suit: Suit::Spade,
            },
            Card {
                rank: Rank::King,
                suit: Suit::Heart,
            },
            Card {
                rank: Rank::Queen,
                suit: Suit::Club,
            },
        ];

        assert_eq!(
            Scorer::get_scoring_hand(&test_cards).unwrap().0.unwrap(),
            ScoringHand::HighCard
        );
    }

    #[test]
    fn score_three_of_a_kind() {
        let test_cards = vec![
            Card {
                rank: Rank::Eight,
                suit: Suit::Club,
            },
            Card {
                rank: Rank::Eight,
                suit: Suit::Diamond,
            },
            Card {
                rank: Rank::Eight,
                suit: Suit::Heart,
            },
            Card {
                rank: Rank::Six,
                suit: Suit::Spade,
            },
            Card {
                rank: Rank::Three,
                suit: Suit::Diamond,
            },
        ];

        assert_eq!(
            Scorer::get_scoring_hand(&test_cards).unwrap().0.unwrap(),
            ScoringHand::ThreeOfAKind
        );
    }

    #[test]
    fn score_two_pair() {
        let test_cards = vec![
            Card {
                rank: Rank::Eight,
                suit: Suit::Club,
            },
            Card {
                rank: Rank::Eight,
                suit: Suit::Diamond,
            },
            Card {
                rank: Rank::Six,
                suit: Suit::Heart,
            },
            Card {
                rank: Rank::Six,
                suit: Suit::Spade,
            },
            Card {
                rank: Rank::Three,
                suit: Suit::Diamond,
            },
        ];

        assert_eq!(
            Scorer::get_scoring_hand(&test_cards).unwrap().0.unwrap(),
            ScoringHand::TwoPair
        );
    }

    #[test]
    fn score_pair() {
        let test_cards = vec![
            Card {
                rank: Rank::Eight,
                suit: Suit::Club,
            },
            Card {
                rank: Rank::Eight,
                suit: Suit::Diamond,
            },
            Card {
                rank: Rank::Seven,
                suit: Suit::Heart,
            },
            Card {
                rank: Rank::Six,
                suit: Suit::Spade,
            },
            Card {
                rank: Rank::Three,
                suit: Suit::Diamond,
            },
        ];

        assert_eq!(
            Scorer::get_scoring_hand(&test_cards).unwrap().0.unwrap(),
            ScoringHand::Pair
        );
    }

    #[test]
    fn score_high_card() {
        let test_cards = vec![
            Card {
                rank: Rank::Jack,
                suit: Suit::Club,
            },
            Card {
                rank: Rank::Eight,
                suit: Suit::Diamond,
            },
            Card {
                rank: Rank::Seven,
                suit: Suit::Heart,
            },
            Card {
                rank: Rank::Six,
                suit: Suit::Spade,
            },
            Card {
                rank: Rank::Three,
                suit: Suit::Diamond,
            },
        ];

        assert_eq!(
            Scorer::get_scoring_hand(&test_cards).unwrap().0.unwrap(),
            ScoringHand::HighCard
        );
    }
}
