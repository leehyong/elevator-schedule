use std::cmp::{max, min, Ordering};
use std::collections::{BTreeMap, BTreeSet, HashMap, LinkedList};
use std::fmt::{Display, Formatter};
use crate::conf::{EVERY_FLOOR_RUN_TIME_IN_MILLISECONDS, MAX_ELEVATOR_NUM, MAX_FLOOR, MAX_PERSON_CAPACITY, MIN_FLOOR, TFloor};
use crate::floor_btn::{Direction, FloorBtnState};
use crate::message::AppMessage;
use crate::state::State;
use tokio::sync::Mutex;
use std::sync::Arc;
use crate::util::{random_bool, random_person_num};


// 电梯
#[derive(Default)]
pub struct Lift {
    // 电梯序号
    pub no: usize,
    // 电梯运行状态
    pub state: State,
    // 电梯内所搭载的人数
    pub persons: i32,
    // 电梯当前停靠楼层
    pub cur_floor: TFloor,
    // 用户输入的停靠楼层
    pub can_click_btn: bool,
    pub stop_floors: BTreeMap<TFloor, Option<Direction>>,
    // 调度器调度的停靠楼层
    // 上行时，schedule_floors 的元素值 > cur_floor
    // 下行时，schedule_floors 的元素值 < cur_floor
    pub schedule_floors: BTreeMap<TFloor, Option<Direction>>,
    // 电梯里的按钮
    pub elevator_btns: Vec<FloorBtnState>,
}
lazy_static!(
    static ref LiftLocks: HashMap<usize,Arc<Mutex<bool>>> = {
        let mut ret = HashMap::with_capacity(MAX_ELEVATOR_NUM);
        for no in 0..MAX_ELEVATOR_NUM{
            ret.insert(no, Arc::new(Mutex::new(true)));
        }
        ret
    };
);

impl Lift {
    pub fn new(no: usize) -> Self {
        let mut r = Self::default();
        r.no = no;
        r.cur_floor = crate::util::random_floor();
        r.elevator_btns = (MIN_FLOOR..=MAX_FLOOR)
            .into_iter()
            .filter(|o| *o != 0)
            .map(|o|
                {
                    let mut btn_state = FloorBtnState::default();
                    btn_state.elevator_no = no;
                    btn_state.floor = o;
                    btn_state
                }).collect();
        r
    }

    pub fn dest_floor(&self) -> Option<TFloor> {
        // 电梯上行时，拿值最小的一个
        // 电梯下行时，拿值最大的一个
        let mut floors = self.schedule_floors
            .keys()
            .into_iter()
            .collect::<BTreeSet<_>>()
            .union(&self.stop_floors
                .keys()
                .into_iter()
                .collect::<BTreeSet<_>>())
            .into_iter()
            .filter(|o| match self.state {
                State::GoingUp | State::GoingUpSuspend => **o >= &self.cur_floor,
                State::GoingDown | State::GoingDownSuspend => **o <= &self.cur_floor,
                State::Stop => true,
                State::Maintaining => false,
            }).map(|o|**o)
            .collect::<Vec<_>>();
        floors.sort();
        if !floors.is_empty(){
            return match self.state {
                State::GoingDown | State::GoingDownSuspend => {
                    Some(floors[floors.len() - 1])
                }
                _ => {
                    Some(floors[0])
                }
            }
        }
        None
    }

    pub fn remove_floor(&mut self, floor: TFloor) -> Option<Direction> {
        self.stop_floors.remove(&floor);
        self.schedule_floors.remove(&floor).unwrap_or(None)
    }
    pub fn is_overload(&self) -> bool {
        self.persons > MAX_PERSON_CAPACITY as i32
    }

    pub fn set_persons(&mut self) {
        let n = random_person_num();
        if random_bool() {
            self.persons += n;
            self.persons = min(self.persons, MAX_PERSON_CAPACITY as i32);
        } else {
            self.persons -= n;
            self.persons = max(self.persons, 0);
        }
    }

    pub async fn suspend_one_by_one_floor(no: usize, is_wait: bool) -> AppMessage {
        let lock = LiftLocks.get(&no).unwrap();
        // 每部电梯一个锁， 从而保证消息的顺序性
        lock.lock().await;
        // 通过sleep ，模拟电梯在运行到了
        if is_wait {
            // 等待居民进出时，需要休眠更长时间
            tokio::time::sleep(
                std::time::Duration::from_millis(
                    EVERY_FLOOR_RUN_TIME_IN_MILLISECONDS as u64 + 500)
            ).await;
        } else {
            tokio::time::sleep(
                std::time::Duration::from_millis(
                    EVERY_FLOOR_RUN_TIME_IN_MILLISECONDS as u64)
            ).await;
        }

        AppMessage::ArriveByOneFloor(no)
    }

    pub async fn user_input_one_by_one_floor(no: usize) -> AppMessage {
        let lock = LiftLocks.get(&no).unwrap();
        // 每部电梯一个锁， 从而保证消息的顺序性
        lock.lock().await;
        // 通过sleep ，模拟电梯在运行到了
        tokio::time::sleep(
            std::time::Duration::from_millis(500)
        ).await;
        AppMessage::WaitUserInputFloor(no)
    }
}

impl Display for Lift {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f,
               "电梯#{}[{}层-{}人:{}]",
               self.no + 1,
               self.cur_floor,
               self.persons,
               self.state.to_string()
        )
    }
}


pub struct LiftUpDownCost {
    pub no: usize,
    pub cost: i32,
    pub cnt: usize,
}


impl PartialEq<Self> for LiftUpDownCost {
    fn eq(&self, other: &Self) -> bool {
        return self.cost == other.cost && self.cnt == other.cnt;
    }
}


impl PartialOrd for LiftUpDownCost {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        match self.cost.cmp(&other.cost) {
            Ordering::Less => Some(Ordering::Less),
            Ordering::Greater => Some(Ordering::Greater),
            Ordering::Equal => {
                // 反转
                other.cnt.partial_cmp(&self.cnt)
            }
        }
    }
}