use std::collections::HashMap;
use std::path::Path;
use std::path::PathBuf;

use crate::app_modal::{ModalAsk, ModalAskMessage};
use cleaner::Cleaner;
use cleaner::Result;
use cleaner::TrashEntry;

use crate::app_status::Status;

// ========

use cleaner::IconCache;
use iced::widget::image;

// ========

#[derive(Debug, Clone)]
pub enum AppMessage {
    DropApp(PathBuf),
    AppPath,
    ProcessApp(PathBuf),
    ScanApp(Result<Cleaner>),
    ReScanApp,

    ModalAsk(ModalAskMessage),
    FindProcs(Result<Cleaner>),
    ConfirmKill(Result<Cleaner>),
    KillFinished(Result<()>, Cleaner),

    UpdateCleaner(Cleaner),
    IconLoaded(IconCache),
    OpenSelectedPath(usize),

    ExportBomFilesLoc,
    ExportBomFiles(Result<PathBuf>),

    MoveToTrash,
    UpdateEntryFiles(Result<Vec<TrashEntry>>),
    ClearList,

    ShowStatus(Status),

    NoOperations,
}

#[derive(Clone)]
pub struct AppState {
    pub app_path: PathBuf,
    pub cleaner: Cleaner,
    pub selected_file: Option<usize>,
    pub show_modal_ask: ModalAsk,
    pub pending_cleaner: Option<Cleaner>,

    pub icon_cache: HashMap<String, image::Handle>,

    pub show_status: Status,
}

impl AppState {
    pub fn new() -> Self {
        let app_path = PathBuf::new();
        let cleaner = Cleaner::default();
        let selected_file = None;
        let show_modal_ask = ModalAsk::default();
        let pending_cleaner = None;

        let icon_cache = HashMap::new();

        let show_status = Status::default();

        Self {
            app_path,
            cleaner,
            selected_file,
            show_modal_ask,
            pending_cleaner,

            icon_cache,

            show_status,
        }
    }

    pub fn reset(&mut self) {
        self.app_path.clear();
        self.cleaner.reset();
        self.selected_file = None;
        self.pending_cleaner = None;
        self.show_status = Status::default();
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
