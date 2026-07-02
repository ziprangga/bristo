mod cell;
mod content_cell;
mod header_cell;
pub use cell::{Cell, CellId};
pub use content_cell::{ContentCell, ContentCellId};
pub use header_cell::HeaderCell;

use iced::widget::Id;
use iced::widget::{Column, Container, scrollable};
use iced::{Element, Length, Padding};

// pub enum TableMode {
//     Vertical,
//     Horizontal,
// }

pub struct Table<M> {
    header: Option<HeaderCell<M>>,
    contents: Option<Vec<ContentCell<M>>>,
    // mode: TableMode,
    spacing: u32,
    width: Option<Length>,
    height: Option<Length>,
    padding: Option<Padding>,
}

impl<M: 'static + Clone> Table<M> {
    pub fn new() -> Self {
        Self {
            header: None,
            contents: None,
            // mode: TableMode::Vertical,
            spacing: 0,
            width: None,
            height: None,
            padding: None,
        }
    }

    pub fn header(mut self, header: impl Into<HeaderCell<M>>) -> Self {
        self.header = Some(header.into());
        self
    }

    pub fn header_element(
        mut self,
        content: impl Into<Element<'static, M>>,
        id: impl Into<Id>,
    ) -> Self {
        let mut header = self.header.unwrap_or_else(HeaderCell::new);
        header = header.cell_element(content, id);
        self.header = Some(header);
        self
    }

    pub fn header_layout<F>(mut self, f: F) -> Self
    where
        F: FnOnce(HeaderCell<M>) -> HeaderCell<M>,
    {
        if let Some(header) = self.header.take() {
            self.header = Some(f(header));
        }
        self
    }

    pub fn content(mut self, content: impl Into<ContentCell<M>>) -> Self {
        let list = self.contents.get_or_insert_with(Vec::new);
        list.push(content.into());
        self
    }

    pub fn contents<I>(mut self, contents: I) -> Self
    where
        I: IntoIterator<Item = ContentCell<M>>,
    {
        let list = self.contents.get_or_insert_with(Vec::new);
        list.extend(contents);
        self
    }

    pub fn content_element(
        mut self,
        element_content: impl Into<Element<'static, M>>,
        cell_id: impl Into<Id>,
        content_id: ContentCellId,
    ) -> Self {
        // Create a child cell out of your raw UI content element
        let inner_cell = Cell::new(element_content).id(cell_id);

        // Wrap it inside a new ContentCell row container
        let new_content = ContentCell::new().id(content_id).cell(inner_cell);

        // Push it onto your row collection vector
        let list = self.contents.get_or_insert_with(Vec::new);
        list.push(new_content);
        self
    }

    pub fn contents_layout<F>(mut self, content_id: &ContentCellId, f: F) -> Self
    where
        F: FnOnce(ContentCell<M>) -> ContentCell<M>,
    {
        // Search content cell collection for a matching Id
        if let Some(mut list) = self.contents.take() {
            // Find the content position
            if let Some(index) = list
                .iter()
                .position(|content| content.get_id() == Some(content_id))
            {
                let content_cell = list.remove(index);
                list.insert(index, f(content_cell));
            }
            // Put the updated vector back
            self.contents = Some(list);
        }
        self
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

    pub fn build(self) -> Element<'static, M> {
        let mut scroll_content = Column::new().spacing(self.spacing);

        if let Some(list) = self.contents {
            for (i, content) in list.into_iter().enumerate() {
                let content_element: Element<'static, M> = content.build(i);
                scroll_content = scroll_content.push(content_element);
            }
        }

        let scroll_container = Container::new(scrollable(scroll_content));

        let header_container = Container::new(if let Some(h) = self.header {
            h.build()
        } else {
            iced::widget::Space::new().into()
        });

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
