use iced::{Background, Color, Shadow, Theme};

#[derive(Debug, Clone, Copy)]
pub struct Appearance {
    /// The text [`Color`] of the content.
    pub text_color: Color,
    /// The [`Background`] of the content.
    pub background: Background,
    /// The [`Shadow`] of the content.
    pub shadow: Shadow,
    /// The text [`Color`] of the header.
    pub header_text_color: Color,
    /// The [`Background`] of the header.
    pub header_background: Background,
    /// The radius of the corners of the card.
    pub border_radius: f32,
    /// The width of the border of the card.
    pub border_width: f32,
    /// The border color of the card.
    pub border_color: Color,
}

impl Appearance {
    /// Derives a new [`Appearance`] with the given [`Background`].
    pub fn with_background(self, background: impl Into<Background>) -> Self {
        Self {
            background: background.into(),
            ..self
        }
    }
}

/// A set of rules that dictate the [`Appearance`] of a card.
pub trait StyleSheet {
    /// The supported style of the [`StyleSheet`].
    type Style: Default;

    /// Produces the [`Appearance`] of a card.
    fn appearance(&self, style: &Self::Style) -> Appearance;
}
///
/// The style of a card.
#[derive(Default)]
pub enum Card {
    /// A simple box.
    #[default]
    Box,
    /// A custom style.
    Custom(Box<dyn StyleSheet<Style = Theme>>),
}

impl From<Appearance> for Card {
    fn from(appearance: Appearance) -> Self {
        Self::Custom(Box::new(move |_: &_| appearance))
    }
}

impl<T: Fn(&Theme) -> Appearance + 'static> From<T> for Card {
    fn from(f: T) -> Self {
        Self::Custom(Box::new(f))
    }
}

impl StyleSheet for Theme {
    type Style = Card;

    fn appearance(&self, style: &Self::Style) -> Appearance {
        match style {
            Card::Box => {
                let palette = self.extended_palette();
                let foreground = self.palette();

                Appearance {
                    text_color: foreground.text,
                    background: palette.background.base.color.into(),
                    shadow: Shadow::default(),
                    header_text_color: palette.primary.strong.text,
                    header_background: palette.primary.strong.color.into(),
                    border_radius: 10.0,
                    border_width: 2.0,
                    border_color: palette.background.strong.color,
                }
            }
            Card::Custom(custom) => custom.appearance(self),
        }
    }
}

impl<T: Fn(&Theme) -> Appearance> StyleSheet for T {
    type Style = Theme;

    fn appearance(&self, style: &Self::Style) -> Appearance {
        (self)(style)
    }
}
