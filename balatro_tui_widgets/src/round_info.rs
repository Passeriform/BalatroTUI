use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Flex, Layout, Margin, Rect},
    style::{Color, Stylize},
    text::Line,
    widgets::Widget,
};

use super::{blind_badge::BlindBadgeWidget, text_box::TextBoxWidget, utility::get_line_with_chips};

/// Content height for [`RoundInfoWidget`].
pub const ROUND_INFO_CONTENT_HEIGHT: u16 = 9;
/// Kerning multiplier for horizontal and vertical print-space equality.
const KERNING_MULTIPLIER: u16 = 2;

/// [`Widget`] to show information about the running round.
///
/// Following details are shown:
/// - Blind chip
/// - Target score
/// - Reward for defeating blind
///
/// Widget construction uses builder pattern which can be started using the
/// [`Self::new()`] method.
///
/// ```
/// # use ratatui::{buffer::Buffer, layout::Rect, prelude::Widget, style::Color};
/// # use balatro_tui_widgets::RoundInfoWidget;
/// let area = Rect::new(0, 0, 100, 100);
/// let mut buffer = Buffer::empty(area);
///
/// RoundInfoWidget::new()
///     .blind_color(Color::Red)
///     .blind_text("Small Blind".to_string())
///     .reward(5)
///     .target_score(500)
///     .render(area, &mut buffer);
/// ```
#[derive(Clone, Debug, Default)]
pub struct RoundInfoWidget {
    blind_text: String,
    blind_color: Color,
    target_score: usize,
    reward: usize,
}

impl RoundInfoWidget {
    /// Create new instance of [`RoundInfoWidget`]
    #[must_use = "Created round info widget instance must be used."]
    #[inline]
    pub fn new() -> Self {
        Self::default()
    }

    /// Update the text to be used for blind and return the [`RoundInfoWidget`]
    /// instance.
    #[must_use = "Round info widget builder returned instance must be used."]
    #[inline]
    pub fn blind_text(mut self, text: String) -> Self {
        self.blind_text = text;
        self
    }

    /// Update the color to be used for blind and return the [`RoundInfoWidget`]
    /// instance.
    #[must_use = "Round info widget builder returned instance must be used."]
    #[inline]
    pub const fn blind_color(mut self, color: Color) -> Self {
        self.blind_color = color;
        self
    }

    /// Update the target score and return the [`RoundInfoWidget`] instance.
    #[must_use = "Round info widget builder returned instance must be used."]
    #[inline]
    pub const fn target_score(mut self, target_score: usize) -> Self {
        self.target_score = target_score;
        self
    }

    /// Update the reward text and return the [`RoundInfoWidget`] instance.
    #[must_use = "Round info widget builder returned instance must be used."]
    #[inline]
    pub const fn reward(mut self, reward: usize) -> Self {
        self.reward = reward;
        self
    }
}

// TODO: Add pub(crate) qualifications

impl Widget for RoundInfoWidget {
    fn render(self, area: Rect, buf: &mut Buffer) {
        // Prepare variables
        let round_info_content = [
            Line::from("Score at least").centered(),
            // TODO: Consider using BigText here
            get_line_with_chips(self.target_score.to_string(), Color::Red).centered(),
            // TODO: Use reward field from round here.
            Line::from(vec![
                "Reward: ".into(),
                "$".repeat(self.reward).yellow().bold(),
            ])
            .centered(),
        ];

        // Prepare areas
        let [inner_area] = Layout::vertical([Constraint::Length(ROUND_INFO_CONTENT_HEIGHT)])
            .flex(Flex::Center)
            .areas(area);
        let [blind_badge_area, round_info_area] = Layout::horizontal([
            Constraint::Length(ROUND_INFO_CONTENT_HEIGHT * KERNING_MULTIPLIER),
            Constraint::Fill(1),
        ])
        .areas(inner_area);

        // Render widgets
        BlindBadgeWidget::new()
            .color(self.blind_color)
            .content(self.blind_text)
            .render(blind_badge_area.inner(Margin::new(2, 1)), buf);
        TextBoxWidget::bordered(round_info_content)
            .flex(Flex::SpaceAround)
            .render(round_info_area, buf);
    }
}
