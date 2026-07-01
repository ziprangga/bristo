use iced::widget::Column;
use iced::widget::Stack;
use iced::widget::text::Wrapping;
use iced::{
    Border, Color, Padding, alignment,
    widget::{Container, Row, Space, Text, container, row, text},
};
use iced::{Element, Length};

use crate::app_state::{AppMessage, AppState};
// use crate::app_tree_view::TreeView;
use widget::button_style::{ButtonStyle, CustomButton};
use widget::drop_file::DropFile;
use widget::table::{Cell, ContentCell, HeaderCell, Table};

// ========

// use cleaner::{get_default_folder_icon, ns_image_to_rgba_bytes};
// use iced::widget::image;

// ========

pub fn view(state: &AppState) -> Element<'_, AppMessage> {
    let drop_zone: Element<AppMessage> = DropFile::new(
        CustomButton::new()
            .content_element(
                iced::widget::text("Drag & Drop App here or click to browse")
                    .size(20)
                    .align_x(alignment::Horizontal::Center)
                    .align_y(alignment::Vertical::Center),
            )
            .width(Length::Fixed(200.0))
            .height(Length::Fixed(200.0))
            .style(ButtonStyle::BlankBorder)
            .on_press(AppMessage::AppPath)
            .build(),
    )
    .build();

    let entries = state.cleaner.all_entries_enumerate();
    // let tree_views = TreeView::from_enumerated_entries(state.cleaner.all_entries_enumerate());

    let has_real_items = entries
        .iter()
        .any(|(_, entry)| !entry.as_path().as_os_str().is_empty());

    let contents = entries
        .into_iter()
        .map(|(i, entry)| {
            let label = entry.as_name().to_string();
            let path = entry.as_path().to_path_buf();

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

            // ===================

            let icon_element: iced::Element<_> = match state.get_cached_icon(&path) {
                Some(icon_handle) => iced::widget::image(icon_handle)
                    .width(iced::Length::Fixed(16.0))
                    .height(iced::Length::Fixed(16.0))
                    .into(),
                None => iced::widget::Space::new().width(16.0).into(),
            };

            // ===============
            ContentCell::new()
                .cell(Cell::new(
                    CustomButton::new()
                        .content_element(
                            // iced::widget::text(label.clone())
                            //     .size(12)
                            //     .wrapping(Wrapping::WordOrGlyph),
                            iced::widget::Row::new()
                                .spacing(8)
                                .align_y(alignment::Vertical::Center)
                                .push(icon_element)
                                .push(
                                    iced::widget::text(label.clone())
                                        .size(12)
                                        .wrapping(Wrapping::WordOrGlyph),
                                ),
                        )
                        .content_layout(|c| c.align_x(alignment::Horizontal::Left))
                        .width(Length::Fill)
                        .on_press(AppMessage::OpenSelectedPath(i))
                        .style(ButtonStyle::Blank)
                        .build(),
                ))
                .cell(Cell::new(
                    Text::new(display_path.clone()).size(12).width(Length::Fill),
                ))
        })
        .collect::<Vec<ContentCell<AppMessage>>>();

    let headers = HeaderCell::new()
        .cell(Cell::new(
            Text::new("Name")
                .size(12)
                .color(Color::WHITE)
                .width(Length::Fill),
        ))
        .cell(Cell::new(
            Text::new("Path")
                .size(12)
                .color(Color::WHITE)
                .width(Length::Fill),
        ));

    let list_view = Table::new()
        .header(headers)
        .contents(contents)
        .content_selected(state.selected_file)
        .content_style(|i, _theme| {
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
        .build();

    let center_view = if !has_real_items {
        drop_zone
    } else {
        list_view
    };

    let button_export_bom_files = if !state.app_path.as_os_str().is_empty()
        && !state
            .cleaner
            .as_app_profile()
            .as_app_log_receipt()
            .is_empty()
    {
        Container::new(
            CustomButton::new()
                .content_element(iced::widget::text("Export Bom Logs").size(12))
                .content_layout(|c| {
                    c.align_x(alignment::Horizontal::Left)
                        .align_y(alignment::Vertical::Center)
                })
                .width(Length::Shrink)
                .style(ButtonStyle::CustomRounded)
                .on_press(AppMessage::ExportBomFilesLoc)
                .build(),
        )
    } else {
        Container::new(
            CustomButton::new()
                .content_element(iced::widget::text("Export Bom Logs").size(12))
                .content_layout(|c| {
                    c.align_x(alignment::Horizontal::Left)
                        .align_y(alignment::Vertical::Center)
                })
                .width(Length::Shrink)
                .style(ButtonStyle::CustomRounded)
                .build(),
        )
    };

    let button_clear_list = Container::new(
        CustomButton::new()
            .content_element(iced::widget::text("Clear list").size(12))
            .content_layout(|c| {
                c.align_x(alignment::Horizontal::Center)
                    .align_y(alignment::Vertical::Center)
            })
            .width(Length::Fill)
            .style(ButtonStyle::BlankBorder)
            .on_press(AppMessage::ClearList)
            .build(),
    )
    .width(Length::Shrink);

    let status_msg = Container::new(
        text(state.show_status.to_string())
            .size(12)
            .width(Length::Fill)
            .center()
            .style(|_| text::Style {
                color: Some(Color::from_rgb8(200, 200, 200)),
            }),
    )
    .width(Length::Fill)
    .align_x(alignment::Horizontal::Center)
    .align_y(alignment::Vertical::Center);

    let button_delete = Container::new(
        CustomButton::new()
            .content_element(iced::widget::text("Move to Trash").size(12))
            .content_layout(|c| {
                c.align_x(alignment::Horizontal::Center)
                    .align_y(alignment::Vertical::Center)
            })
            .width(Length::Fill)
            .style(ButtonStyle::Danger)
            .on_press(AppMessage::MoveToTrash)
            .build(),
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
            .push(button_export_bom_files)
            .push(Space::new().width(Length::Fill))
            .push(button_clear_list)
            .width(Length::Fill)
            .spacing(5)
            .padding([3, 20])
            .align_y(alignment::Vertical::Center)
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
