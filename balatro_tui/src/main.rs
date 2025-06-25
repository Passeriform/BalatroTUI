//! Balatro TUI
//!
//! A toy TUI based implementation for the game `Balatro` by [LocalThunk](https://x.com/LocalThunk).
//!
//! All rights are reserved by `LocalThunk` for the original game.

use color_eyre::{Result, eyre::Context};
use game::Game;

pub mod event;
pub mod game;
pub mod iter_index_ext;
pub mod tui;

#[tokio::main]
async fn main() -> Result<()> {
    // Start Game
    let mut game = Game::new()?;
    game.start()
        .await
        .wrap_err("Error encountered while running the game.")?;

    Ok(())
}
