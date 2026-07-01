mod cell;
mod content_cell;
mod header_cell;
pub use cell::Cell;
pub use content_cell::ContentCell;
pub use header_cell::HeaderCell;

use iced::widget::{Column, Container, container, scrollable};
use iced::{Element, Length, Padding, Theme};

// pub enum TableMode {
//     Vertical,
//     Horizontal,
// }

type ContentStyleFn = dyn Fn(&Theme) -> container::Style;

type HeaderStyleFn = dyn Fn(&Theme) -> container::Style;

pub struct Table<M> {
    header: Option<HeaderCell<M>>,
    contents: Vec<ContentCell<M>>,
    content_selected: Option<usize>,
    header_selected: Option<usize>,
    // mode: TableMode,
    spacing: u32,
    width: Option<Length>,
    height: Option<Length>,
    padding: Option<Padding>,
    header_style: Option<Box<HeaderStyleFn>>,
    content_style: Option<Box<ContentStyleFn>>,
}

impl<M: 'static + Clone> Table<M> {
    pub fn new() -> Self {
        Self {
            header: None,
            contents: Vec::new(),
            content_selected: None,
            header_selected: None,
            // mode: TableMode::Vertical,
            spacing: 0,
            width: None,
            height: None,
            padding: None,
            header_style: None,
            content_style: None,
        }
    }

    pub fn header(mut self, header: impl Into<HeaderCell<M>>) -> Self {
        self.header = Some(header.into());
        self
    }

    pub fn header_selected(mut self, index: Option<usize>) -> Self {
        self.header_selected = index;
        self
    }

    pub fn is_header_selected(&self, index: usize) -> bool {
        self.header_selected == Some(index)
    }

    pub fn content(mut self, content: impl Into<ContentCell<M>>) -> Self {
        self.contents.push(content.into());
        self
    }

    pub fn contents<I>(mut self, contents: I) -> Self
    where
        I: IntoIterator<Item = ContentCell<M>>,
    {
        self.contents.extend(contents);
        self
    }

    pub fn content_selected(mut self, index: Option<usize>) -> Self {
        self.content_selected = index;
        self
    }

    pub fn is_content_selected(&self, index: usize) -> bool {
        self.content_selected == Some(index)
    }

    // pub fn mode(mut self, mode: TableMode) -> Self {
    //     self.mode = mode;
    //     self
    // }

    pub fn spacing(mut self, spacing: u32) -> Self {
        self.spacing = spacing;
        self
    }

    pub fn width(mut self, width: Length) -> Self {
        self.width = Some(width);
        self
    }

    pub fn height(mut self, height: Length) -> Self {
        self.height = Some(height);
        self
    }

    pub fn padding(mut self, padding: impl Into<Padding>) -> Self {
        self.padding = Some(padding.into());
        self
    }

    pub fn header_style<F>(mut self, f: F) -> Self
    where
        F: Fn(&Theme) -> container::Style + 'static,
    {
        self.header_style = Some(Box::new(f));
        self
    }

    pub fn content_style<F>(mut self, f: F) -> Self
    where
        F: Fn(&Theme) -> container::Style + 'static,
    {
        self.content_style = Some(Box::new(f));
        self
    }

    pub fn build(self) -> Element<'static, M> {
        let mut scroll_content = Column::new().spacing(self.spacing);

        for (i, content) in self.contents.into_iter().enumerate() {
            let content_element: Element<'static, M> = content.build(i);
            scroll_content = scroll_content.push(content_element);
        }

        let mut scroll_container = Container::new(scrollable(scroll_content));
        if let Some(style_fn) = self.content_style {
            scroll_container = scroll_container.style(move |theme| style_fn(theme));
        }

        let mut header_container = Container::new(if let Some(h) = self.header {
            h.build()
        } else {
            iced::widget::Space::new().into()
        });

        if let Some(style_fn) = self.header_style {
            header_container = header_container.style(move |theme| style_fn(theme));
        }

        let table_col = Column::new().push(header_container).push(scroll_container);

        let mut parent_col = Container::new(table_col);

        if let Some(w) = self.width {
            parent_col = parent_col.width(w);
        }

        if let Some(h) = self.height {
            parent_col = parent_col.height(h);
        }

        if let Some(p) = self.padding {
            parent_col = parent_col.padding(p);
        }

        parent_col.into()
    }
}
