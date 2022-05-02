use std::cmp::Ordering;
use std::collections::{BTreeSet, LinkedList};
use std::fmt::{Display, Formatter};
use crate::conf::TFloor;
use crate::floor_btn::Direction;
use crate::message::AppMessage;
use crate::state::State;
use tokio::sync::RwLock;
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
    // 调度器调度的停靠楼层
    // 上行时，schedule_floors 的元素值 > cur_floor
    // 下行时，schedule_floors 的元素值 < cur_floor
    pub schedule_floors: BTreeSet<TFloor>,
}

pub async fn run(lift: Arc<RwLock<Lift>>) -> AppMessage {
    let no = lift.read().await.no;
    AppMessage::Arrive(no)
}

impl Lift {
    pub fn new(no: usize) -> Self {
        let mut r = Self::default();
        r.no = no;
        r.cur_floor = crate::util::random_floor();
        r
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