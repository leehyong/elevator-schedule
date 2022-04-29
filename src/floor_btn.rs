use iced::*;
use crate::conf::MIN_FLOOR;
use crate::message::UiMessage;

pub struct FloorBtnState {
    pub floor: i16,
    pub elevator_no: u8,
    pub state: button::State,
}


impl FloorBtnState {
   pub fn floor_view(&mut self) -> Element<UiMessage> {
        Button::new(&mut self.state,
                    Text::new(format!("{}", self.floor))
                        .horizontal_alignment(HorizontalAlignment::Center),
        )
            .width(Length::Units(30))
            .on_press(
            UiMessage::ClickedBtnFloor(self.elevator_no, self.floor)
        )
            .into()
    }
}