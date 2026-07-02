use iced::Element;
use iced::Theme;
use iced::widget::Id;
use iced::widget::Row;
use iced::widget::{Container, container};
use iced::{Length, Padding, alignment};

use crate::table::cell::Cell;

pub type ContentCellId = Id;
type ContentCellStyleFn = dyn Fn(usize, Option<&ContentCellId>, &Theme) -> container::Style;

pub struct ContentCell<M> {
    cells: Vec<Cell<M>>,
    id: Option<ContentCellId>,
    width: Option<Length>,
    height: Option<Length>,
    align_x: Option<alignment::Horizontal>,
    align_y: Option<alignment::Vertical>,
    spacing: Option<u32>,
    padding: Option<Padding>,
    style_fn: Option<Box<ContentCellStyleFn>>,
}

impl<M: Clone + 'static> ContentCell<M> {
    pub fn new() -> Self {
        Self {
            cells: Vec::new(),
            id: None,
            width: None,
            height: None,
            align_x: None,
            align_y: None,
            spacing: None,
            padding: None,
            style_fn: None,
        }
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

    pub fn cell_element(
        mut self,
        content: impl Into<Element<'static, M>>,
        id: impl Into<Id>,
    ) -> Self {
        self.cells.push(Cell::new(content).id(id));
        self
    }

    pub fn cells_layout<F>(mut self, id: &Id, f: F) -> Self
    where
        F: FnOnce(Cell<M>) -> Cell<M>,
    {
        if let Some(index) = self.cells.iter().position(|cell| cell.get_id() == Some(id)) {
            // Remove the cell, transform it via the closure, and put it back
            let cell = self.cells.remove(index);
            self.cells.insert(index, f(cell));
        }
        self
    }

    pub fn id(mut self, id: impl Into<Id>) -> Self {
        self.id = Some(id.into());
        self
    }

    pub fn get_id(&self) -> Option<&Id> {
        self.id.as_ref()
    }

    pub fn width(mut self, width: Length) -> Self {
        self.width = Some(width);
        self
    }

    pub fn height(mut self, height: Length) -> Self {
        self.height = Some(height);
        self
    }

    pub fn align_x(mut self, x: impl Into<alignment::Horizontal>) -> Self {
        self.align_x = Some(x.into());
        self
    }

    pub fn align_y(mut self, y: impl Into<alignment::Vertical>) -> Self {
        self.align_y = Some(y.into());
        self
    }

    pub fn spacing(mut self, spacing: u32) -> Self {
        self.spacing = Some(spacing);
        self
    }

    pub fn padding(mut self, p: impl Into<Padding>) -> Self {
        self.padding = Some(p.into());
        self
    }

    pub fn style<F>(mut self, f: F) -> Self
    where
        F: Fn(usize, Option<&ContentCellId>, &Theme) -> container::Style + 'static,
    {
        self.style_fn = Some(Box::new(f));
        self
    }

    pub fn build(self, index: usize) -> Element<'static, M> {
        let content_cell_id = self.id.clone();
        let padding = self.padding;
        let style_fn = self.style_fn;

        let mut cell_content = Row::new();

        if let Some(spacing) = self.spacing {
            cell_content = cell_content.spacing(spacing as f32);
        }

        for (i, cell) in self.cells.into_iter().enumerate() {
            let content_element: Element<'static, M> = cell.build(i);
            cell_content = cell_content.push(content_element);
        }

        let mut container = Container::new(cell_content);

        if let Some(ref id) = content_cell_id {
            container = container.id(id.clone());
        }

        if let Some(w) = self.width {
            container = container.width(w);
        }

        if let Some(h) = self.height {
            container = container.height(h);
        }

        if let Some(x) = self.align_x {
            container = container.align_x(x);
        }
        if let Some(y) = self.align_y {
            container = container.align_y(y);
        }

        if let Some(p) = padding {
            container = container.padding(p);
        }

        if let Some(style_fn) = style_fn {
            container =
                container.style(move |theme| style_fn(index, content_cell_id.as_ref(), theme));
        }

        container.into()
    }
}

impl<M: Clone + 'static> From<Cell<M>> for ContentCell<M> {
    fn from(cell: Cell<M>) -> Self {
        Self::new().cell(cell)
    }
}
