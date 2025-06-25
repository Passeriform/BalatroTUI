//! This module provides [`EventHandler`] struct that handles sending and
//! receiving events asynchronously.
//!
//! The event handler provides a multi-read, multi-write wrapper that returns
//! [`Event`] using [`EventHandler::next()`].

use std::time::Duration;

use color_eyre::eyre::{Context, OptionExt, Result};
use crossterm::event::{Event as CrosstermEvent, EventStream, KeyEvent, KeyEventKind, MouseEvent};
use futures::{FutureExt, StreamExt};
use tokio::{
    select, spawn,
    sync::mpsc::{UnboundedReceiver, UnboundedSender, unbounded_channel},
    task::JoinHandle,
    time::interval,
};
use tokio_util::sync::CancellationToken;

/// This enum specifies the different events that can be sent over the
/// [`EventHandler`].
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Event {
    /// [`Event::Tick`] event corresponds to a tick of the underlying game loop.
    /// This can be used for constant evaluation per tick. Consider creating a
    /// `select!` arm for this when making arbitrary checks on the state of the
    /// game loop.
    Tick,
    /// [`Event::Key`] event is sent when a key is pressed or released on the
    /// event-accepting interface.
    Key(KeyEvent),
    /// [`Event::Mouse`] event is sent when a mouse is moved or mouse buttons
    /// are pressed or released on the event-accepting interface.
    Mouse(MouseEvent),
    /// [`Event::Resize`] event is sent when the event-accepting interface is
    /// resized. Use this for recomputing render requirements.
    Resize(u16, u16),
    /// [`Event::Exit`] event is the last event sent automatically by the event
    /// handler. Use this for gracefully exiting the game loop.
    Exit,
}

/// [`EventHandler`] is a wrapper interface that keeps track of and propagates
/// events from an event-accepting interface to the game loop.
#[derive(Debug)]
pub struct EventHandler {
    /// Sender allows asynchronous, thread-safe sends to the event handler.
    sender: UnboundedSender<Event>,
    /// Receiver allows asynchronous, thread-safe consumption from the event
    /// handler.
    receiver: UnboundedReceiver<Event>,
    /// Taking from handler marks the end of the [`EventHandler`]. It safely
    /// closes both sender and receiver objects.
    handler: Option<JoinHandle<Result<()>>>,
    /// Signals cancellation from consumer to exit the [`EventHandler`].
    cancellation_token: CancellationToken,
}

impl EventHandler {
    /// Creates a new [`EventHandler`] instance and creates a handler for
    /// sending [`Event`] instances.
    #[must_use = "Created event handler instance must be used."]
    pub fn new(tick_rate: u64) -> Self {
        let tick_duration = Duration::from_millis(tick_rate);

        let (sender, receiver) = unbounded_channel();

        let cancellation_token = CancellationToken::new();

        let handler = spawn(Self::event_handler_future(
            tick_duration,
            sender.clone(),
            cancellation_token.clone(),
        ));

        Self {
            sender,
            receiver,
            handler: Some(handler),
            cancellation_token,
        }
    }

    /// Send an event to the event handler
    #[inline]
    pub fn send_event(&mut self, event: Event) -> Result<()> {
        self.sender
            .send(event)
            .wrap_err("Failed to send message into the event handler sender")
    }

    /// Event handler future to be spawned as a tokio task.
    async fn event_handler_future(
        tick_duration: Duration,
        sender: UnboundedSender<Event>,
        cancellation_token: CancellationToken,
    ) -> Result<()> {
        let mut reader = EventStream::new();
        let mut tick = interval(tick_duration);

        #[expect(
            clippy::pattern_type_mismatch,
            clippy::ignored_unit_patterns,
            clippy::integer_division_remainder_used,
            reason = "False positive: Tokio's select! macro has different semantics than match statements."
        )]
        loop {
            let tick_delay = tick.tick();
            let crossterm_event = reader.next().fuse();
            select! {
                _ = sender.closed() => {
                    break Ok(());
                }
                _ = tick_delay => {
                    sender.send(Event::Tick).wrap_err("Unable to send Tick event over sender channel.")?;
                }
                _ = cancellation_token.cancelled() => {
                    break Ok(());
                }
                Some(Ok(event)) = crossterm_event => {
                    match event {
                        CrosstermEvent::Key(key) => {
                            if key.kind == KeyEventKind::Press {
                                sender.send(Event::Key(key)).wrap_err("Unable to send Key event over sender channel.")?;
                            }
                        },
                        CrosstermEvent::Mouse(mouse) => {
                            sender.send(Event::Mouse(mouse)).wrap_err("Unable to send Mouse event over sender channel.")?;
                        },
                        CrosstermEvent::Resize(x, y) => {
                            sender.send(Event::Resize(x, y)).wrap_err("Unable to send Resize event over sender channel.")?;
                        },
                        CrosstermEvent::FocusLost
                        | CrosstermEvent::FocusGained
                        | CrosstermEvent::Paste(_) => { },
                    }
                }
            };
        }
    }

    /// Sends the next asynchronous [`Event`] instance.
    ///
    /// This requires an `await` call on the returning [`Event`] instance or it
    /// can be chained with other `async` tasks.
    pub async fn next(&mut self) -> Result<Event> {
        self.receiver
            .recv()
            .await
            .ok_or_eyre("Cannot receive next event.")
    }

    /// Halts the [`Event`] stream via a cancellation token and sends an
    /// [`Event::Exit`] event to gracefully exit the game loop.
    pub async fn stop(&mut self) -> Result<()> {
        self.cancellation_token.cancel();
        if let Some(handle) = self.handler.take() {
            return handle
                .await
                .wrap_err("Cannot attain join on event listener task handle.")?
                .wrap_err("An error occurred in the event handler future");
        }
        Ok(())
    }
}
