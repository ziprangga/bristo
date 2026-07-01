use iced::widget::Container;
use iced::{Element, Length, alignment};

pub struct ButtonContent<M> {
    content: Element<'static, M>,
    width: Option<Length>,
    height: Option<Length>,
    align_x: Option<alignment::Horizontal>,
    align_y: Option<alignment::Vertical>,
}

impl<M: Clone + 'static> ButtonContent<M> {
    pub fn new(content: impl Into<Element<'static, M>>) -> Self {
        Self {
            content: content.into(),
            width: None,
            height: None,
            align_x: None,
            align_y: None,
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

    pub fn align_x(mut self, align: alignment::Horizontal) -> Self {
        self.align_x = Some(align);

        self
    }

    pub fn align_y(mut self, align: alignment::Vertical) -> Self {
        self.align_y = Some(align);

        self
    }

    pub fn build(self) -> Element<'static, M> {
        let mut container = Container::new(self.content);
        if let Some(align_x) = self.align_x {
            container = container.align_x(align_x);
        }

        if let Some(align_y) = self.align_y {
            container = container.align_y(align_y);
        }

        if let Some(w) = self.width {
            container = container.width(w);
        }

        if let Some(h) = self.height {
            container = container.height(h);
        }

        container.into()
    }
}
