use iced::{
    advanced::{
        layout::{self, Limits},
        overlay, renderer,
        widget::Tree,
        Widget,
    },
    mouse, touch, Alignment, Element, Event, Length, Padding, Pixels, Point, Size,
};
use iced_futures::core::event;

pub struct Expand<'a, Message, Theme = iced::Theme, Renderer = iced::Renderer> {
    spacing: f32,
    padding: Padding,
    width: Length,
    height: Length,
    max_width: f32,
    align_items: Alignment,
    clip: bool,
    expand: bool,
    elements: Vec<Element<'a, Message, Theme, Renderer>>,
}

impl<'a, Message, Theme, Renderer> Expand<'a, Message, Theme, Renderer>
where
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
            padding: Padding::ZERO,
            width,
            height: Length::Shrink,
            max_width: f32::INFINITY,
            align_items: Alignment::Start,
            clip: false,
            expand,
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
    Message: 'a,
    Theme: 'a,
    Renderer: 'a + renderer::Renderer,
{
    fn from(value: Expand<'a, Message, Theme, Renderer>) -> Self {
        Element::new(value)
    }
}

impl<'a, Message, Theme, Renderer> Widget<Message, Theme, Renderer>
    for Expand<'a, Message, Theme, Renderer>
where
    Renderer: renderer::Renderer,
{
    fn children(&self) -> Vec<iced::advanced::widget::Tree> {
        println!("children");

        self.elements
            .iter()
            .take(self.num_elements())
            .map(Tree::new)
            .collect()
    }

    fn diff(&self, tree: &mut Tree) {
        println!("diff");
        tree.diff_children(
            &self
                .elements
                .iter()
                .take(self.num_elements())
                .collect::<Vec<_>>(),
        )
    }

    fn size(&self) -> iced::Size<Length> {
        println!("size");
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
        println!("Layout");
        layout::flex::resolve(
            layout::flex::Axis::Vertical,
            renderer,
            &limits,
            self.width,
            self.height,
            self.padding,
            self.spacing,
            self.align_items,
            &self.elements(),
            &mut tree.children,
        )
    }

    fn operate(
        &self,
        tree: &mut Tree,
        layout: layout::Layout<'_>,
        renderer: &Renderer,
        operation: &mut dyn iced::advanced::widget::Operation<Message>,
    ) {
        println!("operate");
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
        match event {
            Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left))
            | Event::Touch(touch::Event::FingerPressed { .. }) => {
                let bounds = layout.children().next().unwrap().bounds();
                if cursor.is_over(bounds) {
                    println!("clicked header");
                    self.expand = !self.expand;
                    shell.invalidate_layout();
                    return event::Status::Captured;
                } else {
                    return event::Status::Ignored;
                }
            }
            _ => return event::Status::Ignored,
        }
        // let n = self.num_elements();
        // self.elements
        //     .iter_mut()
        //     .take(n)
        //     .zip(&mut tree.children)
        //     .zip(layout.children())
        //     .map(|((child, state), layout)| {
        //         child.as_widget_mut().on_event(
        //             state,
        //             event.clone(),
        //             layout,
        //             cursor,
        //             renderer,
        //             clipboard,
        //             shell,
        //             viewport,
        //         )
        //     })
        //     .fold(event::Status::Ignored, event::Status::merge)
    }

    fn mouse_interaction(
        &self,
        tree: &Tree,
        layout: layout::Layout<'_>,
        cursor: iced::advanced::mouse::Cursor,
        viewport: &iced::Rectangle,
        renderer: &Renderer,
    ) -> iced::advanced::mouse::Interaction {
        self.elements
            .iter()
            .take(self.num_elements())
            .zip(&tree.children)
            .zip(layout.children())
            .map(|((child, state), layout)| {
                child
                    .as_widget()
                    .mouse_interaction(state, layout, cursor, viewport, renderer)
            })
            .max()
            .unwrap_or_default()
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
            for ((child, state), layout) in self
                .elements
                .iter()
                .take(self.num_elements())
                .zip(&tree.children)
                .zip(layout.children())
            {
                child.as_widget().draw(
                    state,
                    renderer,
                    theme,
                    style,
                    layout,
                    cursor,
                    if self.clip {
                        &clipped_viewport
                    } else {
                        viewport
                    },
                )
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
        let n = self.num_elements();
        overlay::from_children(
            &mut self.elements[..n],
            state,
            layout,
            renderer,
            translation,
        )
    }
}
