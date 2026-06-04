use iced::widget::button::{Status, Style};
use iced::widget::text::Wrapping;
use iced::widget::{Button, Text};
use iced::{Background, Border, Color, Padding, Shadow};
use iced::{Element, Length, Theme, alignment};

const DEFAULT_PADDING: Padding = Padding {
    top: 5.0,
    bottom: 5.0,
    right: 10.0,
    left: 10.0,
};

enum ButtonContent {
    Text(BtnText),
    Image(BtnImage),
}
struct BtnText {
    label: String,
    text_align_x: alignment::Horizontal,
    text_align_y: alignment::Vertical,
    text_size: u32,
    text_color: Option<Color>,
    text_wrapping: Option<Wrapping>,
    text_width: Length,
    text_height: Length,
}

struct BtnImage {
    image: iced::widget::Image,
    img_width: Length,
    img_height: Length,
}

type StyleFn = dyn Fn(&Theme, Status) -> Style;
pub struct CustomButton<M> {
    content: ButtonContent,
    on_press: Option<M>,
    width: Length,
    height: Option<Length>,
    padding: Padding,
    content_width: Option<Length>,
    content_height: Option<Length>,
    style_fn: Option<Box<StyleFn>>,
}

impl<M: 'static + Clone> CustomButton<M> {
    pub fn new(label: impl Into<String>) -> Self {
        Self {
            content: ButtonContent::Text(BtnText {
                label: label.into(),
                text_size: 12,
                text_color: None,
                text_align_x: alignment::Horizontal::Center,
                text_align_y: alignment::Vertical::Center,
                text_wrapping: None,
                text_width: Length::Shrink,
                text_height: Length::Shrink,
            }),
            on_press: None,
            width: Length::Fill,
            height: None,
            padding: DEFAULT_PADDING,
            content_width: None,
            content_height: None,
            style_fn: Some(Box::new(default_style)),
        }
    }

    pub fn image(image: iced::widget::Image) -> Self {
        Self {
            content: ButtonContent::Image(BtnImage {
                image,
                img_width: Length::Fill,
                img_height: Length::Fill,
            }),
            on_press: None,
            width: Length::Fill,
            height: None,
            padding: DEFAULT_PADDING,
            content_width: None,
            content_height: None,
            style_fn: Some(Box::new(default_style)),
        }
    }

    // =================================
    pub fn text_size(mut self, size: u32) -> Self {
        if let ButtonContent::Text(ref mut t) = self.content {
            t.text_size = size;
        }
        self
    }
    pub fn text_color(mut self, color: Color) -> Self {
        if let ButtonContent::Text(ref mut t) = self.content {
            t.text_color = Some(color);
        }
        self
    }

    pub fn text_align_x(mut self, align: alignment::Horizontal) -> Self {
        if let ButtonContent::Text(ref mut t) = self.content {
            t.text_align_x = align;
        }
        self
    }

    pub fn text_align_y(mut self, align: alignment::Vertical) -> Self {
        if let ButtonContent::Text(ref mut t) = self.content {
            t.text_align_y = align;
        }
        self
    }

    pub fn text_wrapping(mut self, wrapping: Wrapping) -> Self {
        if let ButtonContent::Text(ref mut t) = self.content {
            t.text_wrapping = Some(wrapping);
        }
        self
    }

    pub fn text_width(mut self, width: Length) -> Self {
        if let ButtonContent::Text(ref mut t) = self.content {
            t.text_width = width;
        }
        self
    }

    pub fn text_height(mut self, height: Length) -> Self {
        if let ButtonContent::Text(ref mut t) = self.content {
            t.text_height = height;
        }
        self
    }

    // =================================

    pub fn img_width(mut self, width: Length) -> Self {
        if let ButtonContent::Image(ref mut img) = self.content {
            img.img_width = width;
        }
        self
    }

    pub fn img_height(mut self, height: Length) -> Self {
        if let ButtonContent::Image(ref mut img) = self.content {
            img.img_height = height;
        }
        self
    }

    // =================================
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

    pub fn view(self) -> Element<'static, M> {
        let content_btn: Element<'static, M> = match self.content {
            ButtonContent::Text(btn_text) => {
                let content_width = self.content_width.unwrap_or(btn_text.text_width);
                let content_height = self.content_height.unwrap_or(btn_text.text_height);
                let mut txt = Text::new(btn_text.label)
                    .size(btn_text.text_size)
                    .align_x(btn_text.text_align_x)
                    .align_y(btn_text.text_align_y)
                    .width(content_width)
                    .height(content_height);

                if let Some(w) = btn_text.text_wrapping {
                    txt = txt.wrapping(w)
                }

                if self.style_fn.is_none()
                    && let Some(color) = btn_text.text_color
                {
                    txt = txt.color(color);
                }

                txt.into()
            }
            ButtonContent::Image(btn_img) => {
                let width = self.content_width.unwrap_or(btn_img.img_width);
                let height = self.content_height.unwrap_or(btn_img.img_height);
                btn_img.image.width(width).height(height).into()
            }
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

