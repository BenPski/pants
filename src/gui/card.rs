use iced::{
    advanced::{layout::Node, overlay, renderer, widget::Tree, Widget},
    Element, Length, Padding, Point, Size,
};
use iced_futures::core::event;

use super::card_style::StyleSheet;

pub struct Card<'a, Message, Theme = iced::Theme, Renderer = iced::Renderer>
where
    Theme: StyleSheet,
{
    width: Length,
    height: Length,
    max_width: f32,
    max_height: f32,
    padding: Padding,
    spacing: f32,
    style: Theme::Style,
    elements: Vec<Element<'a, Message, Theme, Renderer>>,
}

impl<'a, Message, Theme, Renderer> Card<'a, Message, Theme, Renderer>
where
    Theme: StyleSheet,
    Renderer: renderer::Renderer,
{
    pub fn new(
        head: impl Into<Element<'a, Message, Theme, Renderer>>,
        content: impl Into<Element<'a, Message, Theme, Renderer>>,
    ) -> Self {
        Card {
            width: Length::Fill,
            height: Length::Shrink,
            max_width: f32::INFINITY,
            max_height: f32::INFINITY,
            padding: Padding::new(10.0),
            spacing: 5.0,
            style: Default::default(),
            elements: vec![head.into(), content.into()],
        }
    }

    pub fn height(mut self, value: impl Into<Length>) -> Self {
        self.height = value.into();
        self
    }

    pub fn width(mut self, value: impl Into<Length>) -> Self {
        self.width = value.into();
        self
    }

    pub fn max_height(mut self, value: impl Into<f32>) -> Self {
        self.max_height = value.into();
        self
    }

    pub fn max_width(mut self, value: impl Into<f32>) -> Self {
        self.max_width = value.into();
        self
    }

    pub fn style(mut self, value: impl Into<Theme::Style>) -> Self {
        self.style = value.into();
        self
    }
}

impl<'a, Message, Theme, Renderer> From<Card<'a, Message, Theme, Renderer>>
    for Element<'a, Message, Theme, Renderer>
where
    Message: 'a,
    Theme: 'a + StyleSheet,
    Renderer: 'a + renderer::Renderer,
{
    fn from(value: Card<'a, Message, Theme, Renderer>) -> Self {
        Element::new(value)
    }
}

impl<'a, Message, Theme, Renderer> Widget<Message, Theme, Renderer>
    for Card<'a, Message, Theme, Renderer>
