//! [`Game`] is a logical wrapper around the main flow of the game, ie, [`Run`].
//! [`Game`] provides additional functionalities outside of the lifetime of an
//! instance of [`Run`].
//!
//! The entrypoint of game is [`Game::new()`] to create the instance of a new
//! game and [`Game::start()`] to spawn a new instance of a running game.

use std::{
    num::NonZeroUsize,
    str::FromStr,
    sync::{Arc, RwLock},
};

use balatro_tui_core::{
    blind::Blind,
    card::Card,
    deck::{Deck, DeckConstExt},
    round::{Round, RoundProperties},
    run::{Run, RunProperties, RunState},
    scorer::Scorer,
};
use balatro_tui_widgets::{
    CardListWidget, CardListWidgetState, RoundInfoWidget, RoundScoreWidget, RunStatsWidget,
    RunStatsWidgetState, ScorerPreviewWidget, ScorerPreviewWidgetState, SelectableList,
    SplashScreenWidget,
};
use color_eyre::{
    Result,
    eyre::{Context, OptionExt, bail},
};
use crossterm::event::{KeyCode, KeyModifiers};
use rand::{
    distributions::{Alphanumeric, DistString},
    thread_rng,
};
use ratatui::{
    Frame,
    layout::{Constraint, Flex, Layout, Margin, Rect},
    style::Color,
    widgets::{Block, BorderType, Borders},
};

use crate::{
    event::{Event, EventHandler},
    iter_index_ext::IterIndexExt,
    tui::Tui,
};

/// Tick rate at which the game runs/receives updates.
pub const TICK_RATE: u64 = 144;

/// Maximum selectable cards to form a hand.
///
/// As per standard rules this is set to `5`.
pub const MAXIMUM_SELECTABLE_CARDS: usize = 5;

/// [`Game`] struct holds the state for the running game, including [`Run`]
/// surrounding states, that allow early closure of a run.
#[derive(Clone, Debug)]
pub struct Game {
    /// An instance of a [`Run`]. The run is the actual handler for most
    /// operations. [`Game`] simply forwards the requests to [`Run`] to handle.
    run: Run,
    /// A cached card list widget state. This caching is required for showing
    /// selection and hovering for [`CardListWidget`].
    card_list_widget_state: Option<CardListWidgetState>,
}

impl Game {
    /// Create a new instance of a game.
    ///
    /// This acts as a no-parameter initialization point and should be placed
    /// between user initialization and persistent on-disk configurations.
    #[must_use = "Created game instance must be used."]
    #[inline]
    pub fn new() -> Result<Self> {
        let deck = Arc::new(RwLock::new(Deck::standard()));
        let max_discards = 3;
        let max_hands = 3;
        let run_properties = RunProperties {
            hand_size: 10,
            max_discards,
            max_hands,
            seed: Alphanumeric.sample_string(&mut thread_rng(), 16),
            starting_money: 10,
        };
        let round_properties = RoundProperties {
            hand_size: 10,
            ante: NonZeroUsize::new(1).ok_or_eyre("Could not create ante number")?,
            round_number: NonZeroUsize::new(1).ok_or_eyre("Could not create round number")?,
        };
        Ok(Self {
            run: Run {
                deck: Arc::clone(&deck),
                run_state: RunState::Running,
                money: run_properties.starting_money,
                properties: run_properties,
                round: Round {
                    blind: Blind::Small,
                    deck: Arc::clone(&deck),
                    discards_count: max_discards,
                    hand: Arc::new(RwLock::new(vec![])),
                    hands_count: max_hands,
                    history: vec![],
                    properties: round_properties,
                    score: 0,
                },
                upcoming_round_number: NonZeroUsize::new(1)
                    .ok_or_eyre("Could not create upcoming round number")?,
            },
            card_list_widget_state: None,
        })
    }

