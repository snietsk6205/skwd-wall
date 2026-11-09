//! R1 (custom build): content with a floating panel next to it, drawn above
//! everything and taking no space. Unlike a tooltip it is opened by state,
//! so the keyboard can open it too (preview bubble, group menus).

use iced::advanced::layout::{self, Layout};
use iced::advanced::overlay;
use iced::advanced::renderer;
use iced::advanced::widget::{self, Widget};
use iced::advanced::{Clipboard, Shell};
use iced::{Element, Event, Length, Point, Rectangle, Size, Vector, mouse};

/// Where the floating panel goes, relative to the content.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AnchorSide {
    /// Below the content, starting at its left edge.
    Below,
    /// Above the content, aligned to its end (below it when there is no
    /// room above).
    AboveEnd,
}

pub struct Anchored<'a, Message, Theme, Renderer> {
    content: Element<'a, Message, Theme, Renderer>,
    panel: Option<Element<'a, Message, Theme, Renderer>>,
    side: AnchorSide,
    gap: f32,
    min_top: f32,
}

impl<Message, Theme, Renderer> Anchored<'_, Message, Theme, Renderer> {
    /// Keeps the panel below this y (for example under a top bar).
    #[must_use]
    pub fn min_top(mut self, min_top: f32) -> Self {
        self.min_top = min_top;
        self
    }
}

pub fn anchored<'a, Message, Theme, Renderer>(
    content: impl Into<Element<'a, Message, Theme, Renderer>>,
    panel: Option<Element<'a, Message, Theme, Renderer>>,
    side: AnchorSide,
    gap: f32,
) -> Anchored<'a, Message, Theme, Renderer> {
    Anchored { content: content.into(), panel, side, gap, min_top: 0.0 }
}

impl<Message, Theme, Renderer> Widget<Message, Theme, Renderer>
    for Anchored<'_, Message, Theme, Renderer>
where
    Renderer: iced::advanced::Renderer,
{
    fn children(&self) -> Vec<widget::Tree> {
        let mut children = vec![widget::Tree::new(&self.content)];
        if let Some(panel) = &self.panel {
            children.push(widget::Tree::new(panel));
        }
        children
    }

    fn diff(&self, tree: &mut widget::Tree) {
        match &self.panel {
            Some(panel) => tree.diff_children(&[self.content.as_widget(), panel.as_widget()]),
            None => tree.diff_children(&[self.content.as_widget()]),
        }
    }

    fn size(&self) -> Size<Length> {
        self.content.as_widget().size()
    }

    fn size_hint(&self) -> Size<Length> {
        self.content.as_widget().size_hint()
    }

    fn layout(
        &mut self,
        tree: &mut widget::Tree,
        renderer: &Renderer,
        limits: &layout::Limits,
    ) -> layout::Node {
        self.content.as_widget_mut().layout(&mut tree.children[0], renderer, limits)
    }

    fn update(
        &mut self,
        tree: &mut widget::Tree,
        event: &Event,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        renderer: &Renderer,
        clipboard: &mut dyn Clipboard,
        shell: &mut Shell<'_, Message>,
        viewport: &Rectangle,
    ) {
        self.content.as_widget_mut().update(
            &mut tree.children[0],
            event,
            layout,
            cursor,
            renderer,
            clipboard,
            shell,
            viewport,
        );
    }

    fn mouse_interaction(
        &self,
        tree: &widget::Tree,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        viewport: &Rectangle,
        renderer: &Renderer,
    ) -> mouse::Interaction {
        self.content.as_widget().mouse_interaction(
            &tree.children[0],
            layout,
            cursor,
            viewport,
            renderer,
        )
    }

    fn draw(
        &self,
        tree: &widget::Tree,
        renderer: &mut Renderer,
        theme: &Theme,
        style: &renderer::Style,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        viewport: &Rectangle,
    ) {
        self.content.as_widget().draw(
            &tree.children[0],
            renderer,
            theme,
            style,
            layout,
            cursor,
            viewport,
        );
    }

    fn operate(
        &mut self,
        tree: &mut widget::Tree,
        layout: Layout<'_>,
        renderer: &Renderer,
        operation: &mut dyn widget::Operation,
    ) {
        self.content.as_widget_mut().operate(&mut tree.children[0], layout, renderer, operation);
    }

    fn overlay<'b>(
        &'b mut self,
        tree: &'b mut widget::Tree,
        layout: Layout<'b>,
        renderer: &Renderer,
        viewport: &Rectangle,
        translation: Vector,
    ) -> Option<overlay::Element<'b, Message, Theme, Renderer>> {
        let mut children = tree.children.iter_mut();
        let content_tree = children.next()?;
        let content = self.content.as_widget_mut().overlay(
            content_tree,
            layout,
            renderer,
            viewport,
            translation,
        );
        let panel = match (self.panel.as_mut(), children.next()) {
            (Some(panel), Some(tree)) => Some(overlay::Element::new(Box::new(Panel {
                content: panel,
                tree,
                anchor: layout.bounds() + translation,
                side: self.side,
                gap: self.gap,
                min_top: self.min_top,
            }))),
            _ => None,
        };
        if content.is_none() && panel.is_none() {
            return None;
        }
        Some(overlay::Group::with_children(content.into_iter().chain(panel).collect()).overlay())
    }
}

