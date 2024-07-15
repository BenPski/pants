use iced::{Border, Color, Theme};

#[derive(Debug, Clone, Copy)]
pub struct Appearance {
    /// The [`Border`] of the expand.
    pub border: Border,
}

impl Appearance {
    /// Derives a new [`Appearance`] with no border
    pub fn no_border(self) -> Self {
        Self {
            border: Border {
                width: 0.0,
                color: Color::TRANSPARENT,
                radius: 0.0.into(),
            },
        }
    }
}

/// A set of rules that dictate the [`Appearance`] of an expand.
pub trait StyleSheet {
    /// The supported style of the [`StyleSheet`].
    type Style: Default;

    /// Produces the [`Appearance`] of an expand.
    fn appearance(&self, style: &Self::Style) -> Appearance;
}
///
/// The style of an expand.
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

                Appearance {
                    border: Border {
                        color: palette.background.strong.color,
                        width: 2.0,
                        ..Default::default()
                    },
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
