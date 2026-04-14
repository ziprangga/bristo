use crate::app_modal::ModalAskMessage;
use crate::app_state::{AppMessage, AppState};
use crate::app_task::kill_app_process_async;
use crate::app_task::save_bom_logs_async;
use crate::app_task::scan_app_async;
use crate::app_task::set_input_path;
use crate::app_task::set_output_path;
use crate::app_task::trash_app_async;
use crate::app_task::{add_app, open_loc_async};
use iced::{Event, Subscription, Task, futures::StreamExt, window};
use mini_logger::debug;
use simple_status::status;
use std::collections::HashMap;
use std::path::Path;
use std::path::PathBuf;
use std::sync::Arc;

pub fn update(state: &mut AppState, message: AppMessage) -> Task<AppMessage> {
    match message {
        AppMessage::DropFile(path) => {
            state.reset();
            state.input_file = path.clone();
            let add_app = {
                let channel = state.channel.clone();
                let input_file = state.input_file.clone();
                Task::perform(
                    async move {
                        let emitter = channel.get_emitter();
                        let result = add_app(input_file, emitter).await;
                        match result {
                            Ok(cleaner) => AppMessage::ConfirmKill(Ok(cleaner)),
                            Err(err) => {
                                let failure_status =
                                    status!(stage: "Failed", message: err.to_string(),);
                                AppMessage::ShowStatus(failure_status)
                            }
                        }
                    },
                    |msg| msg,
                )
            };

            let status_task = state
                .channel
                .stream()
                .map(|s| Task::stream(s.map(AppMessage::ShowStatus)))
                .unwrap_or_else(Task::none);

            Task::batch(vec![add_app, status_task])
        }

        AppMessage::InputFile => {
            state.reset();

            Task::perform(set_input_path(), |res| match res {
                Ok(path) => AppMessage::DropFile(path.to_path_buf()),
                Err(e) => {
                    let event = status!("{}", e.to_string());
                    AppMessage::ShowStatus(event)
                }
            })
        }

        AppMessage::ConfirmKill(result) => {
            if let Ok(cleaner) = result {
                if !cleaner.app_data.app_process.is_empty() {
                    state.pending_cleaner = Some(cleaner);

                    // Set the modal message and show it
                    state.show_modal_ask.set_message(format!(
                        "The app '{}' is still running.\nDo you want to kill its running process?\nBe careful to save your work first before continuing.",
                        state.pending_cleaner.as_ref().unwrap().app_data.app.name
                    ));
                    Task::none()
                } else {
                    Task::done(AppMessage::ScanApp(Ok(cleaner)))
                }
            } else {
                Task::none()
            }
        }

        AppMessage::ModalAsk(msg) => match msg {
            ModalAskMessage::ConfirmMsg(answer) => {
                state
                    .show_modal_ask
                    .update(ModalAskMessage::ConfirmMsg(answer));

                let cleaner = state.pending_cleaner.take().unwrap();
                if !answer {
                    return Task::done(AppMessage::ScanApp(Ok(cleaner)));
                }

                let emitter = state.channel.get_emitter();
                let cleaner_arc = Arc::new(cleaner);

                let confirm_task = Task::perform(
                    kill_app_process_async(cleaner_arc.clone(), emitter),
                    move |res| match res {
                        Ok(()) => AppMessage::ScanApp(Ok(
                            Arc::try_unwrap(cleaner_arc).unwrap_or_else(|c| (*c).clone())
                        )),
                        Err(err) => AppMessage::ScanApp(Err(err.to_string())),
                    },
                );

                let status_task = state
                    .channel
                    .stream()
                    .map(|s| {
                        Task::stream(s.map(|status_event| AppMessage::ShowStatus(status_event)))
                    })
                    .unwrap_or_else(Task::none);

                Task::batch(vec![confirm_task, status_task])
            }
        },

        AppMessage::ScanApp(cleaner) => {
            if let Ok(app_input) = cleaner {
                let emitter = state.channel.get_emitter();

                let scan_task =
                    Task::perform(scan_app_async(app_input, emitter), |res| match res {
                        Ok(cleaner) => AppMessage::UpdateCleaner(cleaner),
                        Err(err) => {
                            let event = status!(stage: "Failed", message: err.to_string(),);
                            AppMessage::ShowStatus(event)
                        }
                    });

                let progress_task = state
                    .channel
                    .stream()
                    .map(|s| Task::stream(s.map(AppMessage::ShowStatus)))
                    .unwrap_or_else(Task::none);

                return Task::batch(vec![scan_task, progress_task]);
            }
            Task::none()
        }

        AppMessage::UpdateCleaner(cleaner) => {
            state.cleaner = cleaner;
            let founded = state
                .cleaner
                .app_data
                .all_associate_entries_enumerate()
                .len();
            let event = simple_status::status!(
                stage: "Completed",
                message: format!("{} items found", founded),
            );
            Task::done(AppMessage::ShowStatus(event))
        }

        AppMessage::OpenSelectedPath(index) => {
            state.selected_file = Some(index);
            debug!("Clicked path: {:?}", index);

            let entries = state.cleaner.app_data.all_associate_entries_enumerate();

            if let Some((_i, (path, _label))) = entries.get(index) {
                let path = path.clone();
                return Task::perform(open_loc_async(path), |_| AppMessage::NoOperations);
            }
            Task::none()
        }

        AppMessage::BrowseOutput => Task::perform(set_output_path(), |res| match res {
            Ok(path) => AppMessage::OutputFile(Ok(path)),
            Err(e) => {
                let event = status!("{}", e.to_string());
                AppMessage::ShowStatus(event)
            }
        }),

        AppMessage::OutputFile(result) => {
            match result {
                Ok(path) => {
                    state.output_file = (*path).clone();
                    state.show_status = status!("folder selected");
                }
                Err(e) => {
                    state.show_status = status!("{}", e);
                }
            }
            Task::none()
        }

        AppMessage::ExportFile => {
            let output_dir = if !state.output_file.as_os_str().is_empty() {
                state.output_file.clone()
            } else {
                let home = std::env::var("HOME").unwrap();
                Path::new(&home).join("Desktop")
            };
            let cleaner = state.cleaner.clone();
            Task::perform(save_bom_logs_async(cleaner, output_dir), |res| match res {
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

        AppMessage::TrashApp => {
            let cleaner = state.cleaner.clone();
            Task::perform(trash_app_async(cleaner), |res| match res {
                Ok(failed) => AppMessage::DeletedApp(Ok(failed)),
                Err(err) => AppMessage::DeletedApp(Err(err.to_string())),
            })
        }

        AppMessage::DeletedApp(result) => {
            match result {
                Ok(failed_paths) => {
                    if failed_paths.is_empty() {
                        state.reset();
                        state.show_status = status!("App moved to Trash");
                    } else {
                        let failed_clone = failed_paths.clone();
                        state.cleaner.app_data.associate_files.replace(
                            failed_paths
                                .into_iter()
                                .map(|(path, _reason)| {
                                    let label = path
                                        .file_name()
                                        .map(|n| n.to_string_lossy().to_string())
                                        .unwrap_or_else(|| path.to_string_lossy().to_string());
                                    (path, label)
                                })
                                .collect(),
                        );

                        // Build the message from the actual failed paths
                        // group by reason
                        let mut grouped_reason: HashMap<String, Vec<PathBuf>> = HashMap::new();

                        for (path, reason) in failed_clone {
                            grouped_reason.entry(reason).or_default().push(path);
                        }

                        // build short grouped report message
                        let report = grouped_reason
                            .iter()
                            .map(|(reason, paths)| {
                                format!("{} items failed: {}", paths.len(), reason)
                            })
                            .collect::<Vec<_>>()
                            .join("\n");

                        state.show_status = status!("{}", report)
                    }
                }
                Err(err_msg) => {
                    state.show_status = status!(
                        stage: "Failed:",
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
    iced::event::listen().map(|event| match event {
        Event::Window(window::Event::FileDropped(path)) => AppMessage::DropFile(path),
        _ => AppMessage::NoOperations,
    })
}
