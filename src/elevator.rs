#![feature(map_first_last)]

use std::collections::{BTreeSet, VecDeque};
use std::sync::{Arc, RwLock};
use std::sync::mpsc::{Receiver, Sender};
use std::{thread, time};
use std::cmp::{max, min};
use std::fmt::{Display, Formatter};
use std::ops::Deref;
use std::option::Option::Some;
use std::thread::sleep;
use crate::floor::Floor;
use crate::state::State;
use crate::conf::*;
use crate::message::Message;
use rand::prelude::*;

// 电梯元数据
#[derive(Debug)]
pub struct ElevatorMeta {
    // 当前电梯的人数
    pub persons: u8,
    // 电梯当前所处楼层
    pub cur_floor: i16,
    // 每次电梯从静止态开
    // 始运行时，第一个按入要前往的楼层，决定了电梯是上升还是下降
}

impl Default for ElevatorMeta {
    fn default() -> Self {
        let cur_floor = {
            let mut rng = thread_rng();
            rng.gen_range(MIN_FLOOR..=MAX_FLOOR)
        };
        Self {
            cur_floor,
            persons: 0,
        }
    }
}


impl ElevatorMeta {
    pub fn diff_floor(&self, floor: i16) -> i16 {
        return if self.cur_floor < floor {
            floor - self.cur_floor
        } else {
            self.cur_floor - floor
        };
    }

    pub fn set_person_num(&mut self) -> (bool, u8) {
        // 随机设置 上下电梯人数
        let mut rng = thread_rng();
        let num = rng.gen_range(1..=8);
        // 加
        let plus = rng.gen_bool(0.5f64);
        if plus {
            self.persons = min(self.persons.saturating_add(num), MAX_PERSON_CAPACITY);
        } else {
            // 减
            self.persons = max(self.persons.saturating_sub(num), 0);
        }
        (plus, num)
    }
}

// 电梯
#[derive(Debug)]
pub struct Elevator {
    // 电梯序号
    pub no: u8,
    // 互斥量，需要用作电梯同步互斥的元数据；
    pub meta: Arc<RwLock<ElevatorMeta>>,
    pub stop_floors: Arc<RwLock<VecDeque<i16>>>,
    // 电梯运行状态
    pub state: Arc<RwLock<State>>,
}

impl Elevator {
    pub fn new(no: u8) -> Self {
        Self {
            no,
            meta: Arc::new(RwLock::new(ElevatorMeta::default())),
            stop_floors: Arc::new(RwLock::new(VecDeque::with_capacity(MAX_PERSON_CAPACITY as usize))),
            state: Arc::new(RwLock::new(State::default())),
        }
    }

    // 判定是否可以再上人
    pub fn can_in(&self, floor: Floor) -> bool {
        let mut can = self.meta.read().unwrap().persons < MAX_PERSON_CAPACITY;
        // 保证电梯不超载
        if !can {
            return can;
        }
        use State::*;
        let state = self.state.read().unwrap();
        let meta = self.meta.read().unwrap();
        match floor {
            // 在某楼层上楼
            Floor::Up(floor) => {
                match *state {
                    // 电梯维修、或下行，或者下行中上下人， 均不能上人
                    Maintaining | GoingDownSuspend | GoingDown => can = false,
                    // 电梯只要是静止状态，就可以上人
                    Stop => can = true,
                    // 如果上行，或者上行中上下人时，所在楼层比电梯楼层高，即可上人
                    GoingUp | GoingUpSuspend => can = meta.cur_floor <= floor,
                }
            }
            // 在某楼层下楼
            Floor::Down(floor) => {
                match *state {
                    // 电梯维修、或下行，或者下行中上下人， 均不能上人
                    Maintaining | GoingUp | GoingUpSuspend => can = false,
                    // 电梯只要是静止状态，就可以上人
                    Stop => can = true,
                    // 如果上行，或者上行中上下人时，所在楼层比电梯楼层高，即可上人
                    GoingDown | GoingDownSuspend => can = meta.cur_floor >= floor,
                }
            }
        }
        can
    }

    fn fake_run(nums: u64) {
        if nums > 0 {
            thread::sleep(time::Duration::from_millis(EVERY_FLOOR_RUN_TIME_IN_MILLISECONDS as u64 * nums));
        }
    }
    pub fn sleep_run() {
        thread::sleep(time::Duration::from_millis(ELEVATOR_SLEEP_TIME_IN_MILLISECONDS as u64));
    }
    fn wait_and_close_door() {
        thread::sleep(time::Duration::from_millis(SUSPEND_WAIT_IN_MILLISECONDS as u64));
    }

