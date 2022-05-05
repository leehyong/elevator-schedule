#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::cmp::{max, min};
use std::collections::{BTreeMap, HashMap, LinkedList};
use std::option::Option::Some;
use std::time::{Duration, Instant};
use crate::message::*;
use iced::*;
use iced::futures::SinkExt;
use iced::window::Mode;
use rand::{Rng, thread_rng};
use crate::conf::{MAX_ELEVATOR_NUM, MAX_FLOOR, MAX_PERSON_CAPACITY, MIN_FLOOR, TFloor};
use crate::util::*;
use crate::floor_btn::{Direction, FloorBtnState, WaitFloorTxtState};
use crate::icon::*;
use tokio::sync::RwLock;
use std::sync::Arc;
use crate::lift::{Lift, LiftUpDownCost};
use crate::up_down_elevator_floor::*;
use crate::state::State;
use crate::state::State::{GoingDown, GoingUp};


struct ElevatorApp {
    floor: TFloor,
    tmp_floor: TFloor,
    slider_state: slider::State,
    up_btn_state: button::State,
    plus_btn_state: button::State,
    subtract_btn_state: button::State,
    down_btn_state: button::State,
    // 哪些楼层需要安排电梯去接人的
    wait_floors: LinkedList<WaitFloorTxtState>,
    lifts: Vec<Lift>,
}

impl Default for ElevatorApp {
    fn default() -> Self {
        let mut lifts = Vec::with_capacity(MAX_ELEVATOR_NUM);
        for no in 0..MAX_ELEVATOR_NUM {
            lifts.push(Lift::new(no));
        }
        Self {
            floor: 1,
            tmp_floor: 0,
            slider_state: Default::default(),
            up_btn_state: Default::default(),
            plus_btn_state: Default::default(),
            subtract_btn_state: Default::default(),
            down_btn_state: Default::default(),
            wait_floors: Default::default(),
            lifts,
        }
    }
}

pub fn run_window() {
    let mut settings = Settings::default();
    settings.window.resizable = true; // 不能重新缩放窗口
    settings.default_font = Some(include_bytes!(
        "../assets/font/ZiTiGuanJiaFangSongTi-2.ttf"
    ));
    ElevatorApp::run(settings).unwrap();
}

const BTN_PER_ROW: TFloor = 15;
const WAIT_FLOOR_PER_ROW: TFloor = 16;
const MAX_WAIT_FLOOR_ROW_NUM: TFloor = 2;
const MAX_WAIT_FLOOR_NUM: usize = (BTN_PER_ROW * MAX_WAIT_FLOOR_ROW_NUM) as usize;


impl ElevatorApp {
    const fn floor_rows() -> i32 {
        Self::calc_rows2(MAX_FLOOR - MIN_FLOOR, BTN_PER_ROW)
    }

    fn remove_wait_floor(wait_floors: &mut LinkedList<WaitFloorTxtState>, floor: TFloor, lift: &mut Lift)
    {
        let direction = lift.remove_floor(floor);
        if let Some(direct) = direction {
            // 删除正在等待的楼层
            loop {
                if let Some((idx, _)) = wait_floors
                    .iter()
                    .enumerate()
                    .find(|(_, wf)| wf.floor == floor && wf.direction == direct
                    ) {
                    println!("remove_wait_floor {}, 索引:{}", lift.to_string(), idx);
                    let mut after = wait_floors.split_off(idx);
                    after.pop_front(); // 删除首部元素， 再跟原来的 list 拼接起来
                    wait_floors.append(&mut after);
                } else {
                    break;
                }
            }
        }
    }

    fn new_up_down_elevator(&self, floor: TFloor, typ: FloorType) -> UpDownElevatorFloor {
        match typ {
            FloorType::PersonUp | FloorType::PersonDown => UpDownElevatorFloor {
                floor,
                typ,
                state: EState::Noop,
            },
            FloorType::Elevator(no) => {
                UpDownElevatorFloor {
                    floor,
                    typ,
                    state: match self.lifts[no].state {
                        State::Stop => EState::Stop,
                        State::Maintaining => EState::Noop,
                        _ => EState::Running
                    },
                }
            }
        }
    }