where
    Theme: StyleSheet,
    Renderer: renderer::Renderer,
{
    fn children(&self) -> Vec<Tree> {
        self.elements.iter().map(Tree::new).collect()
    }

    fn diff(&self, tree: &mut Tree) {
        tree.diff_children(&self.elements)
    }

    fn size(&self) -> Size<Length> {
        Size {
            width: self.width,
            height: self.height,
        }
    }

    fn layout(
        &self,
        tree: &mut Tree,
        renderer: &Renderer,
        limits: &iced::advanced::layout::Limits,
    ) -> iced::advanced::layout::Node {
        let limits = limits
            .width(self.width)
            .height(self.height)
            .max_width(self.max_width)
            .max_height(self.max_height)
            .shrink(self.padding);
        let mut nodes = vec![Node::default(), Node::default()];

        nodes[0] = self.elements[0].as_widget().layout(
            &mut tree.children[0],
            renderer,
            &limits.min_width(f32::INFINITY),
        );
        nodes[0].move_to_mut(Point::new(self.padding.left, self.padding.top));
        nodes[1] = self.elements[1]
            .as_widget()
            .layout(&mut tree.children[1], renderer, &limits);
        let header_size = nodes[0].size();
        let content_size = nodes[1].size();
        nodes[1].move_to_mut(Point::new(
            self.padding.left,
            header_size.height + self.padding.top + self.spacing,
        ));

        let intrinsic_size = Size::new(
            header_size.width.max(content_size.width),
            header_size.height + content_size.height + self.spacing,
        );
        Node::with_children(
            limits
                .resolve(self.width, self.height, intrinsic_size)
                .expand(self.padding),
            nodes,
        )
    }

    fn draw(
        &self,
        tree: &Tree,
        renderer: &mut Renderer,
        theme: &Theme,
        style: &renderer::Style,
        layout: iced::advanced::Layout<'_>,
        cursor: iced::advanced::mouse::Cursor,
        viewport: &iced::Rectangle,
    ) {
        if let Some(clipped_viewport) = layout.bounds().intersection(viewport) {
            let appearance = theme.appearance(&self.style);
            renderer.fill_quad(
                renderer::Quad {
                    bounds: layout.bounds(),
                    border: iced::Border {
                        color: appearance.border_color,
                        width: appearance.border_width,
                        radius: appearance.border_radius.into(),
                    },
                    shadow: appearance.shadow,
                },
                appearance.background,
            );
            let items = self
                .elements
                .iter()
                .zip(&tree.children)
                .zip(layout.children())
                .collect::<Vec<_>>();

            // draw header
            let ((header, state), layout) = items[0];
            // rounded portion of header
            let mut rounded_bounds = layout.bounds();
            rounded_bounds.height = 0.0;
            renderer.fill_quad(
                renderer::Quad {
                    bounds: rounded_bounds.expand(appearance.border_radius),
                    border: iced::Border {
                        color: iced::Color::TRANSPARENT,
                        width: 0.0,
                        radius: appearance.border_radius.into(),
                    },
                    shadow: iced::Shadow::default(),
                },
                appearance.header_background,
            );
            // square portion of header
            let mut regular_bounds = layout.bounds();
            regular_bounds.width += 2.0 * appearance.border_radius;
            regular_bounds.x -= appearance.border_radius;
            renderer.fill_quad(
                renderer::Quad {
                    bounds: regular_bounds,
                    border: iced::Border {
                        color: iced::Color::TRANSPARENT,
                        width: 0.0,
                        radius: 0.0.into(),
                    },
                    shadow: iced::Shadow::default(),
                },
                appearance.header_background,
            );
            header.as_widget().draw(
                state,
                renderer,
                theme,
                &renderer::Style {
                    text_color: appearance.header_text_color,
                },
                layout,
                cursor,
                &clipped_viewport,
            );
            // draw content
            let ((content, state), layout) = items[1];

            content.as_widget().draw(
                state,
                renderer,
                theme,
                style,
                layout,
                cursor,
                &clipped_viewport,
            );
        }
    }

    fn operate(
        &self,
        state: &mut Tree,
        layout: iced::advanced::Layout<'_>,
        renderer: &Renderer,
        operation: &mut dyn iced::advanced::widget::Operation<Message>,
    ) {
        operation.container(None, layout.bounds(), &mut |operation| {
            self.elements
                .iter()
                .zip(&mut state.children)
                .zip(layout.children())
                .for_each(|((child, state), layout)| {
                    child
                        .as_widget()
                        .operate(state, layout, renderer, operation);
                });
        });
    }

    fn on_event(
        &mut self,
        state: &mut Tree,
        event: iced::Event,
        layout: iced::advanced::Layout<'_>,
        cursor: iced::advanced::mouse::Cursor,
        renderer: &Renderer,
        clipboard: &mut dyn iced::advanced::Clipboard,
        shell: &mut iced::advanced::Shell<'_, Message>,
        viewport: &iced::Rectangle,
    ) -> iced_futures::core::event::Status {
        self.elements
            .iter_mut()
            .zip(&mut state.children)
            .zip(layout.children())
            .map(|((child, state), layout)| {
                child.as_widget_mut().on_event(
                    state,
                    event.clone(),
                    layout,
                    cursor,
                    renderer,
                    clipboard,
                    shell,
                    viewport,
                )
            })
            .fold(event::Status::Ignored, event::Status::merge)
    }

    fn mouse_interaction(
        &self,
        state: &Tree,
        layout: iced::advanced::Layout<'_>,
        cursor: iced::advanced::mouse::Cursor,
        viewport: &iced::Rectangle,
        renderer: &Renderer,
    ) -> iced::advanced::mouse::Interaction {
        self.elements
            .iter()
            .zip(&state.children)
            .zip(layout.children())
            .map(|((child, state), layout)| {
                child
                    .as_widget()
                    .mouse_interaction(state, layout, cursor, viewport, renderer)
            })
            .max()
            .unwrap_or_default()
    }

    fn overlay<'b>(
        &'b mut self,
        state: &'b mut Tree,
        layout: iced::advanced::Layout<'_>,
        renderer: &Renderer,
        translation: iced::Vector,
    ) -> Option<iced::advanced::overlay::Element<'b, Message, Theme, Renderer>> {
        overlay::from_children(&mut self.elements, state, layout, renderer, translation)
    }
}