    /// Main entrypoint of the game.
    ///
    /// Creates a new [`Tui`] instance and initializes the [`EventHandler`].
    /// Runs the round initialization routine and the game `update` loop
    pub async fn start(&mut self) -> Result<()> {
        // Enter TUI
        let mut tui = Tui::new()?;
        tui.enter().wrap_err("Error occurred while entering Tui")?;

        // Spawn EventHandler
        let mut event_handler = EventHandler::new(TICK_RATE);

        // Start a run
        self.run.start()?;

        // Cached card state
        self.card_list_widget_state = Some(
            CardListWidgetState::from(Arc::<RwLock<Vec<Card>>>::clone(&self.run.round.hand))
                .selection_limit(Some(MAXIMUM_SELECTABLE_CARDS))?,
        );

        // Draw loop
        loop {
            let mut send_result: Result<()> = Ok(());

            let event = event_handler.next().await?;

            let continue_game = Self::evaluate_exit(event, |ev: Event| {
                send_result = event_handler.send_event(ev);
            })?;

            send_result?;

            self.handle_run_events(event)?;
            self.handle_round_events(event)?;
            self.handle_deck_events(event)?;

            let mut draw_result: Result<()> = Ok(());

            _ = tui
                .draw(|frame| {
                    draw_result = self.draw(frame, frame.area());
                })
                .wrap_err("Could not draw on Tui buffer.")?;

            draw_result?;

            if !continue_game {
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
    #[expect(
        clippy::too_many_lines,
        reason = "Refactor: Create CoreRenderer structs to render core widgets."
    )]
    fn draw(&mut self, frame: &mut Frame<'_>, area: Rect) -> Result<()> {
        // Prepare variables
        let scoring_hand_opt = Scorer::get_scoring_hand(
            &self
                .run
                .round
                .hand
                .try_read()
                .or_else(|err| bail!("Could not attain read lock for hand: {err}."))?
                .peek_at_index_set(
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
        let mut splash_state_area = Layout::vertical([Constraint::Ratio(2, 3)])
            .flex(Flex::Center)
            .areas::<1>(area)[0];
        splash_state_area = Layout::horizontal([Constraint::Ratio(2, 3)])
            .flex(Flex::Center)
            .areas::<1>(splash_state_area)[0];
        let [meta_area, play_area] =
            Layout::horizontal([Constraint::Percentage(25), Constraint::Fill(1)]).areas(area);
        let [
            round_info_area,
            round_score_area,
            scoring_area,
            run_stats_area,
        ] = Layout::vertical([
            Constraint::Length(15),
            Constraint::Length(9),
            Constraint::Length(12),
            Constraint::Length(17),
        ])
        .flex(Flex::Center)
        .areas(meta_area.inner(Margin::new(1, 0)));
        let [_, deck_area] =
            Layout::vertical([Constraint::Fill(1), Constraint::Length(10)]).areas(play_area);

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
                level: NonZeroUsize::new(1)
                    .ok_or_eyre("Unable to create a non zero usize for level")?,
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
        frame.render_stateful_widget(
            CardListWidget::new(),
            deck_area,
            self.card_list_widget_state
                .as_mut()
                .ok_or_eyre("Card list widget state not initialized yet.")?,
        );

        match self.run.run_state {
            RunState::Running => (),
            RunState::Finished(win) => {
                if win {
                    frame.render_stateful_widget(
                        SplashScreenWidget::new()
                            .splash("Congratulations!")
                            .message("You won the game!"),
                        splash_state_area,
                        &mut vec![("Money collected", &self.run.money.to_string())],
                    );
                } else {
                    frame.render_stateful_widget(
                        SplashScreenWidget::new()
                            .splash("Game Over")
                            .message("You lost the game!"),
                        splash_state_area,
                        &mut vec![
                            (
                                "Last round reached",
                                &self.run.round.properties.round_number.to_string(),
                            ),
                            (
                                "Last ante reached",
                                &self.run.round.properties.ante.to_string(),
                            ),
                        ],
                    );
                }
            }
        }

        Ok(())
    }

    /// Event handler for handling game-specific input interface events.
    ///
    /// Returns a [`Result<bool>`] where the boolean value indicates whether to
    /// continue the game loop.
    fn evaluate_exit<S>(event: Event, send: S) -> Result<bool>
    where
        S: FnOnce(Event),
    {
        #[expect(
            clippy::wildcard_enum_match_arm,
            reason = "Intended: Unused events may skip implementation as required."
        )]
        match event {
            Event::Key(key_event) => match key_event.code {
                KeyCode::Esc | KeyCode::Char('q') => {
                    send(Event::Exit);
                }
                KeyCode::Char('c' | 'C') => {
                    if key_event.modifiers == KeyModifiers::CONTROL {
                        send(Event::Exit);
                    }
                }
                _ => (),
            },
            Event::Resize(x_size, y_size) => {
                if y_size < 40 || x_size < 150 {
                    bail!(
                        "Terminal size was less than required to render game. Need at least 150x40 character screen to render."
                    );
                }
            }
            Event::Exit => return Ok(false),
            _ => (),
        }

        Ok(true)
    }

    /// Event handler for handling run-specific input interface events.
    fn handle_run_events(&mut self, event: Event) -> Result<()> {
        if event == Event::Tick {
            if self.run.round.hands_count == 0
                && self.run.round.score
                    < self
                        .run
                        .round
                        .blind
                        .get_target_score(self.run.round.properties.ante)?
            {
                self.run.run_state = RunState::Finished(false);
            }

            if self.run.round.score
                >= self
                    .run
                    .round
                    .blind
                    .get_target_score(self.run.round.properties.ante)?
            {
                self.run.run_state = RunState::Finished(true);
            }
        }

        Ok(())
    }

    /// Event handler for handling round-specific input interface events.
    fn handle_round_events(&mut self, event: Event) -> Result<()> {
        #[expect(
            clippy::wildcard_enum_match_arm,
            reason = "Intended: Unused events may skip implementation as required."
        )]
        if let Event::Key(key_event) = event {
            match key_event.code {
                KeyCode::Enter => {
                    if self.run.round.hands_count != 0 {
                        let mut selected = self
                            .run
                            .round
                            .hand
                            .try_write()
                            .or_else(|err| bail!("Could not attain read lock for hand: {err}."))?
                            .drain_from_index_set(
                                &self
                                    .card_list_widget_state
                                    .as_ref()
                                    .ok_or_eyre("Card list widget state not initialized yet.")?
                                    .selected,
                            )?;

                        if selected.is_empty() {
                            return Ok(());
                        }

                        self.run.round.play_hand(&mut selected)?;
                        self.card_list_widget_state
                            .as_mut()
                            .ok_or_eyre("Card list widget state not initialized yet.")?
                            .set_cards(Arc::<RwLock<Vec<Card>>>::clone(&self.run.round.hand));
                    }
                }
                KeyCode::Char('x') => {
                    if self.run.round.discards_count != 0 {
                        let mut selected = self
                            .run
                            .round
                            .hand
                            .try_write()
                            .or_else(|err| bail!("Could not attain write lock for hand: {err}."))?
                            .drain_from_index_set(
                                &self
                                    .card_list_widget_state
                                    .as_ref()
                                    .ok_or_eyre("Card list widget state not initialized yet.")?
                                    .selected,
                            )?;

                        if selected.is_empty() {
                            return Ok(());
                        }

                        self.run.round.discard_hand(&mut selected)?;
                        self.card_list_widget_state
                            .as_mut()
                            .ok_or_eyre("Card list widget state not initialized yet.")?
                            .set_cards(Arc::<RwLock<Vec<Card>>>::clone(&self.run.round.hand));
                    }
                }
                _ => (),
            }
        }

        Ok(())
    }

    /// Event handler for handling deck-specific input interface events.
    fn handle_deck_events(&mut self, event: Event) -> Result<()> {
        #[expect(
            clippy::wildcard_enum_match_arm,
            reason = "Intended: Unused events may skip implementation as required."
        )]
        if let Event::Key(key_event) = event {
            match key_event.code {
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
            }
        }

        Ok(())
    }
}
