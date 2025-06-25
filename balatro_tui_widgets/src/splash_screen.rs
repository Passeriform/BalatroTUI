use std::cmp::max;

use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Flex, Layout, Rect},
    style::Stylize,
    symbols::border,
    text::Line,
    widgets::{Block, Clear, StatefulWidget, Widget},
};
use tui_big_text::{BigText, PixelSize};

use crate::TextBoxWidget;

const FULL_PIXEL_WIDTH: usize = 8;
const QUADRANT_PIXEL_WIDTH: usize = 4;

/// [`Widget`] to display end splash screen.
///
/// Widget construction uses builder pattern which can be started using the
/// [`Self::new()`] method.
///
/// ```
/// # use ratatui::{buffer::Buffer, layout::Rect, prelude::StatefulWidget};
/// # use balatro_tui_widgets::SplashScreenWidget;
/// let area = Rect::new(0, 0, 100, 100);
/// let mut buffer = Buffer::empty(area);
///
/// SplashScreenWidget::new()
///     .splash("Some title")
///     .message("This is some message.")
///     .render(area, &mut buffer, &mut vec![
///         ("stat-1", "4"),
///         ("stat-2", "7"),
///     ]);
/// ```
#[derive(Copy, Clone, Debug, Default)]
pub struct SplashScreenWidget<'widget> {
    /// Text to be displayed as title of the splash screen.
    splash: &'widget str,
    /// Supporting message text to be displayed on the splash screen.
    message: &'widget str,
}

impl<'widget> SplashScreenWidget<'widget> {
    /// Create new instance of [`SplashScreenWidget`].
    #[must_use = "Created splash screen widget state instance must be used."]
    #[inline]
    pub const fn new() -> Self {
        Self {
            splash: "",
            message: "",
        }
    }

    /// Update the splash text and return the [`SplashScreenWidget`] instance.
    #[must_use = "Splash screen widget builder returned instance must be used."]
    #[inline]
    pub const fn splash(mut self, splash: &'widget str) -> Self {
        self.splash = splash;
        self
    }

    /// Update the message text and return the [`SplashScreenWidget`] instance.
    #[must_use = "Splash screen widget builder returned instance must be used."]
    #[inline]
    pub const fn message(mut self, message: &'widget str) -> Self {
        self.message = message;
        self
    }
}

impl<'widget> StatefulWidget for SplashScreenWidget<'widget> {
    type State = Vec<(&'widget str, &'widget str)>;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        // Prepare variables
        let splash_line = self.splash.bold().into_centered_line();
        let message_line = self.message.italic().into_centered_line();
        let stat_lines = state
            .iter()
            .map(|&(key, value)| vec![key.bold(), "\t\t".into(), value.yellow()].into())
            .collect::<Vec<Line<'_>>>();
        let render_big = (area.width as usize)
            > max(
                splash_line
                    .width()
                    .saturating_add(1)
                    .saturating_mul(FULL_PIXEL_WIDTH),
                message_line
                    .width()
                    .saturating_add(1)
                    .saturating_mul(QUADRANT_PIXEL_WIDTH),
            );

        // Prepare areas
        let [splash_area, message_area, mut details_area] = Layout::vertical([
            Constraint::Length(if render_big { 8 } else { 4 }),
            Constraint::Length(if render_big { 4 } else { 1 }),
            Constraint::Length(
                stat_lines
                    .len()
                    .saturating_mul(2)
                    .saturating_add(3)
                    .try_into()
                    .unwrap_or(u16::MAX),
            ),
        ])
        .flex(Flex::SpaceAround)
        .areas(area);
        details_area = Layout::horizontal([Constraint::Length(40)])
            .flex(Flex::SpaceAround)
            .areas::<1>(details_area)[0];

        // Render widgets
        Clear.render(area, buf);
        Block::bordered()
            .border_set(border::DOUBLE)
            .render(area, buf);
        BigText::builder()
            .lines([splash_line])
            .pixel_size(
                if render_big {
                    PixelSize::Full
                } else {
                    PixelSize::Quadrant
                },
            )
            .centered()
            .build()
            .render(splash_area, buf);

        if render_big {
            BigText::builder()
                .lines([message_line])
                .pixel_size(PixelSize::Quadrant)
                .centered()
                .build()
                .render(message_area, buf);
        } else {
            TextBoxWidget::new([message_line]).render(message_area, buf);
        }

        // TODO: Convert to table
        TextBoxWidget::bordered(stat_lines)
            .padding(4)
            .render(details_area, buf);
    }
}