    fn schedule2(&mut self, floor: TFloor, direction: Direction) -> Command<AppMessage> {
        let mut a = vec![
            self.new_up_down_elevator(floor, match direction {
                Direction::Up => FloorType::PersonUp,
                Direction::Down => FloorType::PersonDown,
            })];
        a.extend(self.lifts
            .iter()
            .filter(|lift| lift.state == State::Stop ||
                match direction {
                    Direction::Up => ((lift.state == State::GoingUp || lift.state == State::GoingUpSuspend)
                        && lift.cur_floor <= floor),
                    Direction::Down => ((lift.state == State::GoingDown || lift.state == State::GoingDownSuspend)
                        && lift.cur_floor >= floor),
                })
            // 不能超载
            .filter(|lift| !lift.is_overload())
            .map(|o| self.new_up_down_elevator(o.cur_floor, FloorType::Elevator(o.no)))
        );
        a.shrink_to_fit();
        match direction {
            Direction::Up => a.sort(),
            Direction::Down => a.sort_by(|a, b| b.cmp(a)),
        }
        let mut top_lift = None;
        let mut down_lift = None;
        let mut find = false;
        for item in a {
            match item.typ {
                FloorType::PersonUp | FloorType::PersonDown => find = true,
                FloorType::Elevator(no) => {
                    if !find {
                        top_lift = Some(no);
                    } else {
                        down_lift = Some(no);
                        break;
                    }
                }
            }
        }
        if top_lift.is_some() || down_lift.is_some() {
            let lift_idx;
            match top_lift {
                Some(top) => {
                    match down_lift {
                        Some(down) => {
                            let top_diff = (floor - self.lifts[top].cur_floor).abs();
                            let down_diff = (self.lifts[down].cur_floor - floor).abs();
                            if top_diff >= down_diff {
                                lift_idx = down
                            } else {
                                lift_idx = top;
                            }
                        }
                        _ => lift_idx = top
                    }
                }
                _ => {
                    match down_lift {
                        Some(down) => {
                            lift_idx = down;
                        }
                        _ => unreachable!()
                    }
                }
            }
            let lift = &mut self.lifts[lift_idx];
            lift.schedule_floors.insert(floor, Some(direction));
            let mut is_wait = false;
            if lift.state == State::Stop {
                is_wait = true;
                if lift.cur_floor > floor {
                    lift.state = State::GoingDown;
                } else if lift.cur_floor < floor {
                    lift.state = State::GoingUp;
                } else {
                    // 在同一个楼层时， 就开门进出人就可以了
                    match direction {
                        Direction::Up => lift.state = State::GoingUpSuspend,
                        Direction::Down => lift.state = State::GoingDownSuspend,
                    }
                    lift.set_persons();
                }
            }
            self.wait_floors
                .iter_mut()
                .filter(|wf| wf.floor == floor && wf.direction == direction)
                .for_each(|wf| wf.is_scheduled = true);
            return Command::perform(async move {
                Lift::suspend_one_by_one_floor(lift_idx, is_wait).await
            }, |msg| msg);
        }
        Command::none()
    }

    const fn calc_rows2(total: i32, per: i32) -> i32 {
        let rows = total / per;
        let m = total % per;
        if m == 0 {
            rows
        } else {
            rows + 1
        }
    }

    fn set_random_floor(&mut self) {
        loop {
            let f = random_floor();
            if f != self.floor {
                self.floor = f;
                self.tmp_floor = f;
                // 直到生产一个不同的楼层才终止循环。
                return;
            }
        }
    }

    fn add_to_wait_floor(&mut self, direction: Direction) -> Command<AppMessage> {
        let fi = WaitFloorTxtState {
            floor: self.floor,
            direction,
            is_scheduled: false,
        };
        if MAX_WAIT_FLOOR_NUM > self.wait_floors.len() {
            if !self.wait_floors.contains(&fi) {
                self.wait_floors.push_back(fi);
            }
        } else {
            println!("电梯繁忙，请稍后再试,{}", self.floor);
        }
        self.set_random_floor();
        self.schedule2(fi.floor, direction)
    }
}

impl Application for ElevatorApp {
    type Executor = executor::Default;
    type Message = AppMessage;
    type Flags = ();

    fn new(_: Self::Flags) -> (Self, Command<Self::Message>) {
        (Self::default(), Command::none())
    }

    fn title(&self) -> String {
        format!("多路电梯调度器")
    }

