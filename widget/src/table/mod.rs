mod cell;
pub use cell::Cell;

use iced::widget::{Column, Container, Row, container, scrollable};
use iced::{Element, Length, Padding, Theme};
use std::sync::Arc;

const DEFAULT_PADDING_TABLE: Padding = Padding {
    top: 5.0,
    bottom: 5.0,
    right: 5.0,
    left: 5.0,
};

// pub enum TableMode {
//     Vertical,
//     Horizontal,
// }

pub struct ContentCell<M> {
    cells: Vec<Cell<M>>,
}

impl<M: Clone + 'static> ContentCell<M> {
    pub fn new() -> Self {
        Self { cells: Vec::new() }
    }

    pub fn cell(mut self, cell: impl Into<Cell<M>>) -> Self {
        self.cells.push(cell.into());
        self
    }

    pub fn cells<I>(mut self, cells: I) -> Self
    where
        I: IntoIterator<Item = Cell<M>>,
    {
        self.cells.extend(cells);
        self
    }

    pub fn build(self) -> Element<'static, M> {
        let mut row = Row::new();

        for cell in self.cells {
            row = row.push(cell.build());
        }

        // let container = Container::new(row);
        // container.into()
        row.into()
    }
}

impl<M: Clone + 'static> From<Cell<M>> for ContentCell<M> {
    fn from(cell: Cell<M>) -> Self {
        Self::new().cell(cell)
    }
}

pub struct HeaderCell<M> {
    cells: Vec<Cell<M>>,
}

impl<M: Clone + 'static> HeaderCell<M> {
    pub fn new() -> Self {
        Self { cells: Vec::new() }
    }

    pub fn cell(mut self, cell: impl Into<Cell<M>>) -> Self {
        self.cells.push(cell.into());
        self
    }

    pub fn cells<I>(mut self, cells: I) -> Self
    where
        I: IntoIterator<Item = Cell<M>>,
    {
        self.cells.extend(cells);
        self
    }

    pub fn build(self) -> Element<'static, M> {
        let mut row = Row::new().spacing(10);

        for cell in self.cells {
            row = row.push(cell.build());
        }

        // let container = Container::new(row);
        // container.into()
        row.into()
    }
}

impl<M: Clone + 'static> From<Cell<M>> for HeaderCell<M> {
    fn from(cell: Cell<M>) -> Self {
        Self::new().cell(cell)
    }
}

type ContentStyleFn = dyn Fn(usize, &Theme) -> container::Style;

type HeaderStyleFn = dyn Fn(&Theme) -> container::Style;

pub struct Table<M> {
    headers: Vec<HeaderCell<M>>,
    contents: Vec<ContentCell<M>>,
    content_selected: Option<usize>,
    header_selected: Option<usize>,
    // mode: TableMode,
    spacing: u32,
    width: Length,
    height: Option<Length>,
    padding: Padding,
    header_style: Option<Arc<HeaderStyleFn>>,
    content_style: Option<Arc<ContentStyleFn>>,
}

impl<M: 'static + Clone> Table<M> {
    pub fn new() -> Self {
        Self {
            headers: Vec::new(),
            contents: Vec::new(),
            content_selected: None,
            header_selected: None,
            // mode: TableMode::Vertical,
            spacing: 0,
            width: Length::Fill,
            height: None,
            padding: DEFAULT_PADDING_TABLE,
            header_style: None,
            content_style: None,
        }
    }

    pub fn header(mut self, header: impl Into<HeaderCell<M>>) -> Self {
        self.headers.push(header.into());
        self
    }

    pub fn headers<I>(mut self, headers: I) -> Self
    where
        I: IntoIterator<Item = HeaderCell<M>>,
    {
        self.headers.extend(headers);
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

    pub fn header_style<F>(mut self, f: F) -> Self
    where
        F: Fn(&Theme) -> container::Style + 'static,
    {
        self.header_style = Some(Arc::new(f));
        self
    }

    pub fn content_style<F>(mut self, f: F) -> Self
    where
        F: Fn(usize, &Theme) -> container::Style + 'static,
    {
        self.content_style = Some(Arc::new(f));
        self
    }

    pub fn build(self) -> Element<'static, M> {
        let mut scroll_content = Column::new().spacing(self.spacing);

        for (i, content) in self.contents.into_iter().enumerate() {
            let content_element: Element<'static, M> = content.build();

            let mut content_container = Container::new(content_element).padding(self.padding);

            if let Some(style_fn) = self.content_style.clone() {
                content_container = content_container.style(move |theme| style_fn(i, theme));
            }

            scroll_content = scroll_content.push(content_container);
        }

        let mut header_content = Column::new();

        for header in self.headers.into_iter() {
            let header_element: Element<'static, M> = header.build();

            header_content =
                header_content.push(Container::new(header_element).padding(self.padding));
        }

        header_content = header_content.push(scrollable(scroll_content));

        let mut parent_col = Container::new(header_content)
            .width(self.width)
            .padding(self.padding);

        if let Some(h) = self.height {
            parent_col = parent_col.height(h);
        }

        if let Some(style_fn) = self.header_style.clone() {
            parent_col = parent_col.style(move |theme| style_fn(theme));
        }

        parent_col.into()
    }
}
