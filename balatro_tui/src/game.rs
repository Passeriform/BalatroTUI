//! [`Game`] is a logical wrapper around the main flow of the game, ie, [`Run`].
//! [`Game`] provides additional functionalities outside of the lifetime of an
//! instance of [`Run`].
//!
//! The entrypoint of game is [`Game::new()`] to create the instance of a new
//! game and [`Game::start()`] to spawn a new instance of a running game.

use std::{
    collections::HashSet,
    str::FromStr,
    sync::{Arc, Mutex, RwLock},
};

use balatro_tui_core::{
    blind::Blind,
    card::Card,
    deck::{Deck, DeckConstExt},
    round::{Round, RoundProperties},
    run::{Run, RunProperties},
    scorer::Scorer,
};
use balatro_tui_widgets::{
    CardListWidget, CardListWidgetState, RoundInfoWidget, RoundScoreWidget, RunStatsWidget,
    RunStatsWidgetState, ScorerPreviewWidget, ScorerPreviewWidgetState, SelectableList,
};
use color_eyre::{
    eyre::{bail, Context, OptionExt},
    Result,
};
use crossterm::event::{KeyCode, KeyModifiers};
use itertools::{Either, Itertools};
use ratatui::{
    layout::{Constraint, Flex, Layout, Margin, Rect},
    style::Color,
    widgets::{Block, BorderType, Borders},
    Frame,
};

use crate::{
    event::{Event, EventHandler},
    tui::Tui,
};

// TODO: Add compatibility with non-tui solution

/// Tick rate at which the game runs/receives updates.
pub const TICK_RATE: u64 = 144;

/// Maximum selectable cards to form a hand.
///
/// As per standard rules this is set to `5`.
pub const MAXIMUM_SELECTABLE_CARDS: usize = 5;

/// [`Game`] struct holds the state for the running game, including [`Run`]
/// surrounding states, that allow early closure of a run.
#[expect(
    clippy::partial_pub_fields,
    reason = "Intended: Card list widget is an internal field only accessible by the Game instance."
)]
#[derive(Debug)]
pub struct Game {
    /// An instance of a [`Run`]. The run is the actual handler for most
    /// operations. [`Game`] simply forwards the requests to [`Run`] to handle.
    pub run: Run,
    /// A boolean flag denoting whether the game should send out shutdown
    /// signal.
    pub should_quit: bool,
    /// A cached card list widget state. This caching is required for showing
    /// selection and hovering for [`CardListWidget`].
    pub(self) card_list_widget_state: Option<CardListWidgetState>,
}

