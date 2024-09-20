use ratatui::{
    buffer::Buffer,
    layout::{Alignment, Constraint, Flex, Layout, Margin, Rect},
    text::Line,
    widgets::{block::Title, Block, BorderType, Widget},
};

/// [`Widget`] to render vertically and horizontally centered text.
///
/// Widget construction uses builder pattern which can be started using the
/// [`Self::new()`] method.
///
/// ```
/// # use ratatui::{buffer::Buffer, layout::Rect, prelude::*, text::Line};
/// # use balatro_tui_widgets::TextBoxWidget;
/// let area = Rect::new(0, 0, 100, 100);
/// let mut buffer = Buffer::empty(area);
/// let lines: Vec<Line> = vec!["Some text".into(), "Some other text".into()];
///
/// TextBoxWidget::new(lines).render(area, &mut buffer);
/// ```
///
/// Additionally border and title can be specified to set style for text box.
/// Constraints and flex can also be specified to modify layout alignment for
/// content.
///
/// ```
/// # use ratatui::{buffer::Buffer, layout::{Constraint, Flex, Rect}, prelude::*, text::Line, widgets::{Block, BorderType}};
/// # use balatro_tui_widgets::TextBoxWidget;
/// let area = Rect::new(0, 0, 100, 100);
/// let mut buffer = Buffer::empty(area);
/// let lines: Vec<Line> = vec!["Some text".into(), "Some other text".into()];
///
/// TextBoxWidget::new(lines)
///     .border_block(Block::bordered().border_type(BorderType::Rounded))
///     .title("Title")
///     .constraints([Constraint::Length(1), Constraint::Length(1)])
///     .flex(Flex::SpaceAround)
///     .render(area, &mut buffer);
/// ```
///
/// [`TextBoxWidget`] also provides [`Self::bordered()`] utility method as a
/// shorthand to create bordered text boxes.
///
/// ```
/// # use ratatui::{buffer::Buffer, layout::{Flex, Rect}, prelude::*, text::Line};
/// # use balatro_tui_widgets::TextBoxWidget;
/// let area = Rect::new(0, 0, 100, 100);
/// let mut buffer = Buffer::empty(area);
/// let lines: Vec<Line> = vec!["Some text".into(), "Some other text".into()];
///
/// TextBoxWidget::bordered(lines)
///     .title("Title")
///     .constraints([Constraint::Length(1), Constraint::Length(1)])
///     .flex(Flex::SpaceAround)
///     .render(area, &mut buffer);
/// ```
#[derive(Clone, Debug, Default)]
pub struct TextBoxWidget<'widget> {
    /// Optional [`Block`] widget that surrounds the content.
    border_block: Option<Block<'widget>>,
    /// Overridable constraints for aligning content.
    constraints: Option<Vec<Constraint>>,
    /// Text content to be displayed.
    content: Vec<Line<'widget>>,
    /// Overridable [`Flex`] layout.
    flex: Flex,
    /// Optional title to be displayed on the border. If
    /// [`TextBoxWidget::border_block`] property is not set, this property will
    /// be ignored.
    title: Option<Title<'widget>>,
}

impl<'widget> TextBoxWidget<'widget> {
    /// Create new instance of [`TextBoxWidget`].
    #[must_use = "Created text box widget instance must be used."]
    #[inline]
    pub fn new<C>(content: C) -> Self
    where
        C: IntoIterator,
        C::Item: Into<Line<'widget>> + Widget,
        Vec<Line<'widget>>: From<C>,
    {
        TextBoxWidget {
            border_block: None,
            constraints: None,
            content: content.into(),
            flex: Flex::SpaceAround,
            title: None,
        }
    }

    /// Create a bordered instance of [`TextBoxWidget`]. By default the
    /// borders are rounded. The style can be overridden using
    /// [`Self::border_block()`] method.
    #[must_use = "Text box widget builder returned instance must be used."]
    #[inline]
    pub fn bordered<C>(content: C) -> Self
    where
        C: IntoIterator,
        C::Item: Into<Line<'widget>> + Widget,
        Vec<Line<'widget>>: From<C>,
    {
        let mut text_box = Self::new(content);
        text_box.border_block = Some(Block::bordered().border_type(BorderType::Rounded));
        text_box
    }

    /// Update the [`Self::border_block`] with a new [`Block`] and return the
    /// [`TextBoxWidget`] instance.
    #[must_use = "Text box widget builder returned instance must be used."]
    #[inline]
    pub fn border_block(mut self, border_block: Block<'widget>) -> Self {
        self.border_block = Some(border_block);
        self
    }

    /// Update the layout constraints to be used to align content and return the
    /// [`TextBoxWidget`] instance.
    #[must_use = "Text box widget builder returned instance must be used."]
    #[inline]
    pub fn constraints<I>(mut self, constraints: I) -> Self
    where
        I: IntoIterator,
        I::Item: Into<Constraint>,
        Vec<Constraint>: From<I>,
    {
        self.constraints = Some(constraints.into());
        self
    }

    /// Update the content of the text box and return the [`TextBoxWidget`]
    /// instance.
    #[must_use = "Text box widget builder returned instance must be used."]
    #[inline]
    pub fn content<C>(mut self, content: C) -> Self
    where
        C: IntoIterator,
        C::Item: Into<Line<'widget>> + Widget,
        Vec<Line<'widget>>: From<C>,
    {
        self.content = content.into();
        self
    }

    /// Update the layout flex justify content and return the [`TextBoxWidget`]
    /// instance.
    #[must_use = "Text box widget builder returned instance must be used."]
    #[inline]
    pub const fn flex(mut self, flex: Flex) -> Self {
        self.flex = flex;
        self
    }

    /// Update the title of the block for the text box and return the
    /// [`TextBoxWidget`] instance. If a [`Self::border_block`] is not set, this
    /// property is ignored when rendering.
    #[must_use = "Text box widget builder returned instance must be used."]
    #[inline]
    pub fn title<T>(mut self, title: T) -> Self
    where
        T: Into<Title<'widget>>,
    {
        self.title = Some(title.into());
        self
    }
}

// TODO: Re-implement using ratatui-image.
// TODO: Remove needless clone calls.

impl Widget for TextBoxWidget<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let inner_area = if let Some(mut border_block) = self.border_block {
            if let Some(title) = self.title {
                border_block = border_block.title(title.alignment(Alignment::Center));
            }

            border_block.render(area, buf);
            area.inner(Margin::new(1, 1))
        } else {
            area
        };

        let text_areas = Layout::vertical(
            self.constraints
                .unwrap_or_else(|| vec![Constraint::Length(1); self.content.iter().len()]),
        )
        .flex(self.flex)
        .split(inner_area);

        self.content
            .iter()
            .zip(text_areas.iter())
            .for_each(|(line, &text_area)| {
                line.render(text_area, buf);
            });
    }
}
