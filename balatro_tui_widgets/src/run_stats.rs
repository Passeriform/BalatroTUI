use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Flex, Layout, Margin, Rect},
    text::Line,
    widgets::{StatefulWidget, Widget},
};

use super::text_box::TextBoxWidget;

/// Content height for [`RunStatsWidget`].
const RUN_STATS_CONTENT_HEIGHT: u16 = 15;

/// Render state for [`RunStatsWidget`].
#[derive(Clone, Copy, Debug, Default)]
pub struct RunStatsWidgetState {
    /// Number of remaining hands
    pub hands: usize,
    /// Number of remaining discards
    pub discards: usize,
    /// Money available in the run
    pub money: usize,
    /// Current run ante
    pub ante: usize,
    /// Current round number
    pub round: usize,
}

/// [`Widget`] to show stats for a run.
///
/// Widget construction uses builder pattern which can be started using the
/// [`Self::new()`] method.
///
/// ```
/// # use ratatui::{buffer::Buffer, layout::Rect, prelude::StatefulWidget};
/// # use balatro_tui_widgets::{RunStatsWidgetState, RunStatsWidget};
/// let area = Rect::new(0, 0, 100, 100);
/// let mut buffer = Buffer::empty(area);
/// let mut state = RunStatsWidgetState {
///     hands: 3,
///     discards: 3,
///     money: 20,
///     ante: 2,
///     round: 5,
/// };
///
/// RunStatsWidget::new().render(area, &mut buffer, &mut state);
/// ```
#[derive(Clone, Copy, Debug, Default)]
pub struct RunStatsWidget;

impl RunStatsWidget {
    /// Create new instance of [`RunStatsWidget`]
    #[must_use = "Created run stats widget instance must be used."]
    #[inline]
    pub const fn new() -> Self {
        Self {}
    }
}

// TODO: Rename to game_stats.

impl StatefulWidget for RunStatsWidget {
    type State = RunStatsWidgetState;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        // Prepare areas
        let [inner_area] = Layout::vertical([Constraint::Length(RUN_STATS_CONTENT_HEIGHT)])
            .flex(Flex::Center)
            .areas(area.inner(Margin::new(1, 1)));
        let [_, run_stats_area] =
            Layout::horizontal([Constraint::Fill(1), Constraint::Fill(2)]).areas(inner_area);
        let [round_functional_info_area, money_area, round_meta_info_area] =
            Layout::vertical([Constraint::Fill(1); 3])
                .flex(Flex::Center)
                .areas(run_stats_area);
        let [hands_count_area, discards_count_area] =
            Layout::horizontal([Constraint::Fill(1); 2]).areas(round_functional_info_area);
        let [ante_area, round_number_area] =
            Layout::horizontal([Constraint::Fill(1); 2]).areas(round_meta_info_area);

        // Render widgets
        // TODO: Load RunInfoButtonWidget here.
        TextBoxWidget::bordered([Line::from(state.hands.to_string()).centered()])
            .title("Hands")
            .render(hands_count_area, buf);
        TextBoxWidget::bordered([Line::from(state.discards.to_string()).centered()])
            .title("Discards")
            .render(discards_count_area, buf);
        TextBoxWidget::bordered([Line::from(format!("{}$", state.money)).centered()])
            .title("Money")
            .render(money_area, buf);
        TextBoxWidget::bordered([Line::from(state.ante.to_string()).centered()])
            .title("Ante")
            .render(ante_area, buf);
        TextBoxWidget::bordered([Line::from(state.round.to_string()).centered()])
            .title("Round")
            .render(round_number_area, buf);
    }
}