    fn update(&mut self, message: Self::Message, clipboard: &mut Clipboard) -> Command<Self::Message> {
        // println!("{:?}", message);
        match message {
            AppMessage::SliderChange(floor) => {
                if floor != 0 {
                    self.tmp_floor = floor;
                }
            }
            AppMessage::SliderRelease(floor) => {
                if floor != 0 {
                    self.floor = floor;
                }
            }
            AppMessage::ClickedBtnPlus => {
                if self.floor == -1 {
                    self.floor = 1
                } else {
                    self.floor += 1
                }
                self.floor = min(self.floor, MAX_FLOOR);
                self.tmp_floor = self.floor;
            }
            AppMessage::ClickedBtnSubtract => {
                if self.floor == 1 {
                    self.floor = -1
                } else {
                    self.floor -= 1
                }
                self.floor = max(self.floor, MIN_FLOOR);
                self.tmp_floor = self.floor;
            }
            AppMessage::ClickedBtnUp => {
                return self.add_to_wait_floor(Direction::Up);
            }
            AppMessage::ClickedBtnDown => {
                return self.add_to_wait_floor(Direction::Down);
            }
            AppMessage::Scheduling => {
                return Command::batch(self
                    .wait_floors
                    .iter()
                    .filter(|wf| !wf.is_scheduled)
                    .map(|wf| {
                        let floor = wf.floor;
                        let direction = wf.direction;
                        Command::perform(async {}, move |_| {
                            AppMessage::Scheduling2(floor, direction)
                        })
                    })
                );
            }
            AppMessage::LiftRunning => {
                return Command::batch(self
                    .lifts
                    .iter()
                    .filter(|lift| !lift.stop_floors.is_empty())
                    .map(|lift| {
                        let no = lift.no;
                        println!("LiftRunning {},{}", lift.to_string(),
                                 lift.stop_floors.keys().map(|k| k.to_string())
                                     .collect::<Vec<_>>().join(","));
                        Command::perform(async move {
                            Lift::suspend_one_by_one_floor(no, false).await
                        }, move |msg| msg,
                        )
                    })
                );
            }
            AppMessage::Scheduling2(floor, direction) => {
                return self.schedule2(floor, direction);
            }

            AppMessage::ArriveByOneFloor(no) => {
                let lift = &mut self.lifts[no];
                let no = lift.no;
                return if let Some(dest_floor) = lift.dest_floor() {
                    if lift.state == State::GoingUp {
                        // 避免出现楼层为 0 的情况
                        if lift.cur_floor == -1 {
                            lift.cur_floor = 1;
                        } else {
                            lift.cur_floor += 1;
                        }
                    } else if lift.state == State::GoingDown {
                        if lift.cur_floor == 1 {
                            lift.cur_floor = -1;
                        } else {
                            lift.cur_floor -= 1;
                        }
                    }
                    let is_arrive = lift.cur_floor == dest_floor;
                    if is_arrive {
                        lift.state = match lift.state {
                            State::GoingUp => State::GoingUpSuspend,
                            State::GoingUpSuspend => State::GoingUp,
                            State::GoingDown => State::GoingDownSuspend,
                            State::GoingDownSuspend => State::GoingDown,
                            State::Stop => State::Stop,
                            _ => {
                                println!("ArriveByOneFloor {:?}", lift.state);
                                unreachable!()
                            }
                        };
                        Self::remove_wait_floor(&mut self.wait_floors, dest_floor, lift);
                        println!("ArriveByOneFloor 电梯#{},已达到楼层{},正在等人进出。", no, dest_floor);
                        return Command::perform(async move {}, move |_| AppMessage::WaitUserInputFloor(no));
                    }
                    let no = lift.no;
                    Command::perform(async move {
                        Lift::suspend_one_by_one_floor(no, is_arrive).await
                    }, |msg| msg)
                } else {
                    lift.state = State::Stop;
                    Command::none()
                };
            }

            AppMessage::WaitUserInputFloor(no) => {
                let lift = &mut self.lifts[no];
                match lift.state {
                    State::GoingUpSuspend => lift.state = State::GoingUp,
                    State::GoingDownSuspend => lift.state = State::GoingDown,
                    _ => {}
                };
                lift.set_persons();
                if lift.persons != 0 {
                    lift.can_click_btn = true;
                }
                println!("_WaitUserInputFloor 电梯#{}-{}层,{}", no, lift.cur_floor, lift.state.to_string());
                return Command::perform(async move {
                    Lift::suspend_one_by_one_floor(no, false).await
                }, |msg| msg);
            }

            AppMessage::ClickedBtnFloor(no, floor) => {
                let lift = &mut self.lifts[no];
                if lift.can_click_btn {
                    let btn = lift.elevator_btns
                        .iter_mut()
                        .find(|o| o.floor == floor)
                        .unwrap();
                    btn.is_active = !btn.is_active;
                    btn.last_pressed = Some(Instant::now());
                    let first_floor =  lift.stop_floors.iter().next().map(|o| *o.0);
                    if btn.is_active {
                        let mut can_insert = match first_floor {
                            None => true,
                            Some(_) => {
                                match lift.state {
                                    State::GoingUp |State::GoingUpSuspend => floor > lift.cur_floor,
                                    State::GoingDown |State::GoingDownSuspend => floor < lift.cur_floor,
                                    State::Stop => true,
                                    _ => false,
                                }
                            }
                        };
                        if can_insert {
                            lift.stop_floors.insert(floor, None);
                        }
                    } else {
                        if lift.stop_floors.len() > 1 {
                            // 超过一个输入时， 才允许删除
                            lift.stop_floors.remove(&floor);
                        } else {
                            btn.is_active = true;
                        }
                    }

                    if lift.state == State::Stop {
                        // 有且只有一个输入时， 第一个楼层决定电梯的运行方向
                        if lift.stop_floors.len() == 1 {
                            if let Some(floor) =  lift.stop_floors.iter().next().map(|o| *o.0){
                                if lift.cur_floor < floor {
                                    lift.state = State::GoingUp
                                } else {
                                    lift.state = State::GoingDown
                                }
                                println!("elevator_btns,{}", lift.to_string());
                                lift.elevator_btns
                                    .iter_mut()
                                    .filter(|btn| match lift.state {
                                        State::GoingUp => btn.floor <= lift.cur_floor,
                                        State::GoingDown => btn.floor >= lift.cur_floor,
                                        _ => unreachable!()
                                    })
                                    .for_each(|btn| {
                                        btn.can_click = false
                                    })
                            }
                        }
                    }
                }


                println!("电梯#{},按了{}层, {}", no + 1, floor, lift.stop_floors
                    .keys()
                    .into_iter()
                    .map(|o| o.to_string())
                    .collect::<Vec<_>>()
                    .join(","));
            }
            _ => {}
        }
        Command::none()
    }

