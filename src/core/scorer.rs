use std::{collections::HashMap, error::Error, fmt::{Display, Formatter, Result as FmtResult}};

use itertools::Itertools;
use strum::EnumIter;

use super::card::{Card, Rank, Suit};

#[derive(Debug, Clone, Copy, Ord, PartialOrd, Eq, Hash, PartialEq, EnumIter)]
pub enum ScoringHand {
    FlushFive = 0,
    FlushHouse,
    FiveOfAKind,
    RoyalFlush, // GroupSuit(5) | SeqRank(5) + Ace
    StraightFlush, // GroupSuit(5) | SeqRank(5)
    FourOfAKind,
    FullHouse,
    Flush,
    Straight, // SeqRank(5)
    ThreeOfAKind,
    TwoPair,
    Pair,
    HighCard,
    None,
}

impl Display for ScoringHand {
    #[inline]
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        let display = match *self {
            ScoringHand::FlushFive => "Flush Five",
            ScoringHand::FlushHouse => "Flush House",
            ScoringHand::FiveOfAKind => "Five Of A Kind",
            ScoringHand::RoyalFlush => "Royal Flush",
            ScoringHand::StraightFlush => "Straight Flush",
            ScoringHand::FourOfAKind => "Four Of A Kind",
            ScoringHand::FullHouse => "Full House",
            ScoringHand::Flush => "Flush",
            ScoringHand::Straight => "Straight",
            ScoringHand::ThreeOfAKind => "Three Of A Kind",
            ScoringHand::TwoPair => "Two Pair",
            ScoringHand::Pair => "Pair",
            ScoringHand::HighCard => "High Card",
            ScoringHand::None => "",
        };
        write!(f, "{}", display)
    }
}

#[derive(Debug, Clone, Copy, Ord, PartialOrd, Eq, Hash, PartialEq)]
pub struct Scorer { }

impl Scorer {
    #[inline]
    pub const fn get_chips_and_multiplier(scoring_hand: ScoringHand) -> Result<(usize, usize), Box<dyn Error>> {
        Ok(match scoring_hand {
            ScoringHand::FlushFive => (160, 16),
            ScoringHand::FlushHouse => (140, 14),
            ScoringHand::FiveOfAKind => (120, 12),
            ScoringHand::RoyalFlush => (100, 8),
            ScoringHand::StraightFlush => (100, 8),
            ScoringHand::FourOfAKind => (60, 7),
            ScoringHand::FullHouse => (40, 4),
            ScoringHand::Flush => (35, 4),
            ScoringHand::Straight => (30, 4),
            ScoringHand::ThreeOfAKind => (30, 3),
            ScoringHand::TwoPair => (20, 2),
            ScoringHand::Pair => (10, 2),
            ScoringHand::HighCard => (5, 1),
            ScoringHand::None => (0, 0),
        })
    }

