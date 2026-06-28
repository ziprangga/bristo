use iced::Padding;
use iced::widget::Button;
use iced::widget::button::{Status, Style};
use iced::{Element, Length, Theme};

use crate::button_style::DEFAULT_PADDING_BUTTON;
use crate::button_style::default_style;

pub struct BtnImage<M> {
    image: iced::widget::Image,
    img_width: Length,
    img_height: Length,
    on_press: Option<M>,
    width: Length,
    height: Option<Length>,
    padding: Padding,
    content_width: Option<Length>,
    content_height: Option<Length>,
    style_fn: Option<Box<dyn Fn(&Theme, Status) -> Style + 'static>>,
}

impl<M: Clone + 'static> BtnImage<M> {
    pub fn new(image: iced::widget::Image) -> Self {
        Self {
            image,
            img_width: Length::Fill,
            img_height: Length::Fill,
            on_press: None,
            width: Length::Fill,
            height: None,
            padding: DEFAULT_PADDING_BUTTON,
            content_width: None,
            content_height: None,
            style_fn: Some(Box::new(default_style)),
        }
    }

    pub fn img_width(mut self, width: Length) -> Self {
        self.img_width = width;

        self
    }

    pub fn img_height(mut self, height: Length) -> Self {
        self.img_height = height;

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

    pub fn content_width(mut self, content_width: Length) -> Self {
        self.content_width = Some(content_width);
        self
    }

    pub fn content_height(mut self, content_height: Length) -> Self {
        self.content_height = Some(content_height);
        self
    }

    pub fn style<F>(mut self, style_fn: F) -> Self
    where
        F: Fn(&Theme, Status) -> Style + 'static,
    {
        self.style_fn = Some(Box::new(style_fn));
        self
    }

    pub fn build(self) -> Element<'static, M> {
        let content_btn: Element<'static, M> = {
            let img_width = self.content_width.unwrap_or(self.img_width);
            let img_height = self.content_height.unwrap_or(self.img_height);
            self.image.width(img_width).height(img_height).into()
        };

        let mut btn = Button::new(content_btn)
            .width(self.width)
            .padding(self.padding);

        if let Some(h) = self.height {
            btn = btn.height(h);
        }

        if let Some(msg) = &self.on_press {
            btn = btn.on_press(msg.clone());
        }

        if let Some(style_fn) = self.style_fn {
            btn = btn.style(style_fn);
        }

        btn.into()
    }
}
