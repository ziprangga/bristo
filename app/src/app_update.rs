use crate::app_modal::ModalAskMessage;
use crate::app_state::{AppMessage, AppState};
use crate::app_task::find_app_process_async;
use crate::app_task::kill_app_process_async;
use crate::app_task::save_bom_logs_async;
use crate::app_task::scan_app_async;
use crate::app_task::set_input_path;
use crate::app_task::set_output_path;
use crate::app_task::trash_app_async;
use crate::app_task::{add_app, open_loc_async};
use cleaner::TrashStatus;
use iced::{Event, Subscription, Task, futures::StreamExt, window};
use mini_logger::debug;
use simple_status::status;
use std::path::Path;
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
                            Ok(cleaner) => AppMessage::FindProcs(Ok(cleaner)),
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

        AppMessage::FindProcs(result) => {
            if let Ok(cleaner) = result {
                let emitter = state.channel.get_emitter();

                let find_task =
                    Task::perform(find_app_process_async(cleaner, emitter), |res| match res {
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
                    });

                let progress_task = state
                    .channel
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
            let founded = state.cleaner.all_entries_enumerate().len();
            let event = simple_status::status!(
                stage: "Completed",
                message: format!("{} items found", founded),
            );
            Task::done(AppMessage::ShowStatus(event))
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
                Ok(remaining_entry) => AppMessage::DeletedApp(Ok(remaining_entry)),
                Err(err) => AppMessage::DeletedApp(Err(err.to_string())),
            })
        }

        AppMessage::DeletedApp(result) => {
            match result {
                Ok(remaining_entries) => {
                    if remaining_entries.is_empty() {
                        state.reset();
                        state.show_status = status!("App moved to Trash");
                    } else {
                        state
                            .cleaner
                            .replace_remaining_entries(remaining_entries.clone());

                        let failed_count = remaining_entries
                            .iter()
                            .filter(|e| e.status() == TrashStatus::Failed)
                            .count();

                        let skipped_count = remaining_entries
                            .iter()
                            .filter(|e| e.status() == TrashStatus::Skipped)
                            .count();

                        state.show_status =
                            status!("{} failed, {} skipped", failed_count, skipped_count);
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
