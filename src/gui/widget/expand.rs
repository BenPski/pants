use iced::{
    advanced::{
        layout::{self, Node},
        mouse, overlay, renderer,
        widget::Tree,
        Widget,
    },
    touch, Alignment, Color, Element, Event, Length, Padding, Pixels, Point, Shadow, Size,
};
use iced_futures::core::event;

use crate::gui::{style::expand::StyleSheet, with_padding};

pub struct Expand<'a, Message, Theme = iced::Theme, Renderer = iced::Renderer>
where
    Theme: StyleSheet,
{
    spacing: f32,
    padding: Padding,
    width: Length,
    height: Length,
    max_width: f32,
    align_items: Alignment,
    clip: bool,
    expand: bool,
    style: Theme::Style,
    on_press: Option<Message>,
    elements: Vec<Element<'a, Message, Theme, Renderer>>,
}

impl<'a, Message, Theme, Renderer> Expand<'a, Message, Theme, Renderer>
where
    Message: Clone,
    Theme: StyleSheet,
    Renderer: renderer::Renderer,
{
    pub fn new(
        header: impl Into<Element<'a, Message, Theme, Renderer>>,
        content: impl Into<Element<'a, Message, Theme, Renderer>>,
        expand: bool,
    ) -> Self {
        let header = header.into();
        let content = content.into();
        let header_size = header.as_widget().size_hint();
        let content_size = content.as_widget().size_hint();
        let width = Length::Shrink
            .enclose(header_size.width)
            .enclose(content_size.width);
        Self {
            spacing: 0.0,
            padding: Padding::new(10.0),
            width,
            height: Length::Shrink,
            max_width: f32::INFINITY,
            align_items: Alignment::Start,
            clip: false,
            expand,
            style: Default::default(),
            on_press: None,
            elements: vec![header, content],
        }
    }

    pub fn spacing(mut self, value: impl Into<Pixels>) -> Self {
        self.spacing = value.into().0;
        self
    }

    pub fn padding(mut self, value: impl Into<Padding>) -> Self {
        self.padding = value.into();
        self
    }

    pub fn width(mut self, value: impl Into<Length>) -> Self {
        self.width = value.into();
        self
    }

    pub fn height(mut self, value: impl Into<Length>) -> Self {
        self.height = value.into();
        self
    }

    pub fn max_width(mut self, value: impl Into<Pixels>) -> Self {
        self.max_width = value.into().0;
        self
    }

    pub fn align_items(mut self, value: impl Into<Alignment>) -> Self {
        self.align_items = value.into();
        self
    }

    pub fn clip(mut self, value: impl Into<bool>) -> Self {
        self.clip = value.into();
        self
    }

    pub fn expand(mut self, value: impl Into<bool>) -> Self {
        self.expand = value.into();
        self
    }

    pub fn style(mut self, value: impl Into<Theme::Style>) -> Self {
        self.style = value.into();
        self
    }

    pub fn on_press(mut self, value: impl Into<Message>) -> Self {
        self.on_press = Some(value.into());
        self
    }

    fn num_elements(&self) -> usize {
        if self.expand {
            2
        } else {
            1
        }
    }

    fn elements(&'a self) -> &'a [Element<'a, Message, Theme, Renderer>] {
        &self.elements[..self.num_elements()]
    }

    pub fn toggle(&mut self) {
        self.expand = !self.expand;
    }
}

impl<'a, Message, Theme, Renderer> From<Expand<'a, Message, Theme, Renderer>>
    for Element<'a, Message, Theme, Renderer>
where
    Message: 'a + Clone,
    Theme: 'a + StyleSheet,
    Renderer: 'a + renderer::Renderer,
{
    fn from(value: Expand<'a, Message, Theme, Renderer>) -> Self {
        Element::new(value)
    }
}

impl<'a, Message, Theme, Renderer> Widget<Message, Theme, Renderer>
    for Expand<'a, Message, Theme, Renderer>
where
    Message: Clone,
    Theme: StyleSheet,
    Renderer: renderer::Renderer,
{
    fn children(&self) -> Vec<iced::advanced::widget::Tree> {
        self.elements.iter().map(Tree::new).collect()
    }

    fn diff(&self, tree: &mut Tree) {
        tree.diff_children(&self.elements.iter().collect::<Vec<_>>())
    }

    fn size(&self) -> iced::Size<Length> {
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
            .shrink(self.padding);
        let mut nodes = if self.expand {
            vec![Node::default(), Node::default()]
        } else {
            vec![Node::default()]
        };

        nodes[0] = self.elements[0].as_widget().layout(
            &mut tree.children[0],
            renderer,
            &limits.min_width(f32::INFINITY),
        );
        nodes[0].move_to_mut(Point::new(self.padding.left, self.padding.top));
        let header_size = nodes[0].size();
        let mut content_size = Node::default().size();
        if self.expand {
            nodes[1] =
                self.elements[1]
                    .as_widget()
                    .layout(&mut tree.children[1], renderer, &limits);
            content_size = nodes[1].size();
            nodes[1].move_to_mut(Point::new(
                self.padding.left,
                header_size.height + self.padding.top + self.spacing,
            ));
        }

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

    fn operate(
        &self,
        tree: &mut Tree,
        layout: layout::Layout<'_>,
        renderer: &Renderer,
        operation: &mut dyn iced::advanced::widget::Operation<Message>,
    ) {
        operation.container(None, layout.bounds(), &mut |operation| {
            self.elements()
                .iter()
                .zip(&mut tree.children)
                .zip(layout.children())
                .for_each(|((child, state), layout)| {
                    child
                        .as_widget()
                        .operate(state, layout, renderer, operation)
                })
        });
    }

    fn on_event(
        &mut self,
        tree: &mut Tree,
        event: iced::Event,
        layout: layout::Layout<'_>,
        cursor: iced::advanced::mouse::Cursor,
        renderer: &Renderer,
        clipboard: &mut dyn iced::advanced::Clipboard,
        shell: &mut iced::advanced::Shell<'_, Message>,
        viewport: &iced::Rectangle,
    ) -> iced_futures::core::event::Status {
        let child_event = self
            .elements
            .iter_mut()
            .zip(&mut tree.children)
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
            .fold(event::Status::Ignored, event::Status::merge);
        let header_event = match event {
            Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left))
            | Event::Touch(touch::Event::FingerPressed { .. }) => {
                let bounds = with_padding(layout.children().next().unwrap().bounds(), self.padding);
                if cursor.is_over(bounds) {
                    self.expand = !self.expand;
                    shell.invalidate_layout();
                    if let Some(on_press) = &self.on_press {
                        shell.publish(on_press.clone());
                    }
                    event::Status::Captured
                } else {
                    event::Status::Ignored
                }
            }
            _ => event::Status::Ignored,
        };
        event::Status::merge(child_event, header_event)
    }

    fn mouse_interaction(
        &self,
        tree: &Tree,
        layout: layout::Layout<'_>,
        cursor: iced::advanced::mouse::Cursor,
        viewport: &iced::Rectangle,
        renderer: &Renderer,
    ) -> iced::advanced::mouse::Interaction {
        let child_mouse = self
            .elements
            .iter()
            .zip(&tree.children)
            .zip(layout.children())
            .map(|((child, state), layout)| {
                child
                    .as_widget()
                    .mouse_interaction(state, layout, cursor, viewport, renderer)
            })
            .max()
            .unwrap_or_default();

        let header_mouse = if cursor.is_over(with_padding(
            layout.children().next().unwrap().bounds(),
            self.padding,
        )) {
            mouse::Interaction::Pointer
        } else {
            mouse::Interaction::default()
        };

        mouse::Interaction::max(child_mouse, header_mouse)
    }

    fn draw(
        &self,
        tree: &Tree,
        renderer: &mut Renderer,
        theme: &Theme,
        style: &renderer::Style,
        layout: layout::Layout<'_>,
        cursor: iced::advanced::mouse::Cursor,
        viewport: &iced::Rectangle,
    ) {
        if let Some(clipped_viewport) = layout.bounds().intersection(viewport) {
            let viewport = if self.clip {
                &clipped_viewport
            } else {
                viewport
            };
            let appearance = theme.appearance(&self.style);
            renderer.fill_quad(
                renderer::Quad {
                    bounds: layout.bounds(),
                    border: appearance.border,
                    shadow: Shadow::default(),
                },
                Color::TRANSPARENT,
            );
            let items = self
                .elements
                .iter()
                .zip(&tree.children)
                .zip(layout.children())
                .collect::<Vec<_>>();

            let ((header, state), layout) = items[0];

            header
                .as_widget()
                .draw(state, renderer, theme, style, layout, cursor, viewport);

            if self.expand {
                let ((content, state), layout) = items[1];

                content
                    .as_widget()
                    .draw(state, renderer, theme, style, layout, cursor, viewport);
            }
        }
    }
    fn overlay<'b>(
        &'b mut self,
        state: &'b mut Tree,
        layout: layout::Layout<'_>,
        renderer: &Renderer,
        translation: iced::Vector,
    ) -> Option<iced::advanced::overlay::Element<'b, Message, Theme, Renderer>> {
        overlay::from_children(&mut self.elements, state, layout, renderer, translation)
    }
}
