use iced::widget::Container;
use iced::{Element, Length, alignment};

pub struct ButtonContent<M> {
    content: Element<'static, M>,

    width: Option<Length>,
    height: Option<Length>,

    align_x: alignment::Horizontal,
    align_y: alignment::Vertical,
}

impl<M: Clone + 'static> ButtonContent<M> {
    pub fn new(content: impl Into<Element<'static, M>>) -> Self {
        Self {
            content: content.into(),
            width: Some(Length::Fill),
            height: None,
            align_x: alignment::Horizontal::Center,
            align_y: alignment::Vertical::Center,
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
        self.align_x = align;

        self
    }

    pub fn align_y(mut self, align: alignment::Vertical) -> Self {
        self.align_y = align;

        self
    }

    pub fn build(self) -> Element<'static, M> {
        let mut container = Container::new(self.content)
            .align_x(self.align_x)
            .align_y(self.align_y);

        if let Some(w) = self.width {
            container = container.width(w);
        }

        if let Some(h) = self.height {
            container = container.height(h);
        }

        container.into()
    }
}