impl Game {
    /// Create a new instance of a game.
    ///
    /// This acts as a no-parameter initialization point and should be placed
    /// between user initialization and persistent on-disk configurations.
    #[expect(
        clippy::new_without_default,
        reason = "Intended: Game should only be explicitly created hence default implementation can be skipped."
    )]
    #[must_use = "Created game instance must be used."]
    #[inline]
    pub fn new() -> Self {
        let deck = Arc::new(RwLock::new(Deck::standard()));
        let run_properties = RunProperties::default();
        let max_hands = run_properties.max_hands;
        let max_discards = run_properties.max_discards;
        Self {
            run: Run {
                deck: Arc::clone(&deck),
                money: run_properties.starting_money,
                properties: run_properties,
                round: Round {
                    blind: Blind::Small,
                    deck: Arc::clone(&deck),
                    discards_count: max_discards,
                    hand: vec![],
                    hands_count: max_hands,
                    history: vec![],
                    properties: RoundProperties::default(),
                    score: 0,
                },
                upcoming_round_number: 1,
            },
            should_quit: false,
            card_list_widget_state: None,
        }
    }

    /// Main entrypoint of the game.
    ///
    /// Creates a new [`Tui`] instance and initializes the [`EventHandler`].
    /// Runs the round initialization routine and the game `update` loop
    pub async fn start(&mut self) -> Result<()> {
        // Enter TUI
        let mut tui = Tui::new()?;
        tui.enter()?;

        // Spawn EventHandler
        let mut events = EventHandler::new(TICK_RATE);

        // Start a run
        self.run.start()?;

        // Cached card state
        self.card_list_widget_state = Some(
            CardListWidgetState::from(Arc::from(Mutex::from(self.run.round.hand.clone())))
                .selection_limit(Some(MAXIMUM_SELECTABLE_CARDS))?,
        );

        // Draw loop
        loop {
            self.handle_events(events.next().await?)?;

            let mut inner_result: Result<()> = Ok(());

            _ = tui
                .draw(|frame| {
                    inner_result = self
                        .draw(frame, frame.area())
                        .wrap_err("Could not draw game on the given frame.");
                })
                .wrap_err("Could not draw on Tui buffer.")?;

            inner_result?;

            if self.should_quit {
                break;
            }
        }

        // Exit TUI
        tui.exit()?;

        Ok(())
    }

    /// Draw loop for game state
    ///
    /// Runs every tick provided by the rendering interface.
    fn draw(&mut self, frame: &mut Frame<'_>, area: Rect) -> Result<()> {
        // Prepare variables
        // TODO: Pass these from outside or implement caching to avoid needless calls.
        let scoring_hand_opt = Scorer::get_scoring_hand(
            &self.run.round.hand.peek_at_index_set(
                &self
                    .card_list_widget_state
                    .as_ref()
                    .ok_or_eyre("Card list widget state not initialized yet.")?
                    .selected,
            )?,
        )?
        .0;
        let (chips, multiplier) = if let Some(scoring_hand) = scoring_hand_opt {
            Scorer::get_chips_and_multiplier(scoring_hand)?
        } else {
            (0, 0)
        };

        // Prepare areas
        let [meta_area, play_area] =
            Layout::horizontal([Constraint::Percentage(25), Constraint::Fill(1)]).areas(area);
        let [
            round_info_area,
            round_score_area,
            scoring_area,
            run_stats_area,
        ] = Layout::vertical([
            // TODO: Infer from content length
            Constraint::Length(15),
            Constraint::Length(9),
            Constraint::Length(12),
            Constraint::Length(17),
        ])
        .flex(Flex::Center)
        .areas(meta_area.inner(Margin::new(1, 0)));

        // Render containers
        frame.render_widget(
            Block::new().borders(Borders::LEFT | Borders::RIGHT),
            meta_area,
        );
        frame.render_widget(
            Block::bordered().border_type(BorderType::Rounded),
            round_info_area,
        );
        frame.render_widget(
            Block::bordered().border_type(BorderType::Rounded),
            round_score_area,
        );
        frame.render_widget(
            Block::bordered().border_type(BorderType::Rounded),
            scoring_area,
        );

        // Render widgets
        frame.render_widget(
            RoundInfoWidget::new()
                .blind_color(Color::from_str(self.run.round.blind.get_color()?)?)
                .blind_text(self.run.round.blind.to_string())
                .reward(self.run.round.blind.get_reward()?)
                .target_score(
                    self.run
                        .round
                        .blind
                        .get_target_score(self.run.round.properties.ante)?,
                ),
            round_info_area.inner(Margin::new(1, 1)),
        );
        frame.render_stateful_widget(
            RoundScoreWidget::new(),
            round_score_area.inner(Margin::new(1, 1)),
            &mut self.run.round.score,
        );
        frame.render_stateful_widget(
            ScorerPreviewWidget::new(),
            scoring_area.inner(Margin::new(1, 1)),
            &mut ScorerPreviewWidgetState {
                chips,
                level: 1,
                multiplier,
                scoring_hand_text: scoring_hand_opt.map(|scoring_hand| scoring_hand.to_string()),
            },
        );
        frame.render_stateful_widget(
            RunStatsWidget::new(),
            run_stats_area,
            &mut RunStatsWidgetState {
                hands: self.run.round.hands_count,
                discards: self.run.round.discards_count,
                money: self.run.money,
                ante: self.run.round.properties.ante,
                round: self.run.round.properties.round_number,
            },
        );

        // TODO: Use ListWidget to handle selection instead.

        //////////////////////////////////////////////////////////////////////////

        let [_, deck_area] =
            Layout::vertical([Constraint::Fill(1), Constraint::Length(10)]).areas(play_area);

        frame.render_stateful_widget(
            CardListWidget::new(),
            deck_area,
            self.card_list_widget_state
                .as_mut()
                .ok_or_eyre("Card list widget state not initialized yet.")?,
        );

        Ok(())
    }

    // TODO: Split and move into separate event handler + render traits.
    /// Centralized event handler working on state
    fn handle_events(&mut self, event: Event) -> Result<()> {
        #[expect(
            clippy::wildcard_enum_match_arm,
            reason = "Intended: Unused events may skip implementation as required."
        )]
        match event {
            Event::Key(key_event) => match key_event.code {
                KeyCode::Esc | KeyCode::Char('q') => {
                    self.should_quit = true;
                }
                KeyCode::Char('c' | 'C') => {
                    if key_event.modifiers == KeyModifiers::CONTROL {
                        self.should_quit = true;
                    }
                }
                //////////////////////////////////////////////////////////////////////////
                KeyCode::Enter => {
                    if self.run.round.hands_count != 0 {
                        let mut selected = self.run.round.hand.drain_from_index_set(
                            &self
                                .card_list_widget_state
                                .as_ref()
                                .ok_or_eyre("Card list widget state not initialized yet.")?
                                .selected,
                        )?;
                        self.run.round.play_hand(&mut selected)?;
                        self.card_list_widget_state
                            .as_mut()
                            .ok_or_eyre("Card list widget state not initialized yet.")?
                            .set_cards(Arc::from(Mutex::from(self.run.round.hand.clone())));
                    }
                }
                KeyCode::Char('x') => {
                    if self.run.round.discards_count != 0 {
                        let mut selected = self.run.round.hand.drain_from_index_set(
                            &self
                                .card_list_widget_state
                                .as_ref()
                                .ok_or_eyre("Card list widget state not initialized yet.")?
                                .selected,
                        )?;
                        self.run.round.discard_hand(&mut selected)?;
                        self.card_list_widget_state
                            .as_mut()
                            .ok_or_eyre("Card list widget state not initialized yet.")?
                            .set_cards(Arc::from(Mutex::from(self.run.round.hand.clone())));
                    }
                }
                //////////////////////////////////////////////////////////////////////////
                KeyCode::Right => {
                    if let Some(state) = self.card_list_widget_state.as_mut() {
                        state.move_next()?;
                    }
                }
                KeyCode::Left => {
                    if let Some(state) = self.card_list_widget_state.as_mut() {
                        state.move_prev()?;
                    }
                }
                KeyCode::Up => {
                    _ = self
                        .card_list_widget_state
                        .as_mut()
                        .ok_or_eyre("Card list widget state not initialized yet.")?
                        .select()?;
                }
                KeyCode::Down => {
                    _ = self
                        .card_list_widget_state
                        .as_mut()
                        .ok_or_eyre("Card list widget state not initialized yet.")?
                        .deselect()?;
                }
                _ => (),
            },
            Event::Resize(x_size, y_size) => {
                if y_size < 40 || x_size < 150 {
                    bail!("Terminal size was less than required to render game.");
                }
            }
            _ => (),
        }

        Ok(())
    }
}

