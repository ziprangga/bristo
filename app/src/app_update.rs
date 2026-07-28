use crate::app_modal::ModalAskMessage;
use crate::app_state::{AppMessage, AppState};
use crate::app_status::Status;
use crate::app_task::find_app_process_async;
use crate::app_task::get_icon_asset_async;
use crate::app_task::kill_app_process_async;
use crate::app_task::open_loc_async;
use crate::app_task::process_app;
use crate::app_task::save_bom_logs_async;
use crate::app_task::scan_app_async;
use crate::app_task::set_input_path;
use crate::app_task::set_output_path;
use crate::app_task::trash_app_async;

use cleaner::ErrorKind;
use iced::{Subscription, Task, futures::StreamExt};
use mini_logger::debug;
use simple_status::{ChannelKind, create_channels};
use std::collections::HashMap;
use std::sync::Arc;

pub fn update(state: &mut AppState, message: AppMessage) -> Task<AppMessage> {
    match message {
        AppMessage::DropApp(app_path) => {
            state.reset();
            Task::done(AppMessage::ProcessApp(app_path.to_path_buf()))
        }

        AppMessage::AppPath => {
            state.reset();
            Task::perform(set_input_path(), |res| match res {
                Ok(path) => AppMessage::ProcessApp(path.to_path_buf()),
                Err(e) => {
                    let event = Status::new().with_status_error(e);
                    AppMessage::ShowStatus(event)
                }
            })
        }

        AppMessage::ProcessApp(app_path) => {
            state.app_path = app_path.clone();
            let channel = create_channels(100, ChannelKind::Mpsc);
            let emitter = channel.get_emitter();
            let add_app = {
                let path_input = state.app_path.clone();
                Task::perform(
                    async move {
                        let result = process_app(path_input, Some(emitter)).await;
                        match result {
                            Ok(cleaner) => AppMessage::FindProcs(cleaner),
                            Err(err) => {
                                let failure_status = Status::new().with_status_error(err);
                                AppMessage::ShowStatus(failure_status)
                            }
                        }
                    },
                    |msg| msg,
                )
            };

            let status_task = channel
                .stream()
                .map(|s| {
                    Task::stream(s.map(|event| {
                        let wrapped_status = Status::new().with_status_event(event);
                        AppMessage::ShowStatus(wrapped_status)
                    }))
                })
                .unwrap_or_else(Task::none);

            Task::batch(vec![add_app, status_task])
        }

        AppMessage::FindProcs(cleaner) => {
            let channel = create_channels(100, ChannelKind::Mpsc);

            let emitter = channel.get_emitter();

            let find_task =
                Task::perform(
                    find_app_process_async(cleaner, Some(emitter)),
                    |res| match res {
                        Ok(cleaner) => {
                            if cleaner.as_app_profile().as_app_procs().is_empty() {
                                AppMessage::ScanApp(cleaner)
                            } else {
                                AppMessage::ConfirmKill(cleaner)
                            }
                        }
                        Err(err) => {
                            let event = Status::new().with_status_error(err);
                            AppMessage::ShowStatus(event)
                        }
                    },
                );

            let progress_task = channel
                .stream()
                .map(|s| {
                    Task::stream(s.map(|event| {
                        let wrapped_status = Status::new().with_status_event(event);
                        AppMessage::ShowStatus(wrapped_status)
                    }))
                })
                .unwrap_or_else(Task::none);

            Task::batch(vec![find_task, progress_task])
        }

        AppMessage::ConfirmKill(cleaner) => {
            state.pending_cleaner = Some(cleaner);

            state.show_modal_ask.set_message(format!(
                    "The app '{}' is still running.\nDo you want to kill its running process?\nBe careful to save your work first before continuing.",
                    state.pending_cleaner
                        .as_ref()
                        .unwrap()
                        .as_app_profile()
                        .as_app_metadata()
                        .as_name()
                ));

            Task::none()
        }

        AppMessage::ModalAsk(msg) => match msg {
            ModalAskMessage::ConfirmMsg(answer) => {
                state
                    .show_modal_ask
                    .update(ModalAskMessage::ConfirmMsg(answer));
                let channel = create_channels(100, ChannelKind::Mpsc);

                let cleaner = state.pending_cleaner.take().unwrap();
                if !answer {
                    return Task::done(AppMessage::ScanApp(cleaner));
                }

                let emitter = channel.get_emitter();
                let cleaner_arc = Arc::new(cleaner);

                let confirm_task = Task::perform(
                    kill_app_process_async(cleaner_arc.clone(), Some(emitter)),
                    move |res| {
                        let cleaner = Arc::try_unwrap(cleaner_arc).unwrap_or_else(|c| (*c).clone());
                        AppMessage::KillFinished(res, cleaner)
                    },
                );

                let status_task = channel
                    .stream()
                    .map(|s| {
                        Task::stream(s.map(|status_event| {
                            let wrapped_status = Status::new().with_status_event(status_event);
                            AppMessage::ShowStatus(wrapped_status)
                        }))
                    })
                    .unwrap_or_else(Task::none);

                Task::batch(vec![confirm_task, status_task])
            }
        },

        AppMessage::KillFinished(result, cleaner) => {
            let status = match result {
                Ok(()) => Status::new().with_status_success("killed process"),
                Err(err) => Status::new().with_status_error(err),
            };

            Task::batch(vec![
                Task::done(AppMessage::ShowStatus(status)),
                Task::done(AppMessage::ScanApp(cleaner)),
            ])
        }

        AppMessage::ScanApp(cleaner) => {
            let channel = create_channels(100, ChannelKind::Mpsc);

            let emitter = channel.get_emitter();

            let scan_task =
                Task::perform(scan_app_async(cleaner, Some(emitter)), |res| match res {
                    Ok(cleaner) => AppMessage::UpdateCleaner(cleaner),
                    Err(err) => {
                        let event = Status::new().with_status_error(err);
                        AppMessage::ShowStatus(event)
                    }
                });

            let progress_task = channel
                .stream()
                .map(|s| {
                    Task::stream(s.map(|status_event| {
                        let wrapped_status = Status::new().with_status_event(status_event);
                        AppMessage::ShowStatus(wrapped_status)
                    }))
                })
                .unwrap_or_else(Task::none);

            return Task::batch(vec![scan_task, progress_task]);
        }

        AppMessage::UpdateCleaner(cleaner) => {
            state.cleaner = cleaner;

            let mut tasks = Vec::new();

            for (_i, entry) in state.cleaner.all_entries_enumerate() {
                let path_buf = entry.as_path().to_path_buf();

                // Build the asynchronous background generator task blueprint
                let task = Task::perform(get_icon_asset_async(path_buf, 64.0), |res| match res {
                    // Send a dedicated IconLoaded message to prevent infinite loops
                    Ok(backend_cache) => AppMessage::IconLoaded(backend_cache),
                    Err(err) => {
                        let event = Status::new().with_status_error(err);
                        AppMessage::ShowStatus(event)
                    }
                });

                tasks.push(task);
            }

            Task::batch(tasks)
        }

        AppMessage::IconLoaded(backend_cache) => {
            // Consumes the backend cache map, loads into state.icon_cache, and drops the backend instantly
            state.consume_backend_icon(backend_cache);

            Task::none()
        }

        AppMessage::ReScanApp => {
            // Check if the path is empty (meaning no app has been selected yet)
            if state.app_path.as_os_str().is_empty() {
                let validation_error = cleaner::ErrorKind::failed()
                    .with_summary("Re-scan operation failed")
                    .with_reason("No application path found to re-scan.");
                let warning_status = Status::new().with_status_error(validation_error);
                Task::done(AppMessage::ShowStatus(warning_status))
            } else {
                // Forward the app path back to the initialization logic
                let path_to_process = state.app_path.clone();
                Task::done(AppMessage::ProcessApp(path_to_process))
            }
        }

        AppMessage::OpenSelectedPath(index) => {
            state.selected_file = Some(index);
            debug!("Clicked path: {:?}", index);

            let entries = state.cleaner.all_entries_enumerate();

            if let Some((_i, entry)) = entries.get(index) {
                let path = entry.as_path().to_path_buf();

                return Task::perform(open_loc_async(path), |res| match res {
                    Ok(()) => AppMessage::NoOperations,
                    Err(err) => AppMessage::ShowStatus(Status::new().with_status_error(err)),
                });
            }
            Task::none()
        }

        AppMessage::ExportBomFilesLoc => Task::perform(set_output_path(), |res| match res {
            Ok(path) => AppMessage::ExportBomFiles(path),
            Err(e) => {
                let event = Status::new().with_status_error(e);
                AppMessage::ShowStatus(event)
            }
        }),

        AppMessage::ExportBomFiles(path) => {
            let cleaner = state.cleaner.clone();
            Task::perform(save_bom_logs_async(cleaner, path), |res| match res {
                Ok(()) => {
                    let event = Status::new().with_status_success("BOM file saved successfully");
                    AppMessage::ShowStatus(event)
                }
                Err(err) => {
                    let event = Status::new().with_status_error(err);
                    AppMessage::ShowStatus(event)
                }
            })
        }

        AppMessage::MoveToTrash => {
            let cleaner = std::mem::take(&mut state.cleaner);

            Task::perform(trash_app_async(cleaner), |res| match res {
                Ok(cleaner) => AppMessage::UpdateEntryFiles(cleaner),
                Err(err) => AppMessage::ShowStatus(Status::new().with_status_error(err)),
            })
        }

        AppMessage::UpdateEntryFiles(cleaner) => {
            state.cleaner = cleaner;

            let failed = state.cleaner.as_trash_entry().failed_path();

            if failed.is_empty() {
                state.show_status = Status::new().with_status_success("App moved to Trash");
            } else {
                let mut missing = 0usize;
                let mut grouped: HashMap<ErrorKind, usize> = HashMap::new();

                for (path, error) in failed {
                    if !path.as_path().exists() {
                        missing += 1;
                        continue;
                    }

                    *grouped.entry(error.clone()).or_insert(0) += 1;
                }

                let mut report = Vec::new();

                if missing > 0 {
                    report.push(format!("{missing} items path not exist"));
                }

                report.extend(grouped.into_iter().map(|(error, count)| {
                    let items = if count == 1 { "item" } else { "items" };

                    let kind = error.kind().as_str().to_lowercase();

                    let reason = error.reason().unwrap_or("Unknown");

                    format!("{count} {items} {kind} - {reason}")
                }));

                let reason = report.join("\n");

                let error = ErrorKind::failed().with_reason(reason);

                state.show_status = Status::new().with_status_error(error);
            }

            Task::none()
        }

        AppMessage::ClearList => {
            state.reset();
            Task::none()
        }

        AppMessage::ShowStatus(new_status) => {
            state.show_status.update_status(new_status);
            Task::none()
        }

        AppMessage::NoOperations => Task::none(),
    }
}

pub fn subscription(_state: &AppState) -> Subscription<AppMessage> {
    let file_drop_sub = iced::event::listen().map(|event| match event {
        iced::Event::Window(iced::window::Event::FileDropped(path)) => AppMessage::DropApp(path),
        _ => AppMessage::NoOperations,
    });

    Subscription::batch(vec![file_drop_sub])
}