    fn subscription(&self) -> Subscription<Self::Message> {
        // 每隔3秒检查一次是否有用户要乘电梯，有的话，就要去调度
        Subscription::batch(vec![
            time::every(Duration::from_secs(5))
                .map(|_| AppMessage::Scheduling),
            time::every(Duration::from_secs(5))
                .map(|_| AppMessage::LiftRunning),
        ])
    }

    fn view(&mut self) -> Element<'_, Self::Message> {
        let mut subs = vec![];
        let slider = Slider::new(
            &mut self.slider_state,
            (MIN_FLOOR..=MAX_FLOOR),
            self.tmp_floor,
            AppMessage::SliderChange)
            .on_release(AppMessage::SliderRelease(self.tmp_floor))
            .width(Length::FillPortion(2))
            .into();
        let floor = Text::new(&format!("{}", self.floor))
            .width(Length::Units(30))
            .into();
        let e = Text::new("所在楼层: ")
            .into();

        let up_btn_row = Row::with_children(vec![
            Button::new(&mut self.up_btn_state, up_icon()
                .color(Color::from_rgb8(255, 0, 0)))
                .on_press(AppMessage::ClickedBtnUp)
                .width(Length::Units(30))
                .into(),
            Space::with_width(Length::Units(10)).into(),
            Button::new(&mut self.down_btn_state, down_icon()
                .color(Color::from_rgb8(0, 0, 255)))
                .on_press(AppMessage::ClickedBtnDown)
                .width(Length::Units(30))
                .into(),
        ]).width(Length::FillPortion(1))
            .spacing(10).into();

