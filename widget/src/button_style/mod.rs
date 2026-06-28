mod button_image;
mod button_text;

pub use button_image::BtnImage;
pub use button_text::BtnText;

use iced::Element;
use iced::widget::button::{Status, Style};
use iced::{Background, Border, Color, Padding, Shadow};

const DEFAULT_PADDING_BUTTON: Padding = Padding {
    top: 5.0,
    bottom: 5.0,
    right: 10.0,
    left: 10.0,
};

pub enum ButtonContent<M> {
    Text(BtnText<M>),
    Image(BtnImage<M>),
}
pub struct CustomButton<M> {
    content: ButtonContent<M>,
}

impl<M: Clone + 'static> CustomButton<M> {
    pub fn text(label: impl Into<String>) -> BtnText<M> {
        BtnText::new(label)
    }
    pub fn image(image: iced::widget::Image) -> BtnImage<M> {
        BtnImage::new(image)
    }
    pub fn build(self) -> Element<'static, M> {
        match self.content {
            ButtonContent::Text(btn_text) => btn_text.build(),
            ButtonContent::Image(btn_image) => btn_image.build(),
        }
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
