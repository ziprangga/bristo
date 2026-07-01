use iced::Element;
use iced::Theme;
use iced::widget::Row;
use iced::widget::{Container, container};
use iced::{Length, Padding, alignment};
use std::sync::Arc;

use crate::table::cell::Cell;

type HeaderCellStyleFn = dyn Fn(&Theme) -> container::Style;

pub struct HeaderCell<M> {
    cells: Vec<Cell<M>>,
    width: Option<Length>,
    height: Option<Length>,
    align_x: Option<alignment::Horizontal>,
    align_y: Option<alignment::Vertical>,
    spacing: Option<u32>,
    padding: Option<Padding>,
    style_fn: Option<Arc<HeaderCellStyleFn>>,
}

impl<M: Clone + 'static> HeaderCell<M> {
    pub fn new() -> Self {
        Self {
            cells: Vec::new(),
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
        F: Fn(&Theme) -> container::Style + 'static,
    {
        self.style_fn = Some(Arc::new(f));
        self
    }

    pub fn build(self) -> Element<'static, M> {
        let mut row = Row::new();

        if let Some(spacing) = self.spacing {
            row = row.spacing(spacing as f32);
        }

        for cell in self.cells {
            row = row.push(cell.build());
        }

        let mut container = Container::new(row);

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

        let row_padding = self.padding.unwrap_or(Padding::new(0.0));
        container = container.padding(row_padding);

        if let Some(style_fn) = self.style_fn {
            container = container.style(move |theme| style_fn(theme));
        }

        container.into()
    }
}

impl<M: Clone + 'static> From<Cell<M>> for HeaderCell<M> {
    fn from(cell: Cell<M>) -> Self {
        Self::new().cell(cell)
    }
}