pub fn default_style(_theme: &iced::Theme, status: Status) -> Style {
    match status {
        Status::Pressed => Style {
            background: Some(Background::Color(Color::from_rgb8(50, 50, 250))),
            text_color: Color::from_rgb(3.0 / 255.0, 161.0 / 255.0, 252.0 / 255.0),
            border: Border::default(),
            shadow: Shadow::default(),
            snap: false,
        },
        Status::Hovered => Style {
            background: Some(Background::Color(Color::from_rgb8(10, 135, 230))),
            text_color: Color::from_rgb(50.0 / 255.0, 50.0 / 255.0, 50.0 / 255.0),
            border: Border::default(),
            shadow: Shadow::default(),
            snap: false,
        },
        Status::Active => Style {
            background: Some(Background::Color(Color::from_rgb8(30, 80, 230))),
            text_color: Color::from_rgb(1.0, 1.0, 1.0),
            border: Border::default(),
            shadow: Shadow::default(),
            snap: false,
        },
        Status::Disabled => Style {
            background: Some(Background::Color(Color::from_rgb8(10, 30, 80))),
            text_color: Color::from_rgb8(150, 150, 150),
            border: Border::default(),
            shadow: Shadow::default(),
            snap: false,
        },
    }
}

pub fn custom_btn_style(_theme: &iced::Theme, status: Status) -> Style {
    match status {
        Status::Pressed => Style {
            background: Some(Background::Color(Color::from_rgb8(70, 70, 70))),
            text_color: Color::from_rgb8(50, 50, 50),
            border: Border::default(),
            shadow: Shadow::default(),
            snap: false,
        },
        Status::Hovered => Style {
            background: Some(Background::Color(Color::from_rgb8(80, 80, 80))),
            text_color: Color::from_rgb8(255, 255, 255),
            border: Border::default(),
            shadow: Shadow::default(),
            snap: false,
        },
        Status::Active => Style {
            background: Some(Background::Color(Color::from_rgb8(50, 50, 50))),
            text_color: Color::from_rgb8(3, 161, 252),
            border: Border::default(),
            shadow: Shadow::default(),
            snap: false,
        },
        Status::Disabled => Style {
            background: Some(Background::Color(Color::from_rgb8(10, 30, 80))),
            text_color: Color::from_rgb8(150, 150, 150),
            border: Border::default(),
            shadow: Shadow::default(),
            snap: false,
        },
    }
}

pub fn custom_btn_rounded_style(_theme: &iced::Theme, status: Status) -> Style {
    let border = Border {
        color: Color::from_rgb8(200, 200, 200),
        width: 0.3,
        radius: 5.0.into(),
    };
    match status {
        Status::Pressed => Style {
            background: Some(Background::Color(Color::from_rgb8(70, 70, 70))),
            text_color: Color::from_rgb8(50, 50, 50),
            border,
            shadow: Shadow::default(),
            snap: false,
        },
        Status::Hovered => Style {
            background: Some(Background::Color(Color::from_rgb8(80, 80, 80))),
            text_color: Color::from_rgb8(255, 255, 255),
            border,
            shadow: Shadow::default(),
            snap: false,
        },
        Status::Active => Style {
            background: Some(Background::Color(Color::from_rgb8(50, 50, 50))),
            text_color: Color::from_rgb8(3, 161, 252),
            border,
            shadow: Shadow::default(),
            snap: false,
        },
        Status::Disabled => Style {
            background: None,
            text_color: Color::from_rgb8(150, 150, 150),
            border,
            shadow: Shadow::default(),
            snap: false,
        },
    }
}

pub fn blank_btn_style(_theme: &iced::Theme, status: Status) -> Style {
    match status {
        Status::Pressed => Style {
            background: None,
            text_color: Color::from_rgb8(50, 50, 50),
            border: Border::default(),
            shadow: Shadow::default(),
            snap: false,
        },
        Status::Hovered => Style {
            background: None,
            text_color: Color::from_rgb8(255, 255, 255),
            border: Border::default(),
            shadow: Shadow::default(),
            snap: false,
        },
        Status::Active => Style {
            background: None,
            text_color: Color::from_rgb8(3, 161, 252),
            border: Border::default(),
            shadow: Shadow::default(),
            snap: false,
        },
        Status::Disabled => Style {
            background: None,
            text_color: Color::from_rgb8(150, 150, 150),
            border: Border::default(),
            shadow: Shadow::default(),
            snap: false,
        },
    }
}

