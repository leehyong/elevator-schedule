use crate::conf::TFloor;
use crate::state::State;

// 电梯
pub struct Lift {
    // 电梯序号
    pub no: usize,
    pub state: State,
    pub persons: i32,
    pub cur_floor: TFloor,
}