        subs.push(Button::new(&mut self.subtract_btn_state, subtract_icon())
                      .width(Length::Units(20))
                      .on_press(AppMessage::ClickedBtnSubtract)
                      .into(), );
        subs.push(Space::with_width(Length::Units(5)).into());
        subs.push(slider);
        subs.push(Space::with_width(Length::Units(5)).into());
        subs.push(Button::new(&mut self.plus_btn_state, plus_icon())
                      .width(Length::Units(20))
                      .on_press(AppMessage::ClickedBtnPlus)
                      .into(), );
        subs.push(Space::with_width(Length::Units(20)).into());
        subs.push(e);
        subs.push(Space::with_width(Length::Units(4)).into());
        subs.push(floor);
        subs.push(Space::with_width(Length::Units(20)).into());
        subs.push(up_btn_row);
        subs.push(Space::with_width(Length::FillPortion(2)).into());
        let mut rows = vec![
            Column::with_children(vec![
                Row::with_children(subs)
                    .padding(4)
                    .width(Length::Fill)
                    .align_items(Align::Center).into(),
                Container::new(Row::with_children(
                    vec![
                        Container::new(
                            Text::new("等待的楼层:"))
                            .height(Length::Fill)
                            .align_x(Align::Center)
                            .align_y(Align::Center)
                            .into(),
                        {
                            let mut i = 1;
                            let mut rows = vec![];
                            let mut row_elements = vec![];
                            for f in self.
                                wait_floors
                                .iter_mut()
                                .fold(vec![], |mut row, txt| {
                                    row.push(txt.floor_view());
                                    row
                                }) {
                                row_elements.push(f);
                                if i % WAIT_FLOOR_PER_ROW == 0 {
                                    rows.push(Row::with_children(row_elements
                                        .drain(..)
                                        .collect())
                                        .padding(4)
                                        .spacing(6)
                                        .into())
                                }
                                i += 1;
                            }
                            if !row_elements.is_empty() {
                                rows.push(Row::with_children(row_elements
                                    .drain(..)
                                    .collect())
                                    .padding(4)
                                    .spacing(6).into())
                            }
                            Column::with_children(rows).into()
                        },
                    ])
                    .width(Length::Fill)
                    .align_items(Align::Start)
                ).height(Length::Units(80))
                    .align_x(Align::Start)
                    .align_y(Align::Center)
                    .into(),
            ])

                .width(Length::Fill)
                .spacing(2)
                .into(),
        ];
        let new_rows = self.lifts
            .iter_mut()
            .fold(rows, |mut _rows, lift| {
                let status = Column::with_children(
                    vec![
                        Row::with_children(vec![
                            Text::new("电梯编号:").width(Length::FillPortion(1)).into(),
                            Text::new(format!("{}", lift.no + 1)).width(Length::FillPortion(2)).into(),
                        ]).spacing(10).padding(4).into(),
                        Row::with_children(vec![
                            Text::new("运行状态:").width(Length::FillPortion(1)).into(),
                            Text::new(format!("{}", lift.state.to_string())).color(
                                match lift.state {
                                    State::Maintaining => Color::from_rgb8(250, 255, 51),
                                    State::Stop => Color::BLACK,
                                    State::GoingUp | State::GoingUpSuspend => Color::from_rgb8(255, 0, 0),
                                    State::GoingDown | State::GoingDownSuspend => Color::from_rgb8(0, 0, 255),
                                }
                            ).width(Length::FillPortion(2)).into(),
                            match lift.state {
                                State::Stop | State::Maintaining => Text::new("")
                                    .width(Length::Units(20))
                                    .into(),
                                _ => loading_icon()
                                    .color(Color::from_rgb8(51, 134, 255))
                                    .width(Length::Units(20))
                                    .into()
                            },
                        ]).spacing(10).padding(4).into(),
                        Row::with_children(vec![
                            Text::new("所在楼层:").width(Length::FillPortion(1)).into(),
                            Text::new(format!("{}", lift.cur_floor)).width(Length::FillPortion(2)).into(),
                        ]).spacing(10).padding(4).into(),
                        Row::with_children(vec![
                            Text::new("人数:").width(Length::FillPortion(1)).into(),
                            Text::new(format!("{}", lift.persons)).width(Length::FillPortion(2)).into(),
                        ], ).spacing(10).padding(4).into(),
                        Row::with_children(vec![
                            Text::new(lift
                                .schedule_floors
                                .iter()
                                .map(|o| o.0.to_string())
                                .collect::<Vec<_>>().join(","))
                                .width(Length::Fill)
                                .color(Color::from_rgb8(51, 161, 255))
                                .into(),
                        ], ).spacing(10).padding(4).into(),
                    ]).width(Length::FillPortion(1))
                    .into();
                let mut row_floors = Vec::with_capacity(Self::floor_rows() as usize);
                let mut tmp_row = Vec::with_capacity(BTN_PER_ROW as usize);
                let mut i = 1;
                let lift_no = lift.no;
                let floors = &mut lift.elevator_btns;
                for f in floors
                    .iter_mut()
                    .enumerate()
                    .fold(vec![],
                          |mut row, (ix, floor)| {
                              floor.can_click = lift.can_click_btn;
                              floor.is_active = lift.stop_floors.contains_key(&floor.floor);
                              row.push(floor.floor_view());
                              row
                          }) {
                    tmp_row.push(f);
                    if i % BTN_PER_ROW == 0 {
                        row_floors.push(Row::with_children(
                            tmp_row
                                .drain(..)
                                .collect())
                            .spacing(10)
                            .padding(4)
                            .into()
                        );
                    }
                    i += 1;
                }
                if !tmp_row.is_empty() {
                    row_floors.push(Row::with_children(
                        tmp_row
                            .drain(..)
                            .collect())
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
                    elevator_floors,
                ]).into());
                _rows
            });
        Column::with_children(new_rows)
            .spacing(10)
            .height(Length::Fill)
            .height(Length::Fill).into()
    }
}