    fn handle_schedule_updown_floors(&self, floors: &[i16], send_to_schedule: Sender<Message>, is_up: bool) {
        println!("[schedule]电梯#{},处理调度器安排的{}楼任务", self.no, if is_up { "上" } else { "下" });
        // 处理调度器安排的上下楼任务
        for floor in floors {
            let mut plus = false;
            let mut delta = 0;
            {
                let (can_move, diff) = {
                    let m1 = self.can_in(Floor::Up(*floor));
                    (m1, self.meta.read().unwrap().diff_floor(*floor))
                };
                if can_move {
                    {
                        let mut state = self.state.write().unwrap();
                        if is_up {
                            *state = State::GoingUp;
                        } else {
                            *state = State::GoingDown;
                        }
                    }
                    // 移动到指定楼层

                    // 通过 sleep 假装电梯在逐层运行
                    println!("[schedule]电梯#{},[{} -> {}]，请耐心等待...",
                             self.no,
                             self.meta.read().unwrap().cur_floor,
                             floor,
                    );
                    // 通过 sleep 假装电梯在逐层运行
                    {
                        let mut state = self.state.write().unwrap();
                        if is_up {
                            *state = State::GoingUpSuspend;
                        } else {
                            *state = State::GoingDownSuspend;
                        }
                    }
                    Self::fake_run(diff as u64);
                    (plus, delta) = self.meta.write().unwrap().set_person_num();
                    println!("[schedule]电梯#{},[{} -> {}]，已完成!\t电梯开门...",
                             self.no,
                             self.meta.read().unwrap().cur_floor,
                             floor,
                    );
                }
                // 等人进出电梯
                println!("[schedule]电梯#{}，开门...! {{{}-delta:{}}}", self.no, if plus { "上" } else { "下" }, delta);
                Self::wait_and_close_door();
                println!("[schedule]电梯#{}，关门...! {{{}-delta:{}}}", self.no, if plus { "上" } else { "下" }, delta);
                // 上人了才通知调度器， 用户进入了电梯，现在需要用户输入前往的楼层了
                if plus && delta > 0 {
                    println!("[schedule]电梯#{}，请求用户输入楼层! delta:{}", self.no, delta);
                    for _ in 0..delta {
                        send_to_schedule.send(Message::InputtingFloor(self.no)).unwrap();
                    }
                }
            }
        }
        // self.handle_person_updown_floors(is_up);
    }

    fn handle_state(&self) -> Option<bool> {
        // 处理电梯状态，
        let mut is_up = false;
        let state = self.state.read().unwrap();
        match state.deref() {
            State::Stop => {
                // 静止的电梯，那么第一个用户的输入的楼层觉得电梯是上升还是下降
                let first_floor = {
                    let fs = self.stop_floors.read().unwrap();
                    if fs.is_empty() { None } else { Some(fs[0]) }
                };
                if let Some(first) = first_floor {
                    let cur_floor = self.meta.read().unwrap().cur_floor;
                    if first == cur_floor {
                        println!("楼层 {} == {}", first, cur_floor);
                        return None;
                    }
                    is_up = first > cur_floor;
                    {
                        let mut lock = self.state.write().unwrap();
                        if is_up {
                            *lock = State::GoingUp;
                        } else {
                            *lock = State::GoingDown;
                        }
                    }
                }
            }
            State::GoingUp | State::GoingUpSuspend => {
                is_up = true
            }
            State::GoingDown | State::GoingDownSuspend => {
                is_up = false
            }
            _ => {
                println!("{}", state);
                return None;
            }
        }
        Some(is_up)
    }
    fn handle_person_updown_floors(&self) {
        let floors = self.stop_floors.read().unwrap();
        if floors.is_empty() {
            // println!("[person]电梯#{}-{}层, 没有任务", self.no, self.meta.read().unwrap().cur_floor);
            Self::sleep_run();
            return;
        }
        let mut more_1floor = false; // 是否还有更多楼层，没有更多楼层时需要跑时，则把电梯的状态改为静止[Stop]
        if let Some(is_up) = self.handle_state() {
            let floor = {
                let mut sfloor = self.stop_floors.write().unwrap();
                // 上升时，升序
                if is_up {
                    sfloor.make_contiguous().sort();
                } else {
                    // 下降时，降序
                    sfloor.make_contiguous().sort_by(|a, b| b.cmp(a));
                }
                more_1floor = sfloor.len() > 1;
                sfloor.pop_front().unwrap()
            };
            // 每一次for循环，从 stop_floors 拿出一个合适的楼层进行前往，而不是一次行运行完全部 stop_floors
            let mut diff = {
                let mut meta = self.meta.read().unwrap();
                meta.diff_floor(floor)
            };
            // 通过 sleep 假装电梯在逐层运行
            println!("[person]电梯#{},[{} -> {}]，请耐心等待...",
                     self.no,
                     self.meta.read().unwrap().cur_floor,
                     floor,
            );
            Self::fake_run(diff as u64);
            // 到了指定楼层，则等人进出电梯
            {
                let mut lock = self.state.write().unwrap();
                let mut meta = self.meta.write().unwrap();
                if is_up {
                    *lock = State::GoingUpSuspend;
                    meta.cur_floor += diff;
                } else {
                    *lock = State::GoingDownSuspend;
                    meta.cur_floor -= diff;
                }
            };
            println!("[person]电梯#{},[{} -> {}]，已完成!\t电梯开门...",
                     self.no,
                     self.meta.read().unwrap().cur_floor,
                     floor,
            );
            Self::wait_and_close_door();
            println!("[person]电梯#{}，关门...!", self.no);
            if !more_1floor {
                //没有更多楼层时需要跑时，则把电梯的状态改为静止[Stop]
                let mut state = self.state.write().unwrap();
                *state = State::Stop;
            }
        } else {
            println!("[person]电梯#{}-{}层, 状态或者输入不对，不能处理请求!", self.no, self.meta.read().unwrap().cur_floor);
        }
    }

