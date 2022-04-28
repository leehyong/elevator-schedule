#![feature(map_first_last)]

use std::collections::BTreeSet;
use std::sync::{Arc, RwLock};
use std::sync::mpsc::{Receiver, Sender};
use std::{thread, time};
use std::cmp::{max, min};
use std::fmt::{Display, Formatter};
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
        Self {
            cur_floor,
            persons: 0,
            state: State::default(),
            stop_floors: BTreeSet::new(),
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
#[derive(Debug)]
pub struct Elevator {
    // 电梯序号
    pub no: u8,
    // 互斥量，需要用作电梯同步互斥的元数据；
    pub meta: Arc<RwLock<ElevatorMeta>>,
}

impl Elevator {
    pub fn new(no: u8) -> Self {
        Self {
            no,
            meta: Arc::new(RwLock::new(ElevatorMeta::default())),
        }
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
                let mut lock = self.meta.write().unwrap();
                let can_move = lock.can_in(Floor::Up(*floor));
                let mut diff = lock.diff_floor(*floor);
                if can_move {
                    // 移动到指定楼层
                    if is_up {
                        lock.state = State::GoingUp;
                    } else {
                        lock.state = State::GoingDown;
                    }
                    // 通过 sleep 假装电梯在逐层运行
                    println!("[schedule]电梯#{},[{} -> {}]，请耐心等待...",
                             self.no,
                             lock.cur_floor,
                             floor,
                    );
                    // 通过 sleep 假装电梯在逐层运行
                    Self::fake_run(diff as u64);
                    if is_up {
                        lock.state = State::GoingUpSuspend;
                    } else {
                        lock.state = State::GoingDownSuspend;
                    }
                    (plus, delta) = lock.set_person_num();
                    println!("[schedule]电梯#{},[{} -> {}]，已完成!\t电梯开门...",
                             self.no,
                             lock.cur_floor,
                             floor,
                    );
                }
                // 等人进出电梯
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

    fn handle_person_updown_floors(&self) {
        // 处理用户输入的上下楼任务
        let mut meta = self.meta.write().unwrap();
        let is_up = false;
        let mut floors: Vec<i16> = meta
            .stop_floors
            .iter()
            .map(|o| *o)
            .collect();
        println!("[person]电梯#{}, floors {:?}", self.no, &floors);
        if floors.len() > 0 {
            while let Some(floor) = floors.pop() {
                let mut diff = {
                    meta.stop_floors.remove(&floor);
                    meta.state = if is_up { State::GoingUp } else { State::GoingDown };
                    meta.diff_floor(floor)
                };
                // 通过 sleep 假装电梯在逐层运行
                println!("[person]电梯#{},[{} -> {}]，请耐心等待...",
                         self.no,
                         meta.cur_floor,
                         floor,
                );
                Self::fake_run(diff as u64);
                // 到了指定楼层，则等人进出电梯
                {
                    meta.stop_floors.remove(&floor);
                    if is_up {
                        meta.state = State::GoingUpSuspend;
                        meta.cur_floor += diff;
                    } else {
                        meta.state = State::GoingDownSuspend;
                        meta.cur_floor -= diff;
                    }
                };
                println!("[person]电梯#{},[{} -> {}]，已完成!\t电梯开门...",
                         self.no,
                         meta.cur_floor,
                         floor,
                );
                Self::wait_and_close_door();
                println!("[person]电梯#{}，关门...!", self.no);
            }
        }
        assert_eq!(meta.stop_floors.len(), 0);
        meta.state = State::Stop;
    }

    pub fn run(&self, rx_from_schedule: Receiver<Message>, send_to_schedule: Sender<Message>) {
        use Message::*;
        loop {
            if let Ok(msg) = rx_from_schedule.recv() {
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
                        let mut meta = self.meta.write().unwrap();
                        meta.stop_floors.insert(floor);
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

impl Display for Elevator {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let meta = self.meta.read().unwrap();
        write!(f,
               "电梯#{}, \n\t{}人,在{}层, {}\n",
               self.no,
               meta.persons,
               meta.cur_floor,
               meta.state
        )
    }
}