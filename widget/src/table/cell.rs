use iced::widget::Id;
use iced::widget::{Container, container};
use iced::{Element, Length, Padding, Theme, alignment};

pub type CellId = Id;
type CellStyleFn = dyn Fn(usize, Option<&CellId>, &Theme) -> container::Style;

pub struct Cell<M> {
    content: Element<'static, M>,
    id: Option<CellId>,
    width: Option<Length>,
    height: Option<Length>,
    align_x: Option<alignment::Horizontal>,
    align_y: Option<alignment::Vertical>,
    padding: Option<Padding>,
    style_fn: Option<Box<CellStyleFn>>,
}

impl<M: Clone + 'static> Cell<M> {
    pub fn new(content: impl Into<Element<'static, M>>) -> Self {
        Self {
            content: content.into(),
            id: None,
            width: None,
            height: None,
            align_x: None,
            align_y: None,
            padding: None,
            style_fn: None,
        }
    }

    pub fn set_content(mut self, content: impl Into<Element<'static, M>>) -> Self {
        self.content = content.into();
        self
    }

    pub fn id(mut self, id: impl Into<CellId>) -> Self {
        self.id = Some(id.into());
        self
    }

    pub fn get_id(&self) -> Option<&CellId> {
        self.id.as_ref()
    }

    pub fn width(mut self, width: Length) -> Self {
        self.width = Some(width);
        self
    }

    pub fn height(mut self, height: Length) -> Self {
        self.height = Some(height);
        self
    }

    pub fn align_x(mut self, x: impl Into<alignment::Horizontal>) -> Self {
        self.align_x = Some(x.into());
        self
    }

    pub fn align_y(mut self, y: impl Into<alignment::Vertical>) -> Self {
        self.align_y = Some(y.into());
        self
    }

    pub fn padding(mut self, p: impl Into<Padding>) -> Self {
        self.padding = Some(p.into());
        self
    }

    pub fn style_fn<F>(mut self, f: F) -> Self
    where
        F: Fn(usize, Option<&CellId>, &Theme) -> container::Style + 'static,
    {
        self.style_fn = Some(Box::new(f));
        self
    }

    pub fn build(self, index: usize) -> Element<'static, M> {
        let cell_id = self.id.clone();
        let padding = self.padding;
        let style_fn = self.style_fn;

        let mut container = Container::new(self.content);

        if let Some(ref cell_id) = self.id {
            container = container.id(cell_id.clone());
        }

        if let Some(w) = self.width {
            container = container.width(w);
        }

        if let Some(h) = self.height {
            container = container.height(h);
        }

        if let Some(align_x) = self.align_x {
            container = container.align_x(align_x);
        }

        if let Some(align_y) = self.align_y {
            container = container.align_y(align_y);
        }

        if let Some(p) = padding {
            container = container.padding(p)
        }

        if let Some(style_fn) = style_fn {
            container = container.style(move |theme| style_fn(index, cell_id.as_ref(), theme));
        }

        container.into()
    }
}
