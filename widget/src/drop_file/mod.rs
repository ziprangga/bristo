mod drop_zone;
pub use drop_zone::DropZone;

use iced::{Border, Color, Element, Padding, Theme, widget::container};

const DEFAULT_PADDING_DROP: Padding = Padding {
    top: 5.0,
    bottom: 5.0,
    right: 5.0,
    left: 5.0,
};

pub struct DropFile<M> {
    content: DropZone<M>,
    // on_drop: Option<Box<dyn Fn(std::path::PathBuf) -> M + 'static>>,
    // on_hover: Option<M>,
}

impl<M: Clone + 'static> DropFile<M> {
    pub fn new(content: impl Into<Element<'static, M>>) -> Self {
        Self {
            content: DropZone::new(content),
            // on_drop: None,
            // on_hover: None,
        }
    }

    pub fn content(content: impl Into<Element<'static, M>>) -> DropZone<M> {
        DropZone::new(content)
    }

    // pub fn on_drop<F>(mut self, f: F) -> Self
    // where
    //     F: Fn(std::path::PathBuf) -> M + 'static,
    // {
    //     self.on_drop = Some(Box::new(f));
    //     self
    // }

    // pub fn on_hover(mut self, msg: M) -> Self {
    //     self.on_hover = Some(msg);
    //     self
    // }

    pub fn build(self) -> Element<'static, M> {
        let content = self.content.build();
        content
    }
}

pub fn default_drop_style(_theme: &Theme) -> container::Style {
    container::Style {
        background: None,
        text_color: None,
        border: Border {
            color: Color {
                r: 1.0,
                g: 1.0,
                b: 1.0,
                a: 0.06,
            },
            width: 1.0,
            radius: 8.0.into(),
        },
        snap: false,
        shadow: Default::default(),
    }
}
