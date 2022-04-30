use std::fmt::{Display, Formatter};
use iced::*;
use iced::button::StyleSheet;
use crate::icon::*;
use crate::conf::{MAX_FLOOR, MIN_FLOOR, TFloor};
use crate::message::UiMessage;
use crate::style::{ActiveFloorBtnStyle, ActiveFloorTxtStyle};

#[derive(Default)]
pub struct FloorBtnState {
    pub floor: TFloor,
    // 判定按钮双击
    pub last_pressed: Option<std::time::Instant>,
    pub is_active: bool,
    pub elevator_no: usize,
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

#[derive(Ord, PartialOrd, Eq, PartialEq, Copy, Clone)]
pub enum Direction {
    Up,
    Down,
}

impl Default for Direction {
    fn default() -> Self {
        Direction::Up
    }
}

impl Display for Direction {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", match self {
            Direction::Up => { "上" }
            Direction::Down => { "下" }
        })
    }
}

#[derive(Default, Copy, Clone, Ord, PartialOrd, Eq, PartialEq)]
pub struct WaitFloorTxtState {
    pub floor: TFloor,
    pub direction: Direction,
}

impl WaitFloorTxtState {
    pub fn floor_view(&mut self) -> Element<UiMessage> {
        Container::new(
            Row::with_children(vec![
                Text::new(format!("{}", self.floor))
                    .horizontal_alignment(HorizontalAlignment::Center).into(),
                match self.direction {
                    Direction::Up => up_icon().into(),
                    Direction::Down => down_icon().into(),
                }
            ])
        ).width(Length::Units(50))
            .align_x(Align::Center)
            .style(ActiveFloorTxtStyle::default())
            .into()
    }
}