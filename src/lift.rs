use std::collections::LinkedList;
use std::fmt::{Display, Formatter};
use crate::conf::TFloor;
use crate::state::State;

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
    pub stop_floors: LinkedList<TFloor>,
    // 调度器调度的停靠楼层
    // 上行时，schedule_floors 的元素值 > cur_floor
    // 下行时，schedule_floors 的元素值 < cur_floor
    pub schedule_floors: LinkedList<TFloor>,
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

pub fn schedule() {}