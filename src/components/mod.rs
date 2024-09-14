//! Widgets and components for displaying elements of `BalatroTUI` on the
//! terminal.

// TODO: Move into a separate sub-crate.

#![expect(
    clippy::missing_docs_in_private_items,
    reason = "Intended: This module's contents are re-exported."
)]

mod blind_badge;
mod card;
mod round_info;
mod round_score;
mod run_stats;
mod scorer_preview;
mod text_box;

pub use blind_badge::*;
pub use card::*;
pub use round_info::*;
pub use round_score::*;
pub use run_stats::*;
pub use scorer_preview::*;
pub use text_box::*;
