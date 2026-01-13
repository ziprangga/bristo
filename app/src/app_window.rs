use iced::widget::Column;
use iced::widget::Stack;
use iced::{
    Background, Border, Color, Padding, alignment,
    widget::{Container, Row, Text, container, row, text},
};
use iced::{Element, Length};

use crate::app_state::{AppMessage, AppState};
use widget::button_style::{
    CustomButton, blank_border_style, blank_btn_style, custom_btn_rounded_style, danger_style,
};
use widget::drop_file::DropFile;
use widget::list_view::{HeaderContent, HeaderWidget, ListView, RowContent, WidgetContent};

pub fn view(state: &AppState) -> Element<'_, AppMessage> {
    let drop_zone: Element<AppMessage> = DropFile::widget(|| {
        CustomButton::new("Drag & Drop App here or click to browse")
            .text_size(20)
            .text_color(Color::TRANSPARENT)
            .height(Length::Fixed(200.0))
            .width(Length::Fixed(200.0))
            .style(blank_border_style)
            .on_press(AppMessage::InputFile)
            .view()
    })
    .view();

    let entries = state.cleaner.app_data.all_associate_entries_enumerate();

    let has_real_items = entries
        .iter()
        .any(|(_, (path, _))| !path.as_os_str().is_empty());

    let items = entries
        .into_iter()
        .map(|(i, (path, label))| {
            // ===============
            let display_path = if let Ok(home) = std::env::var("HOME") {
                if let Ok(stripped) = path.strip_prefix(&home) {
                    format!("~/{}", stripped.to_string_lossy())
                } else {
                    path.to_string_lossy().to_string()
                }
            } else {
                path.to_string_lossy().to_string()
            };

            // ===============
            RowContent::Widget(WidgetContent::new(move |_selected| {
                let style = blank_btn_style;
                row![
                    CustomButton::new(label.clone())
                        .text_size(12)
                        .text_align_x(alignment::Horizontal::Left)
                        .width(Length::Fill)
                        .on_press(AppMessage::OpenSelectedPath(i))
                        .style(style)
                        .view(),
                    // Text::new(path.to_string_lossy().to_string())
                    Text::new(display_path.clone())
                        .size(12)
                        .color(Color::from_rgb8(3, 161, 252))
                        .width(Length::Fill)
                ]
                .into()
            }))
        })
        .collect::<Vec<_>>();

    let headers = vec![HeaderContent::Widget(HeaderWidget::new(|_selected| {
        Row::new()
            .spacing(10)
            .push(
                Text::new("Name")
                    .size(12)
                    .color(Color::WHITE)
                    .width(Length::Fill),
            )
            .push(
                Text::new("Path")
                    .size(12)
                    .color(Color::WHITE)
                    .width(Length::Fill),
            )
            .into()
    }))];
    let list_view = ListView::new(items)
        .headers(headers)
        .row_selected(state.selected_file)
        .row_style(|i, _theme| {
            let color = if i % 2 == 0 {
                Color::from_rgb8(32, 36, 42)
            } else {
                Color::from_rgb8(28, 32, 38)
            };

            container::Style {
                background: Some(color.into()),
                border: Border {
                    color: Color::TRANSPARENT,
                    width: 1.0,
                    radius: 5.0.into(),
                },
                ..Default::default()
            }
        })
        .view();

    let center_view = if !has_real_items
        && !state.show_modal_ask.show_modal
        && state.input_file.as_os_str().is_empty()
    {
        drop_zone
    } else {
        list_view
    };

    let output_display = if !state.output_file.as_os_str().is_empty() {
        &state.output_file.display().to_string()
    } else {
        &"Save Bom log file ( Default to Desktop )".to_string()
    };

    let button_path = if !state.input_file.as_os_str().is_empty() {
        Container::new(
            CustomButton::new(output_display)
                .text_align_x(alignment::Horizontal::Left)
                .text_align_y(alignment::Vertical::Center)
                .width(Length::Fill)
                .style(blank_btn_style)
                .on_press(AppMessage::BrowseOutput)
                .view(),
        )
        .style(|_| container::Style {
            background: Some(Background::Color(Color {
                r: 1.0,
                g: 1.0,
                b: 1.0,
                a: 0.06,
            })),
            border: Border {
                color: Color {
                    r: 1.0,
                    g: 1.0,
                    b: 1.0,
                    a: 0.06,
                },
                width: 0.3,
                radius: 5.0.into(),
            },
            ..Default::default()
        })
    } else {
        Container::new(
            CustomButton::new(output_display)
                .text_align_x(alignment::Horizontal::Left)
                .text_align_y(alignment::Vertical::Center)
                .width(Length::Fill)
                .style(blank_btn_style)
                .view(),
        )
        .style(|_| container::Style {
            background: Some(Background::Color(Color {
                r: 1.0,
                g: 1.0,
                b: 1.0,
                a: 0.06,
            })),
            border: Border {
                color: Color {
                    r: 1.0,
                    g: 1.0,
                    b: 1.0,
                    a: 0.06,
                },
                width: 0.3,
                radius: 5.0.into(),
            },
            ..Default::default()
        })
    };

    let button_export = if !state.input_file.as_os_str().is_empty() {
        Container::new(
            CustomButton::new("Export Bom Logs")
                .text_align_x(alignment::Horizontal::Left)
                .text_align_y(alignment::Vertical::Center)
                .width(Length::Shrink)
                .style(custom_btn_rounded_style)
                .on_press(AppMessage::ExportFile)
                .view(),
        )
    } else {
        Container::new(
            CustomButton::new("Export Bom Logs")
                .text_align_x(alignment::Horizontal::Left)
                .text_align_y(alignment::Vertical::Center)
                .width(Length::Shrink)
                .style(custom_btn_rounded_style)
                .view(),
        )
    };

    let bom_output = Container::new(row![button_path, button_export].spacing(5))
        .padding([3, 20])
        .align_y(alignment::Vertical::Center);

    let button_clear_list = Container::new(
        CustomButton::new("Clear list")
            .text_align_y(alignment::Vertical::Center)
            .text_align_x(alignment::Horizontal::Center)
            .width(Length::Fill)
            .style(blank_border_style)
            .on_press(AppMessage::ClearList)
            .view(),
    )
    .width(Length::Shrink)
    .padding([3, 20])
    .align_y(alignment::Vertical::Center);

    let status_msg = state
        .status
        .view(|message_status| {
            if let Some(message) = message_status {
                container(
                    text(message)
                        .size(12)
                        .width(Length::Fill)
                        .center()
                        .style(|_| text::Style {
                            color: Some(Color::from_rgb8(200, 200, 200)),
                        }),
                )
                .width(Length::Fill)
                .align_x(alignment::Horizontal::Center)
                .align_y(alignment::Vertical::Center)
                .into()
            } else {
                container(text("").width(Length::Fill).center())
                    .width(Length::Fill)
                    .align_x(alignment::Horizontal::Center)
                    .align_y(alignment::Vertical::Center)
                    .into()
            }
        })
        .map(AppMessage::Status);

    let button_delete = Container::new(
        CustomButton::new("Move to Trash")
            .text_align_x(alignment::Horizontal::Center)
            .text_align_y(alignment::Vertical::Center)
            .width(Length::Fill)
            .on_press(AppMessage::TrashApp)
            .style(danger_style)
            .view(),
    )
    .width(Length::Shrink)
    .align_x(alignment::Horizontal::Center)
    .align_y(alignment::Vertical::Center);

    // ==================== modal view ====================
    let modal = state
        .show_modal_ask
        .view()
        .map(|e| e.map(AppMessage::ModalAsk));

    // ====================main layout========================
    let top = Container::new(
        Row::new()
            .push(bom_output)
            .push(button_clear_list)
            .width(Length::Fill)
            .height(Length::Shrink),
    );

    let center = Container::new(
        Column::new()
            .push(center_view)
            .width(Length::Fill)
            .height(Length::Fill)
            .spacing(10),
    )
    .width(Length::Fill)
    .height(Length::Fill)
    .padding(10)
    .style(|_theme| container::Style {
        background: None,
        text_color: None,
        border: Border {
            color: Color::from_rgb8(100, 100, 100),
            width: 2.0,
            radius: 8.0.into(),
        },
        snap: false,
        shadow: Default::default(),
    });

    let bottom = Container::new(
        row![status_msg, button_delete,]
            .align_y(alignment::Vertical::Center)
            .spacing(5),
    )
    .width(Length::Fill)
    .align_x(alignment::Horizontal::Center)
    .align_y(alignment::Vertical::Center)
    .padding(Padding {
        top: 6.0,
        bottom: 6.0,
        left: 12.0,
        right: 12.0,
    });

    let content: Element<_> = Column::new()
        .push(top)
        .push(center)
        .push(bottom)
        .width(Length::Fill)
        .height(Length::Fill)
        .spacing(10)
        .padding(10)
        .into();

    // ==================== stack with modal ====================
    if let Some(modal) = modal {
        Stack::new().push(content).push(modal).into()
    } else {
        content
    }
}
