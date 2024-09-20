use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Stylize},
    symbols::Marker,
    text::Line,
    widgets::{
        canvas::{Canvas, Circle},
        Widget,
    },
};

/// [`Widget`] for depicting [`balatro_tui_core::blind::Blind`] with text
/// inside.
///
/// Widget construction uses builder pattern which can be started using the
/// [`Self::new()`] method.
///
/// ```
/// # use ratatui::{buffer::Buffer, layout::Rect, prelude::Widget, style::Color, text::Line};
/// # use balatro_tui_widgets::BlindBadgeWidget;
/// let area = Rect::new(0, 0, 100, 100);
/// let mut buffer = Buffer::empty(area);
///
/// BlindBadgeWidget::new()
///     .color(Color::Green)
///     .content(Line::from("Small Blind"))
///     .render(area, &mut buffer);
/// ```
#[derive(Clone, Debug, Default)]
pub struct BlindBadgeWidget {
    content: String,
    color: Color,
}

impl BlindBadgeWidget {
    /// Create new instance of [`BlindBadgeWidget`].
    #[must_use = "Created blind badge widget instance must be used."]
    #[inline]
    pub const fn new() -> Self {
        Self {
            color: Color::White,
            content: String::new(),
        }
    }

    /// Update the color to be used for chip icon and return the
    /// [`BlindBadgeWidget`] instance.
    #[must_use = "Blind badge widget builder returned instance must be used."]
    #[inline]
    pub const fn color(mut self, color: Color) -> Self {
        self.color = color;
        self
    }

    /// Update the content to be displayed next to chip icon and return the
    /// [`BlindBadgeWidget`] instance.
    #[must_use = "Blind badge widget builder returned instance must be used."]
    #[inline]
    pub fn content<C>(mut self, content: C) -> Self
    where
        String: From<C>,
    {
        self.content = content.into();
        self
    }
}

impl Widget for BlindBadgeWidget {
    // TODO: Use image instead of canvas
    fn render(self, area: Rect, buf: &mut Buffer) {
        // Prepare variables
        let bound = f64::from(area.height);

        // Render widgets
        let canvas = Canvas::default()
            .marker(Marker::Braille)
            .paint(|ctx| {
                ctx.draw(&Circle {
                    x: 0.0,
                    y: 0.0,
                    radius: bound,
                    color: self.color,
                });

                self.content
                    .split_whitespace()
                    .map(String::from)
                    .map(|text_chunk| Line::from(text_chunk).centered().yellow())
                    .rev()
                    .enumerate()
                    .for_each(|(idx, line)| {
                        #[expect(
                            clippy::as_conversions,
                            reason = "Intended: Expanding to a larger type for API conformity."
                        )]
                        ctx.print(-1.0, idx as f64, line);
                    });
            })
            .x_bounds([-bound, bound])
            .y_bounds([-bound, bound]);
        canvas.render(area, buf);
    }
}
