use std::collections::BTreeSet;
use std::sync::{Arc, RwLock};
use std::sync::mpsc::{Receiver, Sender};
use std::{thread, time};
use std::thread::sleep;
use crate::floor::Floor;
use crate::state::State;
use crate::conf::*;
use crate::message::Message;
use rand::prelude::*;
// 电梯元数据
#[derive(Default)]
pub struct ElevatorMeta {
    // 当前电梯的人数
    pub persons: u8,
    // 电梯当前所处楼层
    pub cur_floor: i16,
    // 电梯运行状态
    pub state: State,
    // 当前电梯人数
    // 电梯要停止的楼层。
    // 楼层是升序排列, 使用BTreeSet,BTreeSet的key是有序的
    pub stop_floors: BTreeSet<i16>,
}

impl Default for ElevatorMeta {
    fn default() -> Self {
        let cur_floor = {
            let mut rng = thread_rng();
            rng.gen_range(MIN_FLOOR..=MAX_FLOOR)
        };
        Self{
            cur_floor,
            ..Default::default()
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
    // 判定是否可以再上人
    pub fn can_in(&self, floor: Floor) -> bool {
        let mut can = self.persons < MAX_PERSON_CAPACITY;
        // 保证电梯不超载
        if !can {
            return can;
        }
        use State::*;
        match floor {
            // 在某楼层上楼
            Floor::Up(floor) => {
                match self.state {
                    // 电梯维修、或下行，或者下行中上下人， 均不能上人
                    Maintaining | GoingDownSuspend | GoingDown => can = false,
                    // 电梯只要是静止状态，就可以上人
                    Stop => can = true,
                    // 如果上行，或者上行中上下人时，所在楼层比电梯楼层高，即可上人
                    GoingUp | GoingUpSuspend => can = self.cur_floor <= floor,
                }
            }
            // 在某楼层下楼
            Floor::Down(floor) => {
                match self.state {
                    // 电梯维修、或下行，或者下行中上下人， 均不能上人
                    Maintaining | GoingUp | GoingUpSuspend => can = false,
                    // 电梯只要是静止状态，就可以上人
                    Stop => can = true,
                    // 如果上行，或者上行中上下人时，所在楼层比电梯楼层高，即可上人
                    GoingDown | GoingDownSuspend => can = self.cur_floor >= floor,
                }
            }
        }
        can
    }


    pub fn type_floor(&mut self, floor: i16) {
        // 进入电梯后的操作, 输入要前往的楼层
        if self.state == State::Maintaining {
            return;
        } else if self.state == State::Stop {
            if self.cur_floor == floor {
                return;
            } else if self.cur_floor > floor {
                self.state = State::GoingDown;
            } else {
                self.state = State::GoingUp;
            }
        }
        use State::*;
        match self.state {
            GoingDown | GoingDownSuspend => {
                self.stop_floors.insert(floor);
            }
            GoingUp | GoingUpSuspend => {
                self.stop_floors.insert(floor);
            }
            _ => {}
        }
    }
}

// 电梯
pub struct Elevator {
    // 电梯序号
    no: u8,
    // 互斥量，需要用作电梯同步互斥的元数据；
    meta: Arc<RwLock<ElevatorMeta>>,
}

impl Elevator {
    pub fn new(no: u8) -> Self {
        Self {
            no,
            meta: Arc::new(RwLock::new(ElevatorMeta::default())),
        }
    }

    fn fake_run() {
        thread::sleep(time::Duration::from_millis(EVERY_FLOOR_RUN_TIME_IN_MILLISECONDS as u64));
    }
    pub fn sleep_run() {
        thread::sleep(time::Duration::from_millis(ELEVATOR_SLEEP_TIME_IN_MILLISECONDS as u64));
    }
    fn wait_and_close_door() {
        thread::sleep(time::Duration::from_millis(SUSPEND_WAIT_IN_MILLISECONDS as u64));
    }

    pub fn run(&self, rxFromSchedule: Receiver<Message>, sendTofSchedule:Sender<Message>) {
        use Message::*;
        loop {
            if let Ok(msg) = rxFromSchedule.recv() {
                match msg {
                    Quit => break,
                    Up(floor) => {
                        let (can_going_up, mut diff) = {
                            let lock = self.meta.read().unwrap();
                            (lock.can_in(Floor::Up(floor)), lock.diff_floor(floor))
                        };
                        if can_going_up {
                            let mut lock;
                            // 上到指定楼层
                            while diff > 0 {
                                // 假装运行
                                Self::fake_run();
                                diff -= 1;
                                {
                                    lock = self.meta.write().unwrap();
                                    lock.state = State::GoingUp;
                                    lock.cur_floor += 1;
                                }
                            }
                            // 等人进入电梯
                            Self::wait_and_close_door();
                            // todo, 在电梯输入想去楼， 电梯跑到指定楼层
                        }
                    }
                    Down(floor) => {
                        let (can_going_down, mut diff) = {
                            let lock = self.meta.read().unwrap();
                            (lock.can_in(Floor::Down(floor)), lock.diff_floor(floor))
                        };
                        if can_going_down {
                            let mut lock;
                            // 下到指定楼层
                            while diff > 0 {
                                // 假装运行
                                Self::fake_run();
                                diff -= 1;
                                {
                                    lock = self.meta.write().unwrap();
                                    lock.state = State::GoingDown;
                                    lock.cur_floor -= 1;
                                }
                            }
                            // 等人进入电梯
                            Self::wait_and_close_door();
                            // todo, 在电梯输入想去楼， 电梯跑到指定楼层
                        }
                    }
                    _ => {}
                }
            } else {
                Self::sleep_run();
            }
        }
    }
}


impl Drop for Elevator {
    fn drop(&mut self) {
        println!("电梯{}停止工作了", self.no)
    }
}