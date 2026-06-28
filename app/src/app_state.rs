use anyhow::Result;
use std::path::PathBuf;

use crate::app_modal::{ModalAsk, ModalAskMessage};
use cleaner::Cleaner;
use cleaner::TrashEntry;
use simple_status::{ChannelKind, Channels, Status, init_channels};

#[derive(Debug, Clone)]
pub enum AppMessage {
    DropApp(PathBuf),
    AppPath,
    ProcessApp(PathBuf),
    ScanApp(Result<Cleaner, String>),

    ModalAsk(ModalAskMessage),
    FindProcs(Result<Cleaner, String>),
    ConfirmKill(Result<Cleaner, String>),

    UpdateCleaner(Cleaner),
    OpenSelectedPath(usize),

    ExportBomFilesLoc,
    ExportBomFiles(Result<PathBuf, String>),

    MoveToTrash,
    UpdateEntryFiles(Result<Vec<TrashEntry>, String>),
    ClearList,
    ShowStatus(Status),

    NoOperations,
}

#[derive(Clone)]
pub struct AppState {
    pub app_path: PathBuf,
    pub show_status: Status,
    pub channel: Channels,

    pub cleaner: Cleaner,
    pub selected_file: Option<usize>,
    pub show_modal_ask: ModalAsk,
    pub pending_cleaner: Option<Cleaner>,
}

impl AppState {
    pub fn new(buffer: usize) -> Self {
        let app_path = PathBuf::new();
        let show_status = Status::default();
        let channel = init_channels(buffer, ChannelKind::Broadcast);

        let cleaner = Cleaner::default();
        let selected_file = None;

        let show_modal_ask = ModalAsk::default();
        let pending_cleaner = None;

        Self {
            app_path,
            show_status,
            channel,
            cleaner,
            selected_file,
            show_modal_ask,
            pending_cleaner,
        }
    }

    pub fn reset(&mut self) {
        self.app_path.clear();
        self.cleaner.reset();
        self.selected_file = None;
        self.show_status.reset_event();
        self.pending_cleaner = None;
    }
}
