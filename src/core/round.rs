use std::{error::Error, sync::{Arc, RwLock}};

use crossterm::event::KeyCode;
use ratatui::{layout::{Constraint, Layout, Rect}, Frame};

use crate::{event::Event, tui::TuiComponent};

use super::{blind::{Blind, BlindType}, card::{Card, Sortable}, deck::Deck, scorer::Scorer};

#[derive(Debug, Clone, Eq, Hash, PartialEq)]
pub struct RoundProperties {
    pub blind: Blind,
    pub discards: usize,
    pub hand_size: usize,
    pub hands: usize,
    pub round_number: usize,
}

impl Default for RoundProperties {
    #[inline]
    fn default() -> Self {
        Self {
            blind: Blind::new(BlindType::Small, 1).unwrap(),
            hand_size: 10,
            hands: 3,
            discards: 3,
            round_number: 1,
        }
    }
}

#[derive(Debug, Default)]
pub struct Round {
    pub properties: RoundProperties,
    pub deck: Arc<RwLock<Deck>>,
    pub score: usize,
    pub hand: Deck,
    pub history: Deck,
}

impl Round {
    pub fn start(&mut self) -> Result<(), Box<dyn Error>> {
        self.hand = self.deck.write().unwrap().draw_random(self.properties.hand_size)?;
        self.hand.cards.sort_by_rank();
        Ok(())
    }

    pub fn deal_cards(&mut self, mut last_cards: Vec<Card>) -> Result<(), Box<dyn Error>> {
        let mut new_cards = self.deck.write().unwrap().draw_random(last_cards.len())?;
        self.history.cards.append(&mut last_cards);
        self.hand.cards.append(&mut new_cards.cards);
        self.hand.cards.sort_by_rank();

        Ok(())
    }

    pub fn play_hand(&mut self) -> Result<(), Box<dyn Error>> {
        self.properties.hands -= 1;

        let played_cards = self.hand.draw_selected()?;

        self.score += Scorer::score_cards(&played_cards)?;

        self.deal_cards(played_cards)?;

        Ok(())
    }

    pub fn discard_hand(&mut self) -> Result<(), Box<dyn Error>> {
        self.properties.discards -= 1;

        let discarded_cards = self.hand.draw_selected()?;

        self.deal_cards(discarded_cards)?;

        Ok(())
    }
}

// TODO: Add a scorer animation area.
// TODO: Remove deep variable access, use accessor functions/split responsibilities.

// TODO: Migrate all TuiComponent impl to Widgets
impl TuiComponent for Round {
    fn draw(&mut self, frame: &mut Frame, rect: Rect) {
        let [play_area, deck_area] = Layout::vertical([Constraint::Fill(1), Constraint::Length(10)]).areas(rect);
        self.hand.draw(frame, deck_area);
    }

    fn handle_events(&mut self, event: Event) {
        if let Event::Key(key_event) = event {
            match key_event.code {
                KeyCode::Enter => self.play_hand().unwrap(),
                KeyCode::Char('x') => {
                    if self.properties.discards == 0 {
                        return;
                    }

                    self.discard_hand().unwrap()
                },
                _ => ()
            }
        }
        self.hand.handle_events(event);
    }
}