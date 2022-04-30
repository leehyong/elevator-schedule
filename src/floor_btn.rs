use iced::*;
use crate::conf::MIN_FLOOR;
use crate::message::UiMessage;
use crate::style::ActiveFloorBtnStyle;

#[derive(Default)]
pub struct FloorBtnState {
    pub floor: i16,
    // 判定按钮双击
    pub last_pressed: Option<std::time::Instant>,
    pub is_active: bool,
    pub elevator_no: u8,
    pub state: button::State,
}


impl FloorBtnState {
    pub fn floor_view(&mut self) -> Element<UiMessage> {
        let mut btn = Button::new(&mut self.state,
                                  Text::new(format!("{}", self.floor))
                                      .horizontal_alignment(HorizontalAlignment::Center),
        )
            .width(Length::Units(30))
            .on_press(
                UiMessage::ClickedBtnFloor(self.elevator_no, self.floor)
            );
        if self.last_pressed.is_some() && self.is_active {
            btn = btn.style(ActiveFloorBtnStyle::default());
        }
        btn.into()
    }
}

#[derive(Default)]
pub struct WaitFloorBtnState {
    pub floor: i16,
    pub state: button::State,
}

impl WaitFloorBtnState {
    pub fn floor_view(&mut self) -> Element<UiMessage> {
        Button::new(
            &mut self.state,
            Text::new(format!("{}", self.floor))
                .horizontal_alignment(HorizontalAlignment::Center),
        ).style(ActiveFloorBtnStyle::default())
            .width(Length::Units(30))
            .into()
    }
}