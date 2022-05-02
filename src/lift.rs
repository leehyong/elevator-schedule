use std::cmp::Ordering;
use std::collections::{BTreeSet, HashMap, LinkedList};
use std::fmt::{Display, Formatter};
use crate::conf::{EVERY_FLOOR_RUN_TIME_IN_MILLISECONDS, MAX_ELEVATOR_NUM, TFloor};
use crate::floor_btn::Direction;
use crate::message::AppMessage;
use crate::state::State;
use tokio::sync::Mutex;
use std::sync::Arc;


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
    pub can_click_btn: bool,
    // 调度器调度的停靠楼层
    // 上行时，schedule_floors 的元素值 > cur_floor
    // 下行时，schedule_floors 的元素值 < cur_floor
    pub schedule_floors: BTreeSet<TFloor>,
}
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

impl Lift {
    pub fn new(no: usize) -> Self {
        let mut r = Self::default();
        r.no = no;
        r.cur_floor = crate::util::random_floor();
        r
    }

    async fn suspend_one_by_one_floor(no: usize, cur_floor: TFloor, dest_floor: TFloor){
        let lock = LiftSuspendLocks.get(&no).unwrap();
        // 每部电梯一个锁， 从而保证消息的顺序性
        lock.lock().await;
        // 通过sleep ，模拟电梯在运行到了
        tokio::time::sleep(
            std::time::Duration::from_millis(
                ((dest_floor - cur_floor).abs() as u64)) * EVERY_FLOOR_RUN_TIME_IN_MILLISECONDS
        ).await;
    }

    pub async fn schedule_suspend_one_by_one_floor(no: usize, cur_floor: TFloor, dest_floor: TFloor) -> AppMessage {
        Self::suspend_one_by_one_floor(no, cur_floor, dest_floor).await;
        AppMessage::ScheduleArrive(no, dest_floor)
    }

    pub async fn user_input_one_by_one_floor(no: usize, cur_floor: TFloor, dest_floor: TFloor) -> AppMessage {
        let lock = LiftUserInputLocks.get(&no).unwrap();
        // 每部电梯一个锁， 从而保证消息的顺序性
        lock.lock().await;
        // 通过sleep ，模拟电梯在运行到了
        tokio::time::sleep(
            std::time::Duration::from_millis(500)
        ).await;
        AppMessage::ScheduleWaitUserInputFloor(no, dest_floor)
    }

    pub async fn schedule_user_input_one_by_one_floor(no: usize, cur_floor: TFloor, dest_floor: TFloor) -> AppMessage {
        Self::user_input_one_by_one_floor(no, cur_floor, dest_floor).await;
        AppMessage::ScheduleWaitUserInputFloor(no, dest_floor)
    }

    pub async fn running_suspend_one_by_one_floor(no: usize, cur_floor: TFloor, dest_floor: TFloor) -> AppMessage {
        Self::suspend_one_by_one_floor(no, cur_floor, dest_floor).await;
        AppMessage::RunningArrive(no, dest_floor)
    }

    pub async fn running_user_input_one_by_one_floor(no: usize, cur_floor: TFloor, dest_floor: TFloor) -> AppMessage {
        Self::user_input_one_by_one_floor(no, cur_floor, dest_floor).await;
        AppMessage::RunningWaitUserInputFloor(no, dest_floor)
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