pub fn thumb_style(_theme: &iced::Theme, status: Status) -> Style {
    match status {
        Status::Pressed => Style {
            background: None,
            text_color: Color::TRANSPARENT,
            border: Border {
                color: Color::from_rgb8(3, 161, 252),
                width: 2.0,
                radius: 5.0.into(),
            },
            shadow: Shadow::default(),
            snap: false,
        },
        Status::Hovered => Style {
            background: None,
            text_color: Color::TRANSPARENT,
            border: Border {
                color: Color::from_rgb8(3, 161, 252),
                width: 2.0,
                radius: 5.0.into(),
            },
            shadow: Shadow::default(),
            snap: false,
        },
        Status::Active => Style {
            background: None,
            text_color: Color::TRANSPARENT,
            border: Border {
                color: Color::from_rgb8(200, 200, 200),
                width: 2.0,
                radius: 5.0.into(),
            },
            shadow: Shadow::default(),
            snap: false,
        },
        Status::Disabled => Style {
            background: Some(Background::Color(Color::from_rgb8(10, 30, 80))),
            text_color: Color::from_rgb8(150, 150, 150),
            border: Border::default(),
            shadow: Shadow::default(),
            snap: false,
        },
    }
}
pub fn blank_border_style(_theme: &iced::Theme, status: Status) -> Style {
    match status {
        Status::Active => Style {
            background: None,
            text_color: Color::from_rgb8(100, 100, 100),
            border: Border {
                color: Color::from_rgb8(200, 200, 200),
                width: 0.3,
                radius: 5.0.into(),
            },
            shadow: Shadow::default(),
            snap: false,
        },

        Status::Hovered => Style {
            background: Some(Background::Color(Color {
                r: 1.0,
                g: 1.0,
                b: 1.0,
                a: 0.06,
            })),
            text_color: Color::from_rgb8(130, 130, 130),
            border: Border {
                color: Color::from_rgb8(220, 220, 220),
                width: 0.3,
                radius: 5.0.into(),
            },
            shadow: Shadow::default(),
            snap: false,
        },

        Status::Pressed => Style {
            background: Some(Background::Color(Color {
                r: 1.0,
                g: 1.0,
                b: 1.0,
                a: 0.10,
            })),
            text_color: Color::from_rgb8(80, 80, 80),
            border: Border {
                color: Color::from_rgb8(170, 170, 170),
                width: 0.3,
                radius: 5.0.into(),
            },
            shadow: Shadow::default(),
            snap: false,
        },

        Status::Disabled => Style {
            background: None,
            text_color: Color::from_rgb8(150, 150, 150),
            border: Border {
                color: Color::from_rgb8(180, 180, 180),
                width: 0.3,
                radius: 5.0.into(),
            },
            shadow: Shadow::default(),
            snap: false,
        },
    }
}

pub fn danger_style(_theme: &iced::Theme, status: Status) -> Style {
    match status {
        Status::Active => Style {
            background: Some(Background::Color(Color::from_rgb8(220, 50, 47))),
            text_color: Color::WHITE,
            border: Border {
                color: Color::from_rgb8(180, 40, 40),
                width: 0.5,
                radius: 4.0.into(),
            },
            shadow: Shadow::default(),
            snap: false,
        },

        Status::Hovered => Style {
            background: Some(Background::Color(Color::from_rgb8(235, 70, 65))),
            text_color: Color::WHITE,
            border: Border {
                color: Color::from_rgb8(200, 60, 60),
                width: 0.5,
                radius: 4.0.into(),
            },
            shadow: Shadow::default(),
            snap: false,
        },

        Status::Pressed => Style {
            background: Some(Background::Color(Color::from_rgb8(190, 40, 38))),
            text_color: Color::WHITE,
            border: Border {
                color: Color::from_rgb8(160, 30, 30),
                width: 0.5,
                radius: 4.0.into(),
            },
            shadow: Shadow::default(),
            snap: false,
        },

        Status::Disabled => Style {
            background: Some(Background::Color(Color::from_rgb8(120, 60, 60))),
            text_color: Color::from_rgb8(180, 180, 180),
            border: Border {
                color: Color::from_rgb8(120, 120, 120),
                width: 0.5,
                radius: 4.0.into(),
            },
            shadow: Shadow::default(),
            snap: false,
        },
    }
}

// =====Static variant===========

pub fn thumb_single_static(_theme: &iced::Theme, _status: Status) -> Style {
    Style {
        background: None,
        text_color: Color::WHITE,
        border: Border {
            color: Color::from_rgb8(3, 161, 252),
            width: 2.0,
            radius: 5.0.into(),
        },
        snap: false,
        shadow: Default::default(),
    }
}

pub fn red_color_static(_theme: &iced::Theme, _status: Status) -> Style {
    Style {
        background: Some(Color::from_rgb8(220, 50, 47).into()),
        text_color: Color::WHITE,
        border: Border {
            color: Color::from_rgb8(200, 200, 200),
            width: 0.5,
            radius: 4.0.into(),
        },
        snap: false,
        shadow: Default::default(),
    }
}
