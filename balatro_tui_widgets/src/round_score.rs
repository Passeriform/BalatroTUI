use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Flex, Layout, Rect},
    style::Color,
    text::Line,
    widgets::{StatefulWidget, Widget},
};

use super::{text_box::TextBoxWidget, utility::get_line_with_chips};

/// Content height for [`RoundScoreWidget`]
pub const ROUND_SCORE_CONTENT_HEIGHT: u16 = 5;

/// [`Widget`] to show current score in the running round.
///
/// Widget construction uses builder pattern which can be started using the
/// [`Self::new()`] method.
///
/// ```
/// # use ratatui::{buffer::Buffer, layout::Rect, prelude::StatefulWidget};
/// # use balatro_tui_widgets::RoundScoreWidget;
/// let area = Rect::new(0, 0, 100, 100);
/// let mut buffer = Buffer::empty(area);
/// let mut score = 2000;
///
/// RoundScoreWidget::new().render(area, &mut buffer, &mut score);
/// ```
#[derive(Clone, Copy, Debug, Default)]
pub struct RoundScoreWidget;

impl RoundScoreWidget {
    /// Create new instance of [`RoundScoreWidget`]
    #[must_use = "Created round score widget instance must be used."]
    #[inline]
    pub const fn new() -> Self {
        Self {}
    }
}

impl StatefulWidget for RoundScoreWidget {
    type State = usize;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        // Prepare widgets
        let round_score_content = [Line::from("Round Score").centered()];

        // Prepare areas
        let [inner_area] = Layout::vertical([Constraint::Length(ROUND_SCORE_CONTENT_HEIGHT)])
            .flex(Flex::Center)
            .areas(area);
        let [mut round_score_text_area, round_score_value_area] =
            Layout::horizontal([Constraint::Fill(2), Constraint::Fill(5)]).areas(inner_area);
        round_score_text_area = Layout::vertical([Constraint::Length(1)])
            .flex(Flex::SpaceAround)
            .areas::<1>(round_score_text_area)[0];

        // Render widgets
        TextBoxWidget::new(round_score_content).render(round_score_text_area, buf);
        TextBoxWidget::bordered([get_line_with_chips(state.to_string(), Color::Red).centered()])
            .render(round_score_value_area, buf);
    }
}
