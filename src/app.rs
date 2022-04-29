#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::collections::{BTreeMap, HashMap};
use crate::message::*;
use iced::*;
use iced::futures::SinkExt;
use crate::conf::{MAX_ELEVATOR_NUM, MAX_FLOOR, MIN_FLOOR};
use crate::floor_btn::FloorBtnState;


struct ElevatorApp {
    floor: i16,
    slider_state: slider::State,
    up_btn_state: button::State,
    plus_btn_state: button::State,
    subtract_btn_state: button::State,
    down_btn_state: button::State,
    elevator_btns: BTreeMap<usize, Vec<FloorBtnState>>,
}

impl Default for ElevatorApp {
    fn default() -> Self {
        let mut hp = BTreeMap::new();
        for no in 1..=MAX_ELEVATOR_NUM {
            hp.insert(no, (MIN_FLOOR..=MAX_FLOOR)
                .into_iter()
                .filter(|o| *o != 0)
                .map(|o| FloorBtnState {
                    floor: o,
                    elevator_no: no as u8,
                    state: button::State::default(),
                }).collect());
        }
        Self {
            floor: 1,
            slider_state: Default::default(),
            up_btn_state: Default::default(),
            plus_btn_state: Default::default(),
            subtract_btn_state: Default::default(),
            down_btn_state: Default::default(),
            elevator_btns: hp,
        }
    }
}

pub fn run_window() {
    // let mut settings = Settings::with_flags(AppFlags::new(exe_path));
    let mut settings = Settings::default();
    settings.window.resizable = true; // 不能重新缩放窗口
    settings.default_font = Some(include_bytes!(
        "../assets/font/ZiTiGuanJiaFangSongTi-2.ttf"
    ));
    ElevatorApp::run(settings).unwrap();
}

const BTN_PER_ROW: i16 = 15;

impl ElevatorApp {
    const fn floor_rows() -> i16 {
        let rows = (MAX_FLOOR - MIN_FLOOR) / BTN_PER_ROW;
        let m = (MAX_FLOOR - MIN_FLOOR) % BTN_PER_ROW;
        if m == 0 {
            rows
        } else {
            rows + 1
        }
    }
}

impl Application for ElevatorApp {
    type Executor = executor::Default;
    type Message = UiMessage;
    type Flags = ();

    fn new(_: Self::Flags) -> (Self, Command<Self::Message>) {
        (Self::default(), Command::none())
    }

    fn title(&self) -> String {
        format!("多路电梯调度器")
    }

    fn update(&mut self, message: Self::Message, clipboard: &mut Clipboard) -> Command<Self::Message> {
        match message {
            UiMessage::SliderChange(floor) => {
                if floor != 0 {
                    self.floor = floor;
                }
            }
            UiMessage::ClickedBtnPlus =>{
                if self.floor == -1 {
                    self.floor = 1
                }else{
                    self.floor += 1
                }
            }
            UiMessage::ClickedBtnSubtract =>{
                if self.floor == 1 {
                    self.floor = -1
                }else{
                    self.floor -= 1
                }
            }
            UiMessage::ClickedBtnUp => {
                println!("{:?}", message);
            }
            UiMessage::ClickedBtnDown => {
                println!("{:?}", message);
            }
            UiMessage::ClickedBtnFloor(no, floor) =>{
                println!("电梯#{},按了{}层", no, floor);

            }
            _ => {}
        }
        Command::none()
    }

    fn view(&mut self) -> Element<'_, Self::Message> {
        let mut subs = vec![];
        let slider = Slider::new(
            &mut self.slider_state,
            (MIN_FLOOR..=MAX_FLOOR),
            self.floor,
            UiMessage::SliderChange)
            .width(Length::FillPortion(2))
            .into();
        let floor = Text::new(&format!("{}", self.floor))
            .width(Length::Units(30))
            .into();
        let e = Text::new("所在楼层: ")
            .into();

