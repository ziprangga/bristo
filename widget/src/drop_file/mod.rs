mod drop_zone;
pub use drop_zone::DropZone;

use iced::Element;

pub struct DropFile<M> {
    content: Option<DropZone<M>>,
    // on_drop: Option<Box<dyn Fn(std::path::PathBuf) -> M + 'static>>,
    // on_hover: Option<M>,
}

impl<M: Clone + 'static> DropFile<M> {
    pub fn new() -> Self {
        Self {
            content: None,
            // on_drop: None,
            // on_hover: None,
        }
    }

    pub fn content_element(mut self, content: impl Into<Element<'static, M>>) -> Self {
        let base = self
            .content
            .unwrap_or_else(|| DropZone::new(content.into()));
        self.content = Some(base);
        self
    }

    pub fn content_layout<F>(mut self, f: F) -> Self
    where
        F: FnOnce(DropZone<M>) -> DropZone<M>,
    {
        if let Some(c) = self.content {
            self.content = Some(f(c));
        }
        self
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
        let content = match self.content {
            Some(c) => c.build(),
            None => return iced::widget::Space::new().into(),
        };

        content
    }
}
