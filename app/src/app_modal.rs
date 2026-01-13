use iced::Element;
use iced::widget::mouse_area;
use iced::widget::opaque;
use iced::widget::{Column, Container, Row, Text, container};
use iced::{Background, Border, Color, Length, alignment};

use widget::button_style::{CustomButton, blank_border_style, danger_style};

#[derive(Clone, Default)]
pub struct ModalAsk {
    pub show_modal: bool,
    pub message: String,
}

#[derive(Debug, Clone)]
pub enum ModalAskMessage {
    ConfirmMsg(bool),
}

impl ModalAsk {
    /// Sets the modal message and shows it
    pub fn set_message(&mut self, msg: impl Into<String>) {
        self.message = msg.into();
        self.show_modal = true;
    }

    /// Hides the modal
    pub fn hide(&mut self) {
        self.show_modal = false;
        self.message.clear();
    }

    /// Updates based on the user answer
    pub fn update(&mut self, msg: ModalAskMessage) -> Option<bool> {
        match msg {
            ModalAskMessage::ConfirmMsg(answer) => {
                self.hide();
                Some(answer)
            }
        }
    }

    pub fn view(&self) -> Option<Element<'_, ModalAskMessage>> {
        if !self.show_modal {
            return None;
        }

        // modal text
        let modal_text = Text::new(&self.message)
            .size(14)
            .width(Length::Fill)
            .align_x(alignment::Horizontal::Center)
            .align_y(alignment::Vertical::Center);

        // Yes button
        let yes_btn = CustomButton::new("Yes")
            .text_size(12)
            .width(Length::Fill)
            .on_press(ModalAskMessage::ConfirmMsg(true))
            .style(danger_style)
            .view();

        // No button
        let no_btn = CustomButton::new("No")
            .text_size(12)
            .width(Length::Fill)
            .on_press(ModalAskMessage::ConfirmMsg(false))
            .style(blank_border_style)
            .view();

        // buttons row
        let buttons_row = Row::new().spacing(10).push(yes_btn).push(no_btn);

        // modal content column
        let modal_column = Column::new().push(modal_text).push(buttons_row).spacing(12);

        let modal_container = Container::new(modal_column)
            .padding(15)
            .width(300)
            .align_x(alignment::Horizontal::Center)
            .align_y(alignment::Vertical::Center)
            .style(|_theme| container::Style {
                background: Some(Background::Color(Color::from_rgb8(24, 28, 34))),
                border: Border {
                    color: Color::WHITE,
                    width: 1.0,
                    radius: 8.0.into(),
                },
                ..Default::default()
            });

        // full modal container
        let modal_overlay = Container::new(modal_container)
            .width(Length::Fill)
            .height(Length::Fill)
            .align_x(alignment::Horizontal::Center)
            .align_y(alignment::Vertical::Center)
            .style(|_| iced::widget::container::Style {
                background: Some(Background::Color(Color {
                    r: 0.0,
                    g: 0.0,
                    b: 0.0,
                    a: 0.5,
                })),
                ..Default::default()
            });

        let mouse_area_modal =
            mouse_area(iced::widget::center(opaque(modal_overlay)).style(|_theme| {
                container::Style {
                    background: Some(
                        Color {
                            a: 0.5,
                            ..Color::BLACK
                        }
                        .into(),
                    ),
                    ..container::Style::default()
                }
            }));

        Some(mouse_area_modal.into())
    }
}