// TODO: Move to utility on crate separation.
/// Provides methods to perform container/iterator methods based on index hash
/// set.
trait HashedContainer
where
    Self: IntoIterator + Sized,
{
    /// Returns a cloned [`Vec`] based on arbitrary indices set.
    fn peek_at_index_set(&self, index_set: &HashSet<usize>) -> Result<Self>;
    /// Drains the iterator based on arbitrary indices (see [`Vec::drain()`] for
    /// equivalent usage with contiguous range) and returns the drained items in
    /// a [`Vec`].
    fn drain_from_index_set(&mut self, index_set: &HashSet<usize>) -> Result<Self>;
}

impl HashedContainer for Vec<Card> {
    fn peek_at_index_set(&self, index_set: &HashSet<usize>) -> Result<Self> {
        index_set
            .iter()
            .map(|&idx| {
                self.get(idx)
                    .copied()
                    .ok_or_eyre("Invalid index accessed. Index set may be invalid.")
            })
            .process_results(|iter| iter.collect())
    }

    fn drain_from_index_set(&mut self, index_set: &HashSet<usize>) -> Result<Self> {
        let (selected, leftover): (Self, Self) = self
            .iter()
            .enumerate()
            .map(|(idx, &card)| (idx, card))
            .partition_map(|(idx, card)| {
                if index_set.contains(&idx) {
                    Either::Left(card)
                } else {
                    Either::Right(card)
                }
            });

        *self = leftover;

        Ok(selected)
    }
}