    pub fn run(&self, rx_from_schedule: Receiver<Message>, send_to_schedule: Sender<Message>) {
        use Message::*;
        loop {
            if let Ok(msg) = rx_from_schedule.try_recv() {
                match msg {
                    Quit => break,
                    Up(floor) => {
                        self.handle_schedule_updown_floors(
                            &vec![floor],
                            send_to_schedule.clone(),
                            true);
                    }
                    Ups(floors) => {
                        self.handle_schedule_updown_floors(
                            &floors,
                            send_to_schedule.clone(),
                            true);
                    }
                    InputtedFloor(no, floor) => {
                        assert_eq!(no, self.no);
                        println!("电梯[{},{}]-用户输入楼层：{}",
                                 no,
                                 self.meta.read().unwrap().cur_floor,
                                 floor);
                        match *self.state.read().unwrap() {
                            State::Stop => {
                                self.stop_floors.write().unwrap().push_back(floor)
                            }
                            State::GoingUp | State::GoingUpSuspend => {
                                if floor >= self.meta.read().unwrap().cur_floor {
                                    self.stop_floors.write().unwrap().push_back(floor)
                                } else {
                                    let cur_floor = self.meta.read().unwrap().cur_floor;
                                    println!("电梯#{}-{}层,..输入的楼层[{} < {}]不对，不处理!", self.no, cur_floor, floor, cur_floor);
                                }
                            }
                            State::GoingDown | State::GoingDownSuspend => {
                                if floor <= self.meta.read().unwrap().cur_floor {
                                    self.stop_floors.write().unwrap().push_back(floor)
                                } else {
                                    let cur_floor = self.meta.read().unwrap().cur_floor;
                                    println!("电梯#1{}-{}层,..输入的楼层[{} > {}]不对，不处理!", self.no, cur_floor, floor, cur_floor);
                                }
                            }
                            _ => {
                                println!("电梯#1{}-{}层,...维护中", self.no, self.meta.read().unwrap().cur_floor);
                            }
                        }
                    }

                    Down(floor) => {
                        self.handle_schedule_updown_floors(
                            &vec![floor],
                            send_to_schedule.clone(),
                            false);
                    }
                    Downs(floors) => {
                        self.handle_schedule_updown_floors(
                            &floors,
                            send_to_schedule.clone(),
                            false);
                    }
                    _ => {}
                }
            }
            self.handle_person_updown_floors()
        }
    }
}


impl Drop for Elevator {
    fn drop(&mut self) {
        println!("电梯{}停止工作了", self.no)
    }
}

impl Display for Elevator {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let meta = self.meta.read().unwrap();
        write!(f,
               "电梯#{}, \n\t{}人,在{}层, {}\n",
               self.no,
               meta.persons,
               meta.cur_floor,
               self.state.read().unwrap().to_string()
        )
    }
}