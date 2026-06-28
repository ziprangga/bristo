use iced::widget::button::{Status, Style};
use iced::widget::text::Wrapping;
use iced::widget::{Button, Text};
use iced::{Color, Padding};
use iced::{Element, Length, Theme, alignment};

use crate::button_style::DEFAULT_PADDING_BUTTON;
use crate::button_style::default_style;

pub struct BtnText<M> {
    label: String,
    text_size: u32,
    text_color: Option<Color>,
    text_wrapping: Option<Wrapping>,
    text_align_x: alignment::Horizontal,
    text_align_y: alignment::Vertical,
    text_width: Length,
    text_height: Length,
    on_press: Option<M>,
    width: Length,
    height: Option<Length>,
    padding: Padding,
    content_width: Option<Length>,
    content_height: Option<Length>,
    style_fn: Option<Box<dyn Fn(&Theme, Status) -> Style + 'static>>,
}

impl<M: Clone + 'static> BtnText<M> {
    pub fn new(label: impl Into<String>) -> Self {
        Self {
            label: label.into(),
            text_size: 12,
            text_color: None,
            text_wrapping: None,
            text_align_x: alignment::Horizontal::Center,
            text_align_y: alignment::Vertical::Center,
            text_width: Length::Shrink,
            text_height: Length::Shrink,

            on_press: None,
            width: Length::Fill,
            height: None,
            padding: DEFAULT_PADDING_BUTTON,
            content_width: None,
            content_height: None,
            style_fn: Some(Box::new(default_style)),
        }
    }

    pub fn text_size(mut self, size: u32) -> Self {
        self.text_size = size;

        self
    }
    pub fn text_color(mut self, color: Color) -> Self {
        self.text_color = Some(color);

        self
    }

    pub fn text_align_x(mut self, align: alignment::Horizontal) -> Self {
        self.text_align_x = align;

        self
    }

    pub fn text_align_y(mut self, align: alignment::Vertical) -> Self {
        self.text_align_y = align;

        self
    }

    pub fn text_wrapping(mut self, wrapping: Wrapping) -> Self {
        self.text_wrapping = Some(wrapping);

        self
    }

    pub fn text_width(mut self, width: Length) -> Self {
        self.text_width = width;

        self
    }

    pub fn text_height(mut self, height: Length) -> Self {
        self.text_height = height;

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
            let content_width = self.content_width.unwrap_or(self.text_width);
            let content_height = self.content_height.unwrap_or(self.text_height);
            let mut txt = Text::new(self.label)
                .size(self.text_size)
                .align_x(self.text_align_x)
                .align_y(self.text_align_y)
                .width(content_width)
                .height(content_height);

            if let Some(w) = self.text_wrapping {
                txt = txt.wrapping(w)
            }

            if self.style_fn.is_none()
                && let Some(color) = self.text_color
            {
                txt = txt.color(color);
            }

            txt.into()
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
