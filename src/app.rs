#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::cmp::{max, min};
use std::collections::{BTreeMap, HashMap, LinkedList};
use std::time::{Duration, Instant};
use crate::message::*;
use iced::*;
use iced::futures::SinkExt;
use iced::window::Mode;
use rand::{Rng, thread_rng};
use crate::conf::{MAX_ELEVATOR_NUM, MAX_FLOOR, MIN_FLOOR, TFloor};
use crate::util::*;
use crate::floor_btn::{Direction, FloorBtnState, WaitFloorTxtState};
use crate::icon::*;
use crate::lift::{Lift, LiftUpDownCost};
use crate::up_down_elevator_floor::*;
use crate::state::State;


struct ElevatorApp {
    floor: TFloor,
    tmp_floor: TFloor,
    slider_state: slider::State,
    up_btn_state: button::State,
    plus_btn_state: button::State,
    subtract_btn_state: button::State,
    down_btn_state: button::State,
    // 电梯里的按钮
    elevator_btns: Vec<Vec<FloorBtnState>>,
    // 哪些楼层需要安排电梯去接人的
    wait_floors: LinkedList<WaitFloorTxtState>,
    lifts: Vec<Lift>,
}

impl Default for ElevatorApp {
    fn default() -> Self {
        let mut hp = Vec::with_capacity(MAX_ELEVATOR_NUM as usize);
        let mut lifts = Vec::with_capacity(MAX_ELEVATOR_NUM);
        for no in 0..MAX_ELEVATOR_NUM {
            hp.push(
                (MIN_FLOOR..=MAX_FLOOR)
                    .into_iter()
                    .filter(|o| *o != 0)
                    .map(|o|
                        {
                            let mut btn_state = FloorBtnState::default();
                            btn_state.elevator_no = no;
                            btn_state.floor = o;
                            btn_state
                        }).collect());
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
            elevator_btns: hp,
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
const MAX_WAIT_FLOOR_ROW_NUM: TFloor = 4;
const MAX_WAIT_FLOOR_NUM: usize = (BTN_PER_ROW * MAX_WAIT_FLOOR_ROW_NUM) as usize;


impl ElevatorApp {
    const fn floor_rows() -> i32 {
        Self::calc_rows2(MAX_FLOOR - MIN_FLOOR, BTN_PER_ROW)
    }

    fn handle_up_floors(&self, floors: &[TFloor]) -> Vec<LiftUpDownCost> {
        // todo , 怎么规划，才能做到最小代价？
        self.lifts
            .iter()
            .filter(
                move |o| o.state == State::Stop
            )
            .map(
                |lift| {
                    // 每个静止的电梯都要考虑， 上下两个方向的成本
                    let mut cnt = 0;
                    let cost = floors.iter()
                        .filter(|floor| **floor >= lift.cur_floor)
                        .map(|floor| {
                            cnt += 1;
                            (floor - lift.cur_floor)
                        }).sum();
                    LiftUpDownCost {
                        no,
                        direction:Direction::Up,
                        cost,
                        cnt,
                    }
                }).collect()
    }
    fn handle_down_floors(&self, floors: &[TFloor]) -> Vec<LiftUpDownCost> {
        self.lifts
            .iter()
            .filter(
                move |o| o.state == State::Stop
            )
            .map(
                lift | {
                    // 每个静止的电梯都要考虑， 上下两个方向的成本
                    let mut cnt = 0;
                    let cost = floors.iter()
                        .filter(|floor| **floor <= lift.cur_floor)
                        .map(|floor| {
                            cnt += 1;
                            lift.cur_floor - floor
                        }).sum();
                    LiftUpDownCost {
                        no,
                        direction:Direction::Down,
                        cost,
                        cnt,
                    }
                }).collect()
    }


    fn schedule_stopped_lift(&mut self, up_floors: &[TFloor], down_floors: &[TFloor]) {
        // 上行代价和下行代价相同是，尽量去接 楼层数更多的
        // 最小的上行代价
        let up = self.handle_up_floors(up_floors);
        let down = self.handle_down_floors(down_floors);
        // match up.no {
        //     Some(up) => {
        //         match down.no {
        //             Some(down) => {
        //                 let up_lift = &mut self.lifts[up];
        //                 floors
        //                     .iter()
        //                     .filter(|o| **o >= up_lift.cur_floor)
        //                     .for_each(|v| {
        //                         lift.schedule_floors.insert(*v);
        //                     });
        //                 if up != down {
        //                     let down_lift = &mut self.lifts[down];
        //                     floors
        //                         .iter()
        //                         .filter(|o| **o <= down_lift.cur_floor)
        //                         .for_each(|v| {
        //                             lift.schedule_floors.insert(*v);
        //                         });
        //                 }
        //             }
        //             None => {}
        //         }
        //     }
        //     None => {}
        // }
    }

    fn schedule_running_lift(&mut self) {
        for direction in [Direction::Up, Direction::Down] {
            let mut one_direction_floors = self.wait_floors
                .iter()
                .filter(|o| o.direction == direction)
                .map(|o| {
                    UpDownElevatorFloor { floor: o.floor, typ: FloorType::Person }
                })
                .collect::<Vec<_>>();
            self.lifts
                .iter_mut()
                .filter(
                    move |o|
                        match direction {
                            Direction::Up => { o.state == State::GoingUp || o.state == State::GoingUpSuspend }
                            Direction::Down => { o.state == State::GoingDown || o.state == State::GoingDownSuspend }
                        })
                .for_each(
                    |lift| {
                        one_direction_floors.push(UpDownElevatorFloor { floor: lift.cur_floor, typ: FloorType::Elevator(lift.no) })
                    });
            // 通过排序， 确定每个电梯应该响应哪些楼层
            match direction {
                // 上升， 升序
                Direction::Up => one_direction_floors.sort(),
                // 下降， 降序
                Direction::Down => one_direction_floors.sort_by(|a, b| b.cmp(a))
            }
            let mut elevator = None;
            for item in one_direction_floors {
                match item.typ {
                    FloorType::Elevator(idx) => {
                        elevator = Some(&mut self.lifts[idx])
                    }
                    FloorType::Person => {
                        if let Some(ele) = elevator {
                            ele.schedule_floors.insert(item.floor);
                        }
                    }
                }
            }
        }
    }

    fn schedule(&mut self) -> Command<AppMessage> {
        // 1、优先从运行的的电梯中，去选择合适的电梯去处理
        self.schedule_running_lift();
        // 2、或者从停止的电梯中，去选择合适的电梯去处理
        let f = self.wait_floors
            .iter()
            .filter(|o| {
                for lift in self.lifts.iter().filter(|o| o.state == State::Stop) {
                    // 方向一直性检查
                    if lift.schedule_floors.contains(&o.floor) {
                        match lift.state {
                            State::GoingUp | State::GoingUpSuspend => {
                                if o.direction == Direction::Up {
                                    // 不需要被选中
                                    return false;
                                }
                            }
                            State::GoingDown | State::GoingDownSuspend => {
                                if o.direction == Direction::Down {
                                    // 不需要被选中
                                    return false;
                                }
                            }
                            _ => {}
                        }
                    }
                }
                // 所有电梯都没有调度到该楼层
                true
            });
        let remain_up_floors = f.filter(|o| o.direction == Direction::Up)
            .map(|o| o.floor)
            .collect::<Vec<_>>();
        let remain_down_floors = f.filter(|o| o.direction == Direction::Down)
            .map(|o| o.floor)
            .collect::<Vec<_>>();
        self.schedule_stopped_lift(&remain_up_floors, &remain_down_floors);
        Command::perform(async {}, AppMessage::Scheduled)
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
                break;
            }
        }
    }

    fn add_to_wait_floor(&mut self, direction: Direction) {
        let fi = WaitFloorTxtState {
            floor: self.floor,
            direction,
        };
        if MAX_WAIT_FLOOR_NUM > self.wait_floors.len() {
            if !self.wait_floors.contains(&fi) {
                self.wait_floors.push_back(fi);
            }
        } else {
            println!("电梯繁忙，请稍后再试,{}", self.floor);
        }
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
                self.add_to_wait_floor(Direction::Up);
                self.set_random_floor();
            }
            AppMessage::ClickedBtnDown => {
                self.add_to_wait_floor(Direction::Down);
                self.set_random_floor();
            }
            AppMessage::Scheduling => {
                // todo
                println!("电梯调度");
                self.schedule()
            }
            AppMessage::Scheduled => {
                // todo
                println!("电梯调度完成了");
            }

            AppMessage::ClickedBtnFloor(no, floor) => {
                let btn = self.elevator_btns[no as usize]
                    .iter_mut()
                    .find(|o| o.floor == floor)
                    .unwrap();
                // todo:  由于iced 的Button没有双击事件，此处无法正确模拟双击， 留待以后再解决 双击取消某楼层
                if let Some(inst) = btn.last_pressed {
                    // 在一定毫秒内毫秒内连续点击了多次，就认为是双击了
                    println!("inst.elapsed().as_millis() < 1000_000 : {}, {}", inst.elapsed().as_millis() < 1000_000, inst.elapsed().as_micros());
                    if inst.elapsed().as_millis() < 1000 {
                        btn.is_active = false;
                    }
                    btn.last_pressed = None
                } else {
                    btn.is_active = true;
                    btn.last_pressed = Some(Instant::now());
                }
                // println!("电梯#{},按了{}层, {}, {:?}", no, floor, btn.is_active, btn.last_pressed);
                println!("电梯#{},按了{}层,", no + 1, floor);
            }
            _ => {}
        }
        Command::none()
    }

    fn subscription(&self) -> Subscription<Self::Message> {
        // 每隔3秒检查一次是否有用户要乘电梯，有的话，就要去调度
        Subscription::batch(vec![
            time::every(Duration::from_secs(3))
                .map(|_| AppMessage::Scheduling),
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
                    .padding(10)
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
                ).height(Length::Units(110))
                    .align_x(Align::Start)
                    .align_y(Align::Center)
                    .into(),
            ])

                .width(Length::Fill)
                .spacing(4)
                .into(),
        ];
        let new_rows = self.elevator_btns
            .iter_mut()
            .enumerate()
            .fold(rows, |mut _rows, (elevator_no, floors)| {
                let status = Column::with_children(vec![
                    Row::with_children(vec![
                        Text::new("电梯编号:").width(Length::FillPortion(1)).into(),
                        Text::new(format!("{}", elevator_no + 1)).width(Length::FillPortion(1)).into(),
                    ]).spacing(10).padding(4).into(),
                    Row::with_children(vec![
                        Text::new("运行状态:").width(Length::FillPortion(1)).into(),
                        Text::new(format!("{}运行中", elevator_no + 1)).width(Length::FillPortion(1)).into(),
                    ]).spacing(10).padding(4).into(),
                    Row::with_children(vec![
                        Text::new("所在楼层:").width(Length::FillPortion(1)).into(),
                        Text::new(format!("{}", elevator_no + 1)).width(Length::FillPortion(1)).into(),
                    ]).spacing(10).padding(4).into(),
                    Row::with_children(vec![
                        Text::new("人数:").width(Length::FillPortion(1)).into(),
                        Text::new(format!("{}", 0)).width(Length::FillPortion(1)).into(),
                    ]).spacing(10).padding(4).into(),
                ]).width(Length::FillPortion(1))
                    .into();
                let mut row_floors = Vec::with_capacity(Self::floor_rows() as usize);
                let mut tmp_row = Vec::with_capacity(BTN_PER_ROW as usize);
                let mut i = 1;
                for f in floors
                    .iter_mut()
                    .enumerate()
                    .fold(vec![],
                          |mut row, (ix, floor)| {
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
            .spacing(30)
            .height(Length::Fill)
            .height(Length::Fill).into()
    }
}
