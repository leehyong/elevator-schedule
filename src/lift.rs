use std::cmp::Ordering;
use std::collections::{BTreeSet, HashMap, LinkedList};
use std::fmt::{Display, Formatter};
use crate::conf::{EVERY_FLOOR_RUN_TIME_IN_MILLISECONDS, MAX_ELEVATOR_NUM, TFloor};
use crate::floor_btn::Direction;
use crate::message::AppMessage;
use crate::state::State;
use tokio::sync::Mutex;
use std::sync::Arc;

lazy_static!(
    static ref LiftSuspendLocks: HashMap<usize,Arc<Mutex<bool>>> = {
        let mut ret = HashMap::with_capacity(MAX_ELEVATOR_NUM);
        for no in 0..MAX_ELEVATOR_NUM{
            ret.insert(no, Arc::new(Mutex::new(true)));
        }
        ret
    };

    static ref LiftUserInputLocks: HashMap<usize,Arc<Mutex<bool>>> = {
        let mut ret = HashMap::with_capacity(MAX_ELEVATOR_NUM);
        for no in 0..MAX_ELEVATOR_NUM{
            ret.insert(no, Arc::new(Mutex::new(true)));
        }
        ret
    };
);

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
    pub stop_floors: BTreeSet<TFloor>,
    // 调度器调度的停靠楼层
    // 上行时，schedule_floors 的元素值 > cur_floor
    // 下行时，schedule_floors 的元素值 < cur_floor
    pub schedule_floors: BTreeSet<TFloor>,
}


impl Lift {
    pub fn new(no: usize) -> Self {
        let mut r = Self::default();
        r.no = no;
        r.cur_floor = crate::util::random_floor();
        r
    }

    pub async fn suspend_one_by_one_floor(no: usize, cur_floor: TFloor, dest_floor: TFloor) -> AppMessage {
        let lock = LiftSuspendLocks.get(&no).unwrap();
        lock.lock
        tokio::time::sleep(
            std::time::Duration::from_millis(
                ((dest_floor - cur_floor).abs() as u64)) * EVERY_FLOOR_RUN_TIME_IN_MILLISECONDS
        ).await;
        AppMessage::ArriveSuspend(no, dest_floor)
    }
    pub async fn run_one_by_one_floor2(no: usize, cur_floor: TFloor, dest_floor: TFloor) -> AppMessage {
        AppMessage::Arrive(no, dest_floor)
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