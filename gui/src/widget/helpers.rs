use iced::{
    widget::{center, mouse_area, opaque, Stack},
    Element,
};
use iced_aw::{
    card::Status,
    style::card::{self},
    widgets::Card,
};

/// A widget used internally for collecting data, a convenience wrapper around [`Card`] to get the
/// right style

pub fn form<'a, Message>(
    header: impl Into<Element<'a, Message>>,
    content: impl Into<Element<'a, Message>>,
) -> iced_aw::Card<'a, Message> {
    Card::new(header, content).style(|theme: &iced::Theme, status: Status| {
        let palette = theme.extended_palette();
        let primary = iced_aw::style::card::primary(theme, status);
        card::Style {
            border_color: palette.primary.strong.color,
            head_background: palette.primary.strong.color.into(),
            head_text_color: palette.primary.strong.text,
            ..primary
        }
    })
}

pub fn modal<'a, Message>(
    primary: impl Into<Element<'a, Message>>,
    overlay: impl Into<Element<'a, Message>>,
    on_blur: Message,
) -> iced::widget::Stack<'a, Message>
where
    Message: Clone + 'a,
{
    Stack::from_vec(vec![
        primary.into(),
        mouse_area(center(opaque(overlay))).on_press(on_blur).into(),
    ])
}
