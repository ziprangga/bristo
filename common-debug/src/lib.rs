pub use log;
// -------------------------------
// Debug-only Logger Init
// -------------------------------
#[cfg(feature = "debug-dev")]
pub fn init_dev_logger() {
    // Initialize logger; ignore errors if already initialized
    let _ = env_logger::builder()
        .filter_module("cleaner", log::LevelFilter::Debug)
        .filter_module("widget", log::LevelFilter::Debug)
        .filter_module("Bristo", log::LevelFilter::Debug)
        .try_init();
}

#[cfg(not(feature = "debug-dev"))]
pub fn init_dev_logger() {}

// -------------------------------
// Debug-only macro
// -------------------------------

#[cfg(feature = "debug-dev")]
#[macro_export]
macro_rules! debug_dev {
    ($($arg:tt)*) => {
        {
            $crate::log::debug!($($arg)*);
        }
    };
}

#[cfg(not(feature = "debug-dev"))]
#[macro_export]
macro_rules! debug_dev {
    ($($arg:tt)*) => {};
}
