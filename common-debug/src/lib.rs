pub use log;
// -------------------------------
// Debug-only Logger Init
// -------------------------------
#[cfg(all(debug_assertions, feature = "debug-dev"))]
pub fn init_dev_logger() {
    // Initialize logger; ignore errors if already initialized
    let _ = env_logger::builder()
        // .filter_level(log::LevelFilter::Info)
        .filter_module("cleaner", log::LevelFilter::Debug)
        .filter_module("widget", log::LevelFilter::Debug)
        .filter_module("Bristo", log::LevelFilter::Debug)
        .try_init();
}

#[cfg(not(feature = "debug-dev"))]
pub fn init_dev_logger() {
    // no-op
}

// -------------------------------
// Debug-only macro
// -------------------------------

#[macro_export]
macro_rules! debug_dev {
    ($($arg:tt)*) => {
        #[cfg(debug_assertions)]
        // #[cfg(all(debug_assertions, feature = "debug-dev"))]
        {
            $crate::log::debug!($($arg)*);
        }
    };
}
