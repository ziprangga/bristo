// Copyright 2026 ziprangga
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

mod app_modal;
mod app_state;
mod app_task;
// mod app_tree_view;
pub mod app_status;
mod app_update;
mod app_window;
pub mod ui_element;

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
