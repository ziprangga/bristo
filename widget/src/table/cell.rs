use iced::widget::{Container, container};
use iced::{Element, Length, Padding, Theme, alignment};

use std::sync::atomic::{AtomicUsize, Ordering};
static NEXT_ID: AtomicUsize = AtomicUsize::new(0);

fn next_id() -> CellId {
    NEXT_ID.fetch_add(1, Ordering::Relaxed)
}

pub type CellId = usize;
type CellStyleFn = dyn Fn(CellId, bool, &Theme) -> container::Style;

pub struct Cell<M> {
    content: Element<'static, M>,
    id: CellId,
    wrap: bool,
    selected: Option<CellId>,
    width: Length,
    height: Option<Length>,
    align_x: alignment::Horizontal,
    align_y: alignment::Vertical,
    padding: Option<Padding>,
    style_fn: Option<Box<CellStyleFn>>,
}

impl<M: Clone + 'static> Cell<M> {
    pub fn new(content: impl Into<Element<'static, M>>) -> Self {
        Self {
            content: content.into(),
            id: next_id(),
            wrap: false,
            selected: None,
            width: Length::Fill,
            height: Some(Length::Fill),
            align_x: alignment::Horizontal::Center,
            align_y: alignment::Vertical::Center,
            padding: None,
            style_fn: None,
        }
    }

    pub fn set_content(mut self, content: impl Into<Element<'static, M>>) -> Self {
        self.content = content.into();
        self
    }

    pub fn set_id(mut self, id: CellId) -> Self {
        self.id = id;
        self
    }

    pub fn get_id(&self) -> CellId {
        self.id
    }

    pub fn wrap(mut self, wrap: bool) -> Self {
        self.wrap = wrap;
        self
    }

    pub fn selected(mut self, id: Option<CellId>) -> Self {
        self.selected = id;
        self
    }

    pub fn is_selected(&self, id: usize) -> bool {
        self.selected == Some(id)
    }

    pub fn width(mut self, width: Length) -> Self {
        self.width = width;
        self
    }

    pub fn height(mut self, height: Length) -> Self {
        self.height = Some(height);
        self
    }

    pub fn align_x(mut self, x: impl Into<alignment::Horizontal>) -> Self {
        self.align_x = x.into();
        self
    }

    pub fn align_y(mut self, y: impl Into<alignment::Vertical>) -> Self {
        self.align_y = y.into();
        self
    }

    pub fn padding(mut self, p: impl Into<Padding>) -> Self {
        self.padding = Some(p.into());
        self
    }

    pub fn style_fn<F>(mut self, f: F) -> Self
    where
        F: Fn(CellId, bool, &Theme) -> container::Style + 'static,
    {
        self.style_fn = Some(Box::new(f));
        self
    }

    pub fn build(self) -> Element<'static, M> {
        if !self.wrap {
            return self.content;
        }

        let mut container = Container::new(self.content)
            .align_x(self.align_x)
            .align_y(self.align_y)
            .width(self.width);

        if let Some(h) = self.height {
            container = container.height(h);
        }

        if let Some(p) = self.padding {
            container = container.padding(p)
        }

        if let Some(style_fn) = self.style_fn {
            let cell_id = self.id;
            let selected = self.selected == Some(cell_id);

            container = container.style(move |theme| style_fn(cell_id, selected, theme));
        }

        container.into()
    }
}
