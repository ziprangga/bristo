mod content;
mod style;

pub use content::ButtonContent;
pub use style::{ButtonStyle, custom_btn_rounded_style};

use iced::Element;
use iced::Padding;

use iced::widget::button::Status;
use iced::{Length, Theme};

const DEFAULT_PADDING_BUTTON: Padding = Padding {
    top: 5.0,
    bottom: 5.0,
    right: 10.0,
    left: 10.0,
};

pub struct CustomButton<M> {
    content: Option<ButtonContent<M>>,

    on_press: Option<M>,
    width: Length,
    height: Option<Length>,
    padding: Padding,
    style_fn: ButtonStyle,
}

impl<M: Clone + 'static> CustomButton<M> {
    pub fn new() -> Self {
        Self {
            content: None,
            on_press: None,
            width: Length::Fill,
            height: None,
            padding: DEFAULT_PADDING_BUTTON,
            style_fn: ButtonStyle::Default,
        }
    }

    pub fn content_element(mut self, content: impl Into<Element<'static, M>>) -> Self {
        let base = self
            .content
            .unwrap_or_else(|| ButtonContent::new(content.into()));
        self.content = Some(base);
        self
    }

    pub fn content_layout<F>(mut self, f: F) -> Self
    where
        F: FnOnce(ButtonContent<M>) -> ButtonContent<M>,
    {
        if let Some(c) = self.content {
            self.content = Some(f(c));
        }
        self
    }

    pub fn on_press(mut self, msg: M) -> Self {
        self.on_press = Some(msg);
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

    pub fn padding(mut self, padding: impl Into<Padding>) -> Self {
        self.padding = padding.into();
        self
    }

    pub fn style(mut self, style: ButtonStyle) -> Self {
        self.style_fn = style;
        self
    }

    pub fn build(self) -> Element<'static, M> {
        let content = match self.content {
            Some(c) => c.build(),
            None => return iced::widget::Space::new().into(),
        };

        let mut button = iced::widget::button(content)
            .width(self.width)
            .padding(self.padding);

        if let Some(height) = self.height {
            button = button.height(height);
        }

        let styled =
            button.style(move |theme: &Theme, status: Status| self.style_fn.style(theme, status));

        match self.on_press {
            Some(msg) => styled.on_press(msg).into(),
            None => styled.into(),
        }
    }
}
