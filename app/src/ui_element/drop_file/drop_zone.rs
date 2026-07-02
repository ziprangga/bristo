use iced::Padding;
use iced::widget::{Container, container};
use iced::{Element, Length};
use iced::{Theme, alignment};

type DropStyleFn = dyn Fn(&Theme) -> container::Style;

pub struct DropZone<M> {
    content: Element<'static, M>,
    width: Option<Length>,
    height: Option<Length>,
    align_x: Option<alignment::Horizontal>,
    align_y: Option<alignment::Vertical>,
    padding: Option<Padding>,
    style_fn: Option<Box<DropStyleFn>>,
}

impl<M: Clone + 'static> DropZone<M> {
    pub fn new(content: impl Into<Element<'static, M>>) -> Self {
        Self {
            content: content.into(),
            width: None,
            height: None,
            align_x: None,
            align_y: None,
            padding: None,
            style_fn: None,
        }
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

    pub fn style<F>(mut self, f: F) -> Self
    where
        F: Fn(&Theme) -> container::Style + 'static,
    {
        self.style_fn = Some(Box::new(f));
        self
    }

    pub fn build(self) -> Element<'static, M> {
        let mut container = Container::new(self.content);

        if let Some(w) = self.width {
            container = container.width(w);
        }

        if let Some(h) = self.height {
            container = container.height(h);
        }

        if let Some(x) = self.align_x {
            container = container.align_x(x);
        }
        if let Some(y) = self.align_y {
            container = container.align_y(y);
        }

        if let Some(p) = self.padding {
            container = container.padding(p)
        }

        if let Some(style_fn) = self.style_fn {
            container = container.style(style_fn);
        }

        container.into()
    }
}