        let up_btn_row = Row::with_children(vec![
            Button::new(&mut self.up_btn_state, Text::new("上"))
                .on_press(UiMessage::ClickedBtnUp)
                .into(),
            Space::with_width(Length::Units(10)).into(),
            Button::new(&mut self.down_btn_state, Text::new("下"))
                .on_press(UiMessage::ClickedBtnDown)
                .into(),
        ]).width(Length::FillPortion(1))
            .spacing(10).into();

        subs.push(Button::new(&mut self.subtract_btn_state, Text::new("-"))
                      .width(Length::Units(20))
                      .on_press(UiMessage::ClickedBtnSubtract)
                      .into(),);
        subs.push(Space::with_width(Length::Units(5)).into());
        subs.push(slider);
        subs.push(Space::with_width(Length::Units(5)).into());
        subs.push(Button::new(&mut self.plus_btn_state, Text::new("+"))
                      .width(Length::Units(20))
                      .on_press(UiMessage::ClickedBtnPlus)
                      .into(),);
        subs.push(Space::with_width(Length::Units(20)).into());
        subs.push(e);
        subs.push(Space::with_width(Length::Units(4)).into());
        subs.push(floor);
        subs.push(Space::with_width(Length::Units(20)).into());
        subs.push(up_btn_row);
        subs.push(Space::with_width(Length::FillPortion(2)).into());
        let mut rows = vec![
            Row::with_children(subs)
                .padding(20)
                .width(Length::Fill)
                .align_items(Align::Center).into(),
        ];
        let new_rows = self.elevator_btns
            .iter_mut()
            .fold( rows, |mut _rows, (elevator_no, floors) |{
                let status = Column::with_children(vec![
                    Row::with_children(vec![
                        Text::new("电梯编号:").width(Length::FillPortion(1)).into(),
                        Text::new(format!("{}", elevator_no)).width(Length::FillPortion(1)).into(),
                    ]).spacing(10).padding(4).into(),
                    Row::with_children(vec![
                        Text::new("运行状态:").width(Length::FillPortion(1)).into(),
                        Text::new(format!("{}运行中", elevator_no)).width(Length::FillPortion(1)).into(),
                    ]).spacing(10).padding(4).into(),
                    Row::with_children(vec![
                        Text::new("所在楼层:").width(Length::FillPortion(1)).into(),
                        Text::new(format!("{}", elevator_no)).width(Length::FillPortion(1)).into(),
                    ]).spacing(10).padding(4).into(),
                    Row::with_children(vec![
                        Text::new("人数:").width(Length::FillPortion(1)).into(),
                        Text::new(format!("{}", 0)).width(Length::FillPortion(1)).into(),
                    ]).spacing(10).padding(4).into(),
                ]).width(Length::FillPortion(1))
                    .into();
                let mut row_floors = Vec::with_capacity(Self::floor_rows() as usize);
                let mut tmp_row = Vec::with_capacity(BTN_PER_ROW as usize);
                let mut i = 0;
                for f in floors
                    .iter_mut()
                    .enumerate()
                    .fold(vec![],
                          |mut row, (ix, floor)| {
                              row.push(floor.floor_view());
                              row
                          }){
                    if i % BTN_PER_ROW == 0 {
                        if !tmp_row.is_empty(){
                            row_floors.push(Row::with_children(
                                tmp_row.drain(0..tmp_row.len()).collect())
                                .spacing(10)
                                .padding(4)
                                .into()
                            );
                        }
                    }else{
                        tmp_row.push(f);
                    }
                    i += 1;
                }
                if !tmp_row.is_empty(){
                    row_floors.push(Row::with_children(
                        tmp_row.drain(0..tmp_row.len()).collect())
                        .spacing(10)
                        .padding(4)
                        .into()
                    );
                }
                let elevator_floors = Column::with_children(row_floors)
                    .width(Length::FillPortion(3))
                    .into();
                _rows.push(Row::with_children(vec![
                    status,
                    elevator_floors
                ]).into());
                _rows
        });
        Column::with_children(new_rows)
            .spacing(30)
            .height(Length::Fill)
            .height(Length::Fill).into()
    }
}
