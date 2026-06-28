use iced::Padding;
use iced::widget::{Container, container};
use iced::{Element, Length};
use iced::{Theme, alignment};

use crate::drop_file::DEFAULT_PADDING_DROP;
use crate::drop_file::default_drop_style;

type DropStyleFn = dyn Fn(&Theme) -> container::Style;

pub struct DropZone<M> {
    content: Element<'static, M>,
    on_drop: Option<Box<dyn Fn(std::path::PathBuf) -> M + 'static>>,
    on_hover: Option<M>,
    width: Length,
    height: Option<Length>,
    align_x: alignment::Horizontal,
    align_y: alignment::Vertical,
    padding: Option<Padding>,
    style_fn: Option<Box<DropStyleFn>>,
}

impl<M: Clone + 'static> DropZone<M> {
    pub fn new(content: impl Into<Element<'static, M>>) -> Self {
        Self {
            content: content.into(),
            on_drop: None,
            on_hover: None,
            width: Length::Fill,
            height: Some(Length::Fill),
            align_x: alignment::Horizontal::Center,
            align_y: alignment::Vertical::Center,
            padding: Some(DEFAULT_PADDING_DROP),
            style_fn: Some(Box::new(default_drop_style)),
        }
    }

    pub fn content(mut self, content: impl Into<Element<'static, M>>) -> Self {
        self.content = content.into();
        self
    }

    pub fn on_drop<F>(mut self, f: F) -> Self
    where
        F: Fn(std::path::PathBuf) -> M + 'static,
    {
        self.on_drop = Some(Box::new(f));
        self
    }

    pub fn on_hover(mut self, msg: M) -> Self {
        self.on_hover = Some(msg);
        self
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

    pub fn style<F>(mut self, f: F) -> Self
    where
        F: Fn(&Theme) -> container::Style + 'static,
    {
        self.style_fn = Some(Box::new(f));
        self
    }

    pub fn build(self) -> Element<'static, M> {
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
            container = container.style(style_fn);
        }

        container.into()
    }
}
