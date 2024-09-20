use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Flex, Layout, Rect},
    text::Line,
    widgets::{StatefulWidget, Widget},
};

use super::text_box::TextBoxWidget;

/// Content height for [`ScorerPreviewWidget`].
const SCORER_PREVIEW_CONTENT_HEIGHT: u16 = 10;

/// Render state for [`ScorerPreviewWidget`].
#[derive(Clone, Debug, Default)]
pub struct ScorerPreviewWidgetState {
    /// Number of chips counted for the scoring hand.
    pub chips: usize,
    /// Level of the scored hand.
    pub level: usize,
    /// Multiplier for the scoring hand.
    pub multiplier: usize,
    /// Text content representing the scoring hand. If [`None`],
    /// [`ScorerPreviewWidget`] does not display the scoring hand text.
    pub scoring_hand_text: Option<String>,
}

/// [`Widget`] to show live scorer preview.
///
/// Widget construction uses builder pattern which can be started using the
/// [`Self::new()`] method.
///
/// ```
/// # use ratatui::{buffer::Buffer, layout::Rect, prelude::StatefulWidget};
/// # use balatro_tui_core::scorer::ScoringHand;
/// # use balatro_tui_widgets::{ScorerPreviewWidget, ScorerPreviewWidgetState};
/// let area = Rect::new(0, 0, 100, 100);
/// let mut buffer = Buffer::empty(area);
/// let mut cards = ScorerPreviewWidgetState {
///     chips: 10,
///     level: 2,
///     multiplier: 5,
///     scoring_hand_text: Some(ScoringHand::FourOfAKind.to_string()),
/// };
///
/// ScorerPreviewWidget::new().render(area, &mut buffer, &mut cards)
/// ```
///
/// ```
/// # use ratatui::{buffer::Buffer, layout::Rect, prelude::StatefulWidget};
/// # use balatro_tui_widgets::{ScorerPreviewWidget, ScorerPreviewWidgetState};
/// let area = Rect::new(0, 0, 100, 100);
/// let buffer = Buffer::empty(area);
/// let cards = ScorerPreviewWidgetState {
///     chips: 10,
///     level: 2,
///     multiplier: 5,
///     scoring_hand_text: None,
/// };
///
/// ScorerPreviewWidget::new().render(area, &mut buffer, &mut cards)
/// ```
#[derive(Clone, Copy, Debug, Default)]
pub struct ScorerPreviewWidget;

impl ScorerPreviewWidget {
    /// Create new instance of [`ScorerPreviewWidget`]
    #[must_use = "Created score preview widget instance must be used."]
    #[inline]
    pub const fn new() -> Self {
        Self {}
    }
}

// TODO: Add custom errors if required.
// TODO: Add const modifier to struct creation methods

impl StatefulWidget for ScorerPreviewWidget {
    type State = ScorerPreviewWidgetState;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        // Prepare areas
        let [inner_area] = Layout::vertical([Constraint::Length(SCORER_PREVIEW_CONTENT_HEIGHT)])
            .flex(Flex::Center)
            .areas(area);
        let [scoring_hand_text_area, scoring_area] =
            Layout::vertical([Constraint::Length(5), Constraint::Length(5)])
                .flex(Flex::Center)
                .areas(inner_area);
        let [chips_area, multiply_sign_area, multiplier_area] = Layout::horizontal([
            Constraint::Fill(1),
            Constraint::Length(3),
            Constraint::Fill(1),
        ])
        .areas(scoring_area);

        // Render widgets
        if let Some(hand) = state.scoring_hand_text.as_ref() {
            // TODO: Add leveling system for scoring hand types
            TextBoxWidget::new([Line::from(format!("{} [lvl. {}]", hand, state.level)).centered()])
                .constraints([Constraint::Length(1)])
                .render(scoring_hand_text_area, buf);
        }
        // TODO: Use big text to render chips, multiplier, scoring hand and multiply
        // icon.
        TextBoxWidget::bordered([Line::from(state.chips.to_string()).centered()])
            .render(chips_area, buf);
        TextBoxWidget::new([Line::from("\u{d7}".to_owned()).centered()])
            .render(multiply_sign_area, buf);
        TextBoxWidget::bordered([Line::from(state.multiplier.to_string()).centered()])
            .render(multiplier_area, buf);
    }
}
