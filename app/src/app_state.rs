use anyhow::Result;
use std::collections::HashMap;
use std::path::Path;
use std::path::PathBuf;

use crate::app_modal::{ModalAsk, ModalAskMessage};
use cleaner::Cleaner;
use cleaner::TrashEntry;
use simple_status::{ChannelKind, Channels, Status, init_channels};

// ========

use cleaner::IconCache;
use iced::widget::image;

// ========

#[derive(Debug, Clone)]
pub enum AppMessage {
    DropApp(PathBuf),
    AppPath,
    ProcessApp(PathBuf),
    ScanApp(Result<Cleaner, String>),
    ReScanApp,

    ModalAsk(ModalAskMessage),
    FindProcs(Result<Cleaner, String>),
    ConfirmKill(Result<Cleaner, String>),

    UpdateCleaner(Cleaner),
    IconLoaded(IconCache),
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

    pub icon_cache: HashMap<String, image::Handle>,
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

        let icon_cache = HashMap::new();

        Self {
            app_path,
            show_status,
            channel,
            cleaner,
            selected_file,
            show_modal_ask,
            pending_cleaner,

            icon_cache,
        }
    }

    pub fn reset(&mut self) {
        self.app_path.clear();
        self.cleaner.reset();
        self.selected_file = None;
        self.show_status.reset_event();
        self.pending_cleaner = None;
    }

    pub fn get_cached_icon(&self, path: &Path) -> Option<image::Handle> {
        // Call the backend source of truth directly
        let cache_key = IconCache::get_cache_key(path);

        self.icon_cache.get(&cache_key).cloned()
    }

    pub fn consume_backend_icon(&mut self, backend: IconCache) {
        // By using a for-loop over the map directly, we take ownership of its elements
        for (key, (width, height, rgba_bytes)) in backend.icon_cache_owned() {
            // Convert the raw pixel vector into Iced's GPU-ready Handle format
            let ui_handle = image::Handle::from_rgba(
                width as u32,
                height as u32,
                rgba_bytes, // Consumes the inner Vec<u8> directly
            );

            // Store it in the AppState icon cache
            self.icon_cache.insert(key, ui_handle);
        }
        // At this point, the `backend` variable goes out of scope and is completely dropped,
        // freeing up all backend memory buffers instantly.
    }
}
