use crate::app_modal::ModalAskMessage;
use crate::app_state::{AppMessage, AppState};
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

use cleaner::TrashStatus;
use iced::{Subscription, Task, futures::StreamExt};
use mini_logger::debug;
use simple_status::status;
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
                    let event = status!("{}", e.to_string());
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
                            Ok(cleaner) => AppMessage::FindProcs(Ok(cleaner)),
                            Err(err) => {
                                let failure_status =
                                    status!(action: "Failed", message: err.to_string(),);
                                AppMessage::ShowStatus(failure_status)
                            }
                        }
                    },
                    |msg| msg,
                )
            };

            let status_task = channel
                .stream()
                .map(|s| Task::stream(s.map(AppMessage::ShowStatus)))
                .unwrap_or_else(Task::none);

            Task::batch(vec![add_app, status_task])
        }

        AppMessage::FindProcs(result) => {
            let channel = create_channels(100, ChannelKind::Mpsc);
            if let Ok(cleaner) = result {
                let emitter = channel.get_emitter();

                let find_task = Task::perform(
                    find_app_process_async(cleaner, Some(emitter)),
                    |res| match res {
                        Ok(cleaner) => {
                            if cleaner.as_app_profile().as_app_procs().is_empty() {
                                AppMessage::ScanApp(Ok(cleaner))
                            } else {
                                AppMessage::ConfirmKill(Ok(cleaner))
                            }
                        }
                        Err(err) => {
                            let event = status!("{}", err.to_string());
                            AppMessage::ShowStatus(event)
                        }
                    },
                );

                let progress_task = channel
                    .stream()
                    .map(|s| Task::stream(s.map(AppMessage::ShowStatus)))
                    .unwrap_or_else(Task::none);

                Task::batch(vec![find_task, progress_task])
            } else {
                Task::none()
            }
        }

        AppMessage::ConfirmKill(result) => {
            if let Ok(cleaner) = result {
                state.pending_cleaner = Some(cleaner);

                state.show_modal_ask.set_message(format!(
                    "The app '{}' is still running.\nDo you want to kill its running process?\nBe careful to save your work first before continuing.",
                    state.pending_cleaner
                        .as_ref()
                        .unwrap()
                        .as_app_profile()
                        .as_app_metadata()
                        .as_info()
                        .as_name()
                ));

                Task::none()
            } else {
                Task::none()
            }
        }

        AppMessage::ModalAsk(msg) => match msg {
            ModalAskMessage::ConfirmMsg(answer) => {
                state
                    .show_modal_ask
                    .update(ModalAskMessage::ConfirmMsg(answer));
                let channel = create_channels(100, ChannelKind::Mpsc);

                let cleaner = state.pending_cleaner.take().unwrap();
                if !answer {
                    return Task::done(AppMessage::ScanApp(Ok(cleaner)));
                }

                let emitter = channel.get_emitter();
                let cleaner_arc = Arc::new(cleaner);

                let confirm_task = Task::perform(
                    kill_app_process_async(cleaner_arc.clone(), Some(emitter)),
                    move |res| match res {
                        Ok(()) => AppMessage::ScanApp(Ok(
                            Arc::try_unwrap(cleaner_arc).unwrap_or_else(|c| (*c).clone())
                        )),
                        Err(err) => AppMessage::ScanApp(Err(err.to_string())),
                    },
                );

                let status_task = channel
                    .stream()
                    .map(|s| {
                        Task::stream(s.map(|status_event| AppMessage::ShowStatus(status_event)))
                    })
                    .unwrap_or_else(Task::none);

                Task::batch(vec![confirm_task, status_task])
            }
        },

        AppMessage::ScanApp(cleaner) => {
            let channel = create_channels(100, ChannelKind::Mpsc);
            if let Ok(app_input) = cleaner {
                let emitter = channel.get_emitter();

                let scan_task =
                    Task::perform(scan_app_async(app_input, Some(emitter)), |res| match res {
                        Ok(cleaner) => AppMessage::UpdateCleaner(cleaner),
                        Err(err) => {
                            let event = status!(action: "Failed", message: err.to_string(),);
                            AppMessage::ShowStatus(event)
                        }
                    });

                let progress_task = channel
                    .stream()
                    .map(|s| Task::stream(s.map(AppMessage::ShowStatus)))
                    .unwrap_or_else(Task::none);

                return Task::batch(vec![scan_task, progress_task]);
            }
            Task::none()
        }

        AppMessage::ReScanApp => {
            // Check if the path is empty (meaning no app has been selected yet)
            if state.app_path.as_os_str().is_empty() {
                let warning_status = status!(
                    action: "Warning",
                    message: "No application path found to re-scan.",
                );
                Task::done(AppMessage::ShowStatus(warning_status))
            } else {
                // Forward the app path back to the initialization logic
                let path_to_process = state.app_path.clone();
                Task::done(AppMessage::ProcessApp(path_to_process))
            }
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
                        let event = status!(action: "Failed", message: err.to_string(),);
                        AppMessage::ShowStatus(event)
                    }
                });

                tasks.push(task);
            }

            let founded = state.cleaner.all_entries_enumerate().len();
            let event = status!(
                action: "Completed",
                message: format!("{} items found", founded),
            );
            tasks.push(Task::done(AppMessage::ShowStatus(event)));
            Task::batch(tasks)
        }

        AppMessage::IconLoaded(backend_cache) => {
            // Consumes the backend cache map, loads into state.icon_cache, and drops the backend instantly
            state.consume_backend_icon(backend_cache);

            Task::none()
        }

        AppMessage::OpenSelectedPath(index) => {
            state.selected_file = Some(index);
            debug!("Clicked path: {:?}", index);

            let entries = state.cleaner.all_entries_enumerate();

            if let Some((_i, entry)) = entries.get(index) {
                let path = entry.as_path().to_path_buf();
                return Task::perform(open_loc_async(path), |_| AppMessage::NoOperations);
            }
            Task::none()
        }

        AppMessage::ExportBomFilesLoc => Task::perform(set_output_path(), |res| match res {
            Ok(path) => AppMessage::ExportBomFiles(Ok(path)),
            Err(e) => {
                let event = status!("{}", e.to_string());
                AppMessage::ShowStatus(event)
            }
        }),

        AppMessage::ExportBomFiles(result) => match result {
            Ok(path) => {
                let cleaner = state.cleaner.clone();
                Task::perform(save_bom_logs_async(cleaner, path), |res| match res {
                    Ok(()) => {
                        let event = status!("Bom file saved");
                        AppMessage::ShowStatus(event)
                    }
                    Err(err) => {
                        let event = status!("{}", err.to_string());
                        AppMessage::ShowStatus(event)
                    }
                })
            }
            Err(e) => {
                let event = status!("{}", e);
                Task::done(AppMessage::ShowStatus(event))
            }
        },

        AppMessage::MoveToTrash => {
            let cleaner = state.cleaner.clone();
            Task::perform(trash_app_async(cleaner), |res| match res {
                Ok(remaining_entry) => AppMessage::UpdateEntryFiles(Ok(remaining_entry)),
                Err(err) => AppMessage::UpdateEntryFiles(Err(err.to_string())),
            })
        }

        AppMessage::UpdateEntryFiles(result) => {
            match result {
                Ok(remaining_entries) => {
                    if remaining_entries.is_empty() {
                        state.reset();
                        state.show_status = status!("App moved to Trash");
                    } else {
                        state
                            .cleaner
                            .replace_remaining_entries(remaining_entries.clone());

                        let mut missing = 0usize;
                        let mut grouped: HashMap<(TrashStatus, String), usize> = HashMap::new();

                        for e in &remaining_entries {
                            let path = e.entry().as_path();

                            if !path.exists() {
                                missing += 1;
                                continue;
                            }

                            let reason = e.reason().unwrap_or("Unknown").to_string();

                            *grouped.entry((e.status(), reason)).or_insert(0) += 1;
                        }

                        let mut items: Vec<((TrashStatus, String), usize)> =
                            grouped.into_iter().collect();

                        items.sort_by_key(|((status, _reason), _count)| match status {
                            TrashStatus::Failed => 0,
                            TrashStatus::Skipped => 1,
                        });

                        let mut report = Vec::new();

                        report.push(format!("{} item not moved", remaining_entries.len()));

                        if missing > 0 {
                            report.push(format!("{} items path not exist", missing));
                        }

                        for ((status, reason), count) in items {
                            let label = match status {
                                TrashStatus::Failed => "failed",
                                TrashStatus::Skipped => "skipped",
                            };

                            report.push(format!("{} items {}: {}", count, label, reason));
                        }
                        state.show_status = status!("{}", report.join("\n"));
                    }
                }

                Err(err_msg) => {
                    state.show_status = status!(
                        action: "Failed:",
                        message: err_msg,
                    );
                }
            }

            Task::none()
        }

        AppMessage::ClearList => {
            state.reset();
            Task::none()
        }

        AppMessage::ShowStatus(new_status) => {
            state.show_status = new_status;
            Task::none()
        }

        AppMessage::NoOperations => Task::none(),
    }
}

pub fn subscription(_state: &AppState) -> Subscription<AppMessage> {
    // iced::event::listen().map(|event| match event {
    //     Event::Window(window::Event::FileDropped(path)) => AppMessage::DropApp(path),
    //     _ => AppMessage::NoOperations,
    // })
    let file_drop_sub = iced::event::listen().map(|event| match event {
        iced::Event::Window(iced::window::Event::FileDropped(path)) => AppMessage::DropApp(path),
        _ => AppMessage::NoOperations,
    });

    // let status = if let Some(stream) = simple_status::stream() {
    //     Subscription::run(stream.map(AppMessage::ShowStatus))
    // } else {
    //     Subscription::none()
    // };

    Subscription::batch(vec![file_drop_sub])
}