    #[inline]
    pub fn get_scoring_hand(cards: &Vec<Card>) -> Result<(ScoringHand, Vec<Rank>), Box<dyn Error>> {
        let suit_groups: Vec<(Suit, usize)> = cards.iter().fold(HashMap::new(), |mut groups, card| {
            groups.entry(card.suit)
            .and_modify(|e| *e += 1)
            .or_insert(1);
            groups
        }).into_iter().sorted_by(|a, b| b.1.cmp(&a.1)).collect();

        // TODO: Sort by rank before forming groups.
        let rank_groups: Vec<(Rank, usize)> = cards.iter().fold(HashMap::new(), |mut groups, card| {
            groups.entry(card.rank)
                .and_modify(|e| *e += 1)
                .or_insert(1);
            groups
        }).into_iter().sorted_by(|a, b| b.1.cmp(&a.1)).collect();

        if suit_groups.is_empty() || rank_groups.is_empty() {
            return Ok((ScoringHand::None, vec![]))
        }

        if suit_groups[0].1 == 5 && rank_groups[0].1 == 5 {
            return Ok((ScoringHand::FlushFive, vec![rank_groups[0].0; rank_groups[0].1]))
        }

        if rank_groups.len() >= 2 && suit_groups[0].1 == 5 && rank_groups[0].1 == 3 && rank_groups[1].1 == 2 {
            // TODO: Generalize and abstract the played_ranks logic into helper. Accept range as input.
            let mut played_ranks = vec![];
            played_ranks.append(&mut vec![rank_groups[0].0; rank_groups[0].1]);
            played_ranks.append(&mut vec![rank_groups[1].0; rank_groups[1].1]);
            return Ok((ScoringHand::FlushHouse, played_ranks))
        }

        if rank_groups[0].1 == 5 {
            return Ok((ScoringHand::FiveOfAKind, vec![rank_groups[0].0; rank_groups[0].1]))
        }

        // TODO: Implement Royal Flush check
        // TODO: Implement Straight Flush check

        if rank_groups[0].1 == 4 {
            return Ok((ScoringHand::FourOfAKind, vec![rank_groups[0].0; rank_groups[0].1]))
        }

        if rank_groups.len() >= 2 && rank_groups[0].1 == 3 && rank_groups[1].1 == 2 {
            let mut played_ranks = vec![];
            played_ranks.append(&mut vec![rank_groups[0].0; rank_groups[0].1]);
            played_ranks.append(&mut vec![rank_groups[1].0; rank_groups[1].1]);
            return Ok((ScoringHand::FullHouse, played_ranks))
        }

        if suit_groups[0].1 == 5 {
            return Ok((ScoringHand::Flush, cards.iter().map(|card| card.rank).collect()))
        }

        // TODO: Implement Straight check
        // TODO: Implement Ace wrap-around for straight checks

        if rank_groups[0].1 == 3 {
            return Ok((ScoringHand::ThreeOfAKind, vec![rank_groups[0].0; rank_groups[0].1]))
        }

        if rank_groups.len() >= 2 && rank_groups[0].1 == 2 && rank_groups[1].1 == 2 {
            let mut played_ranks = vec![];
            played_ranks.append(&mut vec![rank_groups[0].0; rank_groups[0].1]);
            played_ranks.append(&mut vec![rank_groups[1].0; rank_groups[1].1]);
            return Ok((ScoringHand::TwoPair, played_ranks))
        }

        if rank_groups[0].1 == 2 {
            return Ok((ScoringHand::Pair, vec![rank_groups[0].0; rank_groups[0].1]))
        }


        Ok((ScoringHand::HighCard, vec![rank_groups[0].0; rank_groups[0].1]))
    }

    pub fn score_cards(cards: &Vec<Card>) -> Result<usize, Box<dyn Error>> {
        let (scoring_hand, scored_ranks) = Self::get_scoring_hand(&cards)?;
        let (base_chips, multiplier) = Self::get_chips_and_multiplier(scoring_hand)?;
        let chips_increment = Self::score_chips_from_ranks(&scored_ranks)?;
        Ok((base_chips + chips_increment) * multiplier)
    }

