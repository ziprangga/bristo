use cleaner::FileEntry;
use std::path::Path;

#[derive(Debug, Clone)]
pub enum TreeView {
    App(usize, FileEntry),
    AscGroup,
    AscFile(usize, FileEntry),
    BtmGroup,
    BtmFile(usize, FileEntry),
}

impl TreeView {
    pub fn from_enumerated_entries(entries: Vec<(usize, FileEntry)>) -> Vec<Self> {
        let mut app = None;
        let mut asc = Vec::new();
        let mut btm = Vec::new();

        for (i, entry) in entries {
            match &entry {
                FileEntry::AppPath(_) => app = Some(TreeView::App(i, entry)),
                FileEntry::AscFiles(_) => asc.push(TreeView::AscFile(i, entry)),
                FileEntry::BtmFiles(_) => btm.push(TreeView::BtmFile(i, entry)),
            }
        }

        let mut rows = Vec::new();

        if let Some(app) = app {
            rows.push(app);
        }

        if !asc.is_empty() {
            rows.push(TreeView::AscGroup);
            rows.extend(asc);
        }

        if !btm.is_empty() {
            rows.push(TreeView::BtmGroup);
            rows.extend(btm);
        }

        rows
    }

    pub fn entry_index(&self) -> Option<usize> {
        match self {
            TreeView::App(i, _) | TreeView::AscFile(i, _) | TreeView::BtmFile(i, _) => Some(*i),

            TreeView::AscGroup | TreeView::BtmGroup => None,
        }
    }

    pub fn entry(&self) -> Option<&FileEntry> {
        match self {
            TreeView::App(_, e) | TreeView::AscFile(_, e) | TreeView::BtmFile(_, e) => Some(e),

            TreeView::AscGroup | TreeView::BtmGroup => None,
        }
    }

    pub fn level(&self) -> usize {
        match self {
            Self::App(_, _) => 0,
            Self::AscGroup | Self::BtmGroup => 1,
            Self::AscFile(_, _) | Self::BtmFile(_, _) => 2,
        }
    }

    pub fn as_name(&self) -> &str {
        match self {
            Self::App(_, entry) => entry.as_name(),
            Self::AscGroup => "Associate Files",
            Self::AscFile(_, entry) => entry.as_name(),
            Self::BtmGroup => "Btm Files",
            Self::BtmFile(_, entry) => entry.as_name(),
        }
    }

    pub fn as_path(&self) -> Option<&Path> {
        match self {
            Self::App(_, entry) => Some(entry.as_path()),
            Self::AscFile(_, entry) => Some(entry.as_path()),
            Self::BtmFile(_, entry) => Some(entry.as_path()),

            Self::AscGroup | Self::BtmGroup => None,
        }
    }
}