struct Panel<'a, 'b, Message, Theme, Renderer> {
    content: &'b mut Element<'a, Message, Theme, Renderer>,
    tree: &'b mut widget::Tree,
    anchor: Rectangle,
    side: AnchorSide,
    gap: f32,
    min_top: f32,
}

/// Top-left corner of a `size` panel next to `anchor` inside `viewport`.
fn anchored_origin(
    anchor: Rectangle,
    size: Size,
    viewport: Size,
    side: AnchorSide,
    gap: f32,
    min_top: f32,
) -> Point {
    let (x, y) = match side {
        AnchorSide::Below => (anchor.x, anchor.y + anchor.height + gap),
        AnchorSide::AboveEnd => {
            let above = anchor.y - size.height - gap;
            let y = if above >= min_top { above } else { anchor.y + anchor.height + gap };
            (anchor.x + anchor.width - size.width, y)
        }
    };
    Point::new(
        x.clamp(0.0, (viewport.width - size.width).max(0.0)),
        y.clamp(
            min_top.min(viewport.height - size.height).max(0.0),
            (viewport.height - size.height).max(0.0),
        ),
    )
}

impl<Message, Theme, Renderer> overlay::Overlay<Message, Theme, Renderer>
    for Panel<'_, '_, Message, Theme, Renderer>
where
    Renderer: iced::advanced::Renderer,
{
    fn layout(&mut self, renderer: &Renderer, bounds: Size) -> layout::Node {
        let node = self.content.as_widget_mut().layout(
            self.tree,
            renderer,
            &layout::Limits::new(Size::ZERO, bounds),
        );
        let origin =
            anchored_origin(self.anchor, node.size(), bounds, self.side, self.gap, self.min_top);
        node.move_to(origin)
    }

    fn draw(
        &self,
        renderer: &mut Renderer,
        theme: &Theme,
        style: &renderer::Style,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
    ) {
        self.content.as_widget().draw(
            self.tree,
            renderer,
            theme,
            style,
            layout,
            cursor,
            &layout.bounds(),
        );
    }

    fn update(
        &mut self,
        event: &Event,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        renderer: &Renderer,
        clipboard: &mut dyn Clipboard,
        shell: &mut Shell<'_, Message>,
    ) {
        self.content.as_widget_mut().update(
            self.tree,
            event,
            layout,
            cursor,
            renderer,
            clipboard,
            shell,
            &layout.bounds(),
        );
    }

    fn mouse_interaction(
        &self,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        renderer: &Renderer,
    ) -> mouse::Interaction {
        self.content.as_widget().mouse_interaction(
            self.tree,
            layout,
            cursor,
            &layout.bounds(),
            renderer,
        )
    }

    fn operate(
        &mut self,
        layout: Layout<'_>,
        renderer: &Renderer,
        operation: &mut dyn widget::Operation,
    ) {
        self.content.as_widget_mut().operate(self.tree, layout, renderer, operation);
    }
}

impl<'a, Message, Theme, Renderer> From<Anchored<'a, Message, Theme, Renderer>>
    for Element<'a, Message, Theme, Renderer>
where
    Message: 'a,
    Theme: 'a,
    Renderer: iced::advanced::Renderer + 'a,
{
    fn from(anchored: Anchored<'a, Message, Theme, Renderer>) -> Self {
        Element::new(anchored)
    }
}
