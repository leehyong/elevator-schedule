use std::fmt::{Display, Formatter};
use iced::*;
use iced::button::StyleSheet;
use crate::icon::*;
use crate::conf::{MAX_FLOOR, MIN_FLOOR, TFloor};
use crate::message::AppMessage;
use crate::style::{ActiveFloorBtnStyle, ActiveFloorTxtStyle};

#[derive(Default)]
pub struct FloorBtnState {
    pub floor: TFloor,
    // 判定按钮双击
    pub last_pressed: Option<std::time::Instant>,
    pub is_active: bool,
    pub can_click: bool,
    pub elevator_no: usize,
    pub state: button::State,
}


impl FloorBtnState {
    pub fn floor_view(&mut self) -> Element<AppMessage> {
        let mut txt = Text::new(format!("{}", self.floor))
            .horizontal_alignment(HorizontalAlignment::Center);
        if self.can_click {
            txt = txt.color(Color::from_rgb8(255, 63, 51 ));
        }else {
            txt = txt.color(iced::Color::BLACK);
        }
        let mut btn = Button::new(
            &mut self.state,
            txt,
        ).width(Length::Units(30));
        if self.can_click {
            btn = btn.on_press(
                AppMessage::ClickedBtnFloor(self.elevator_no, self.floor)
            );
        }
        if self.is_active {
            btn = btn.style(ActiveFloorBtnStyle::default());
        }
        btn.into()
    }
}

#[derive(Ord, PartialOrd, Eq, PartialEq, Copy, Clone, Debug)]
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
    pub is_scheduled: bool,
}


impl WaitFloorTxtState {
    fn my_color(&self) -> Color {
        if self.is_scheduled {
            match self.direction {
                Direction::Up => Color::from_rgb8(255, 0, 0),
                Direction::Down => Color::from_rgb8(0, 0, 255),
            }
        } else {
            Color::WHITE
        }
    }

    pub fn floor_view(&mut self) -> Element<AppMessage> {
        let color = self.my_color();
        Container::new(
            Row::with_children(vec![
                Text::new(format!("{}", self.floor))
                    .color(color)
                    .horizontal_alignment(HorizontalAlignment::Center).into(),
                match self.direction {
                    Direction::Up => up_icon().color(color).into(),
                    Direction::Down => down_icon().color(color).into(),
                },
            ])
        ).width(Length::Units(50))
            .align_x(Align::Center)
            .style(ActiveFloorTxtStyle::default())
            .into()
    }
}