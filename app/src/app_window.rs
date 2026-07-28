use iced::widget::Column;
use iced::widget::Stack;
use iced::widget::text::Wrapping;
use iced::{
    Border, Color, Padding, alignment,
    widget::{Container, Row, Space, Text, button, container, text},
};
use iced::{Element, Length};

use crate::app_state::{AppMessage, AppState};
use crate::ui_element::{ButtonThemeStyle, CustomStyle};
// use crate::app_tree_view::TreeView;
use crate::ui_element::DropFile;
use crate::ui_element::{Cell, ContentCell, HeaderCell, Table};

pub fn view(state: &AppState) -> Element<'_, AppMessage> {
    let text_box = text("Drag & Drop App here or click to browse")
        .size(20)
        .align_x(alignment::Horizontal::Center)
        .align_y(alignment::Vertical::Center);

    let button_input = button(text_box)
        .width(Length::Fixed(200.0))
        .height(Length::Fixed(200.0))
        .on_press(AppMessage::AppPath)
        .custom_style(ButtonThemeStyle::BlankBorder);

    let drop_zone: Element<AppMessage> = DropFile::new()
        .content_element(button_input)
        .content_layout(|c| {
            c.width(Length::Fill)
                .height(Length::Fill)
                .align_x(alignment::Horizontal::Center)
                .align_y(alignment::Vertical::Center)
                .padding(5)
                .style(|_| container::Style {
                    background: None,
                    text_color: None,
                    border: Border {
                        color: Color {
                            r: 1.0,
                            g: 1.0,
                            b: 1.0,
                            a: 0.06,
                        },
                        width: 1.0,
                        radius: 8.0.into(),
                    },
                    snap: false,
                    shadow: Default::default(),
                })
        })
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
            let display_path = entry.to_string();

            let icon_element: iced::Element<_> = match state.get_cached_icon(&path) {
                Some(icon_handle) => iced::widget::image(icon_handle)
                    .width(iced::Length::Fixed(16.0))
                    .height(iced::Length::Fixed(16.0))
                    .into(),
                None => iced::widget::Space::new().width(16.0).into(),
            };

            let name_with_icon = Row::new()
                .push(icon_element)
                .push(
                    iced::widget::text(label.clone())
                        .size(12)
                        .wrapping(Wrapping::WordOrGlyph),
                )
                .spacing(8)
                .align_y(alignment::Vertical::Center);

            let cell_name_with_icon = Cell::new(
                button(name_with_icon)
                    .custom_style(ButtonThemeStyle::Blank)
                    .width(Length::Fill)
                    .on_press(AppMessage::OpenSelectedPath(i)),
            );

            let cell_path = Cell::new(
                Text::new(display_path.clone())
                    .size(12)
                    .width(Length::Fill)
                    .height(Length::Fill)
                    .align_x(alignment::Horizontal::Left)
                    .align_y(alignment::Vertical::Center),
            );

            // ===============
            ContentCell::new()
                .cell(cell_name_with_icon)
                .cell(cell_path)
                .width(Length::Fill)
                .padding(5)
                .style(|i, _id, _theme| {
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
        .padding(5)
        .width(Length::Fill)
        .build();

    let center_view = if !has_real_items {
        drop_zone
    } else {
        list_view
    };

    let button_export_bom_files_active = button(text("Export Bom Logs").size(12))
        .width(Length::Shrink)
        .on_press(AppMessage::ExportBomFilesLoc)
        .custom_style(ButtonThemeStyle::CustomRounded);

    let button_export_bom_files_disabled = button(text("Export Bom Logs").size(12))
        .width(Length::Shrink)
        .custom_style(ButtonThemeStyle::CustomRounded);

    let button_export_bom_files = if !state.app_path.as_os_str().is_empty()
        && !state
            .cleaner
            .as_app_profile()
            .as_app_log_receipt()
            .is_empty()
    {
        Container::new(button_export_bom_files_active)
    } else {
        Container::new(button_export_bom_files_disabled)
    };

    let button_clear_list = Container::new(
        button(text("Clear list").size(12))
            .width(Length::Fill)
            .custom_style(ButtonThemeStyle::BlankBorder)
            .on_press(AppMessage::ClearList),
    )
    .width(Length::Shrink);

    let button_re_scan = if !state.app_path.as_os_str().is_empty() {
        Container::new(
            button(text("Re Scan").size(12))
                .width(Length::Fill)
                .custom_style(ButtonThemeStyle::CustomRounded)
                .on_press(AppMessage::ReScanApp),
        )
        .width(Length::Shrink)
    } else {
        Container::new(
            button(text("Re Scan").size(12))
                .width(Length::Fill)
                .custom_style(ButtonThemeStyle::CustomRounded),
        )
        .width(Length::Shrink)
    };

    let status = Column::new().width(Length::Fill).spacing(10);

    let status = match &state.show_status.status_result() {
        Some(result) => status.push(
            Container::new(
                text(result.to_string())
                    .size(12)
                    .width(Length::Fill)
                    .center()
                    .style(|_| text::Style {
                        color: Some(Color::from_rgb8(255, 150, 0)),
                    }),
            )
            .width(Length::Fill)
            .align_x(alignment::Horizontal::Center)
            .align_y(alignment::Vertical::Center),
        ),
        None => status,
    };

    let status = match &state.show_status.status_event() {
        Some(event) => status.push(
            Container::new(
                text(event.to_string())
                    .size(12)
                    .width(Length::Fill)
                    .center()
                    .style(|_| text::Style {
                        color: Some(Color::from_rgb8(200, 200, 200)),
                    }),
            )
            .width(Length::Fill)
            .align_x(alignment::Horizontal::Center)
            .align_y(alignment::Vertical::Center),
        ),
        None => status,
    };

    let button_delete = Container::new(
        button(text("Move to Trash").size(12))
            .width(Length::Fill)
            .custom_style(ButtonThemeStyle::Danger)
            .on_press(AppMessage::MoveToTrash),
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
            .push(button_re_scan)
            .push(button_clear_list)
            .width(Length::Fill)
            .spacing(10)
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
        Row::new()
            .push(status)
            .push(button_delete)
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