    #[inline]
    pub fn score_chips_from_ranks(ranks: &Vec<Rank>) -> Result<usize, Box<dyn Error>> {
        Ok(ranks.iter().fold(0, |acc, rank| acc + rank.get_score()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn score_flush_five() {
        let test_cards = vec![
            Card { rank: Rank::Ten, suit: Suit::Club },
            Card { rank: Rank::Ten, suit: Suit::Club },
            Card { rank: Rank::Ten, suit: Suit::Club },
            Card { rank: Rank::Ten, suit: Suit::Club },
            Card { rank: Rank::Ten, suit: Suit::Club },
        ];

        assert_eq!(Scorer::get_scoring_hand(&test_cards).unwrap().0, ScoringHand::FlushFive);
    }

    #[test]
    fn score_flush_house() {
        let test_cards = vec![
            Card { rank: Rank::Eight, suit: Suit::Club },
            Card { rank: Rank::Eight, suit: Suit::Club },
            Card { rank: Rank::Eight, suit: Suit::Club },
            Card { rank: Rank::Three, suit: Suit::Club },
            Card { rank: Rank::Three, suit: Suit::Club },
        ];

        assert_eq!(Scorer::get_scoring_hand(&test_cards).unwrap().0, ScoringHand::FlushHouse);
    }

    #[test]
    fn score_five_of_a_kind() {
        let test_cards = vec![
            Card { rank: Rank::Ten, suit: Suit::Club },
            Card { rank: Rank::Ten, suit: Suit::Heart },
            Card { rank: Rank::Ten, suit: Suit::Diamond },
            Card { rank: Rank::Ten, suit: Suit::Spade },
            Card { rank: Rank::Ten, suit: Suit::Club },
        ];

        assert_eq!(Scorer::get_scoring_hand(&test_cards).unwrap().0, ScoringHand::FiveOfAKind);
    }

    #[test]
    fn score_four_of_a_kind() {
        let test_cards = vec![
            Card { rank: Rank::Seven, suit: Suit::Club },
            Card { rank: Rank::Seven, suit: Suit::Heart },
            Card { rank: Rank::Seven, suit: Suit::Diamond },
            Card { rank: Rank::Seven, suit: Suit::Spade },
            Card { rank: Rank::Three, suit: Suit::Club },
        ];

        assert_eq!(Scorer::get_scoring_hand(&test_cards).unwrap().0, ScoringHand::FourOfAKind);
    }

    #[test]
    fn score_full_house() {
        let test_cards = vec![
            Card { rank: Rank::Eight, suit: Suit::Club },
            Card { rank: Rank::Eight, suit: Suit::Club },
            Card { rank: Rank::Eight, suit: Suit::Club },
            Card { rank: Rank::Three, suit: Suit::Diamond },
            Card { rank: Rank::Three, suit: Suit::Diamond },
        ];

        assert_eq!(Scorer::get_scoring_hand(&test_cards).unwrap().0, ScoringHand::FullHouse);
    }

    #[test]
    fn score_flush() {
        let test_cards = vec![
            Card { rank: Rank::Eight, suit: Suit::Club },
            Card { rank: Rank::Five, suit: Suit::Club },
            Card { rank: Rank::Jack, suit: Suit::Club },
            Card { rank: Rank::Seven, suit: Suit::Club },
            Card { rank: Rank::Three, suit: Suit::Club },
        ];

        assert_eq!(Scorer::get_scoring_hand(&test_cards).unwrap().0, ScoringHand::Flush);
    }

    #[test]
    fn score_three_of_a_kind() {
        let test_cards = vec![
            Card { rank: Rank::Eight, suit: Suit::Club },
            Card { rank: Rank::Eight, suit: Suit::Diamond },
            Card { rank: Rank::Eight, suit: Suit::Heart },
            Card { rank: Rank::Six, suit: Suit::Spade },
            Card { rank: Rank::Three, suit: Suit::Diamond },
        ];

        assert_eq!(Scorer::get_scoring_hand(&test_cards).unwrap().0, ScoringHand::ThreeOfAKind);
    }

    #[test]
    fn score_two_pair() {
        let test_cards = vec![
            Card { rank: Rank::Eight, suit: Suit::Club },
            Card { rank: Rank::Eight, suit: Suit::Diamond },
            Card { rank: Rank::Six, suit: Suit::Heart },
            Card { rank: Rank::Six, suit: Suit::Spade },
            Card { rank: Rank::Three, suit: Suit::Diamond },
        ];

        assert_eq!(Scorer::get_scoring_hand(&test_cards).unwrap().0, ScoringHand::TwoPair);
    }

    #[test]
    fn score_pair() {
        let test_cards = vec![
            Card { rank: Rank::Eight, suit: Suit::Club },
            Card { rank: Rank::Eight, suit: Suit::Diamond },
            Card { rank: Rank::Seven, suit: Suit::Heart },
            Card { rank: Rank::Six, suit: Suit::Spade },
            Card { rank: Rank::Three, suit: Suit::Diamond },
        ];

        assert_eq!(Scorer::get_scoring_hand(&test_cards).unwrap().0, ScoringHand::Pair);
    }

    #[test]
    fn score_high_card() {
        let test_cards = vec![
            Card { rank: Rank::Jack, suit: Suit::Club },
            Card { rank: Rank::Eight, suit: Suit::Diamond },
            Card { rank: Rank::Seven, suit: Suit::Heart },
            Card { rank: Rank::Six, suit: Suit::Spade },
            Card { rank: Rank::Three, suit: Suit::Diamond },
        ];

        assert_eq!(Scorer::get_scoring_hand(&test_cards).unwrap().0, ScoringHand::HighCard);
    }
}