mod app_modal;
mod app_state;
mod app_status;
mod app_task;
mod app_update;
mod app_window;

use crate::app_state::{AppMessage, AppState};
use crate::app_update::{subscription, update};
use crate::app_window::view;
use iced::{Size, Task, application, window};
use mini_logger::debug;

fn init() -> (AppState, Task<AppMessage>) {
    let app_state = AppState::new();
    (app_state, Task::none())
}

fn main() {
    mini_logger::init();
    debug!("Starting main app in debug mode...");

    application(init, update, view)
        .title("Bristo")
        .position(window::Position::Centered)
        .window(window::Settings {
            size: Size::new(600.0, 350.0),
            min_size: Some(Size::new(600.0, 350.0)),
            resizable: true,
            decorations: true,
            ..Default::default()
        })
        .subscription(subscription)
        .run()
        .expect("Failed to run application");
}
