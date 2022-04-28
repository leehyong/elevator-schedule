
// 最大电梯部数
pub const MAX_ELEVATOR_NUM: usize = 4;
// 最大电梯楼层
pub const MAX_FLOOR: i16 = 40;
// 最小电梯楼层数
pub const MIN_FLOOR: i16 = -4;
// 最大承载人数
pub const MAX_PERSON_CAPACITY: u8 = 18;

// 上下人等待时间, 单位：豪秒
pub const SUSPEND_WAIT_IN_MILLISECONDS: u32 = 5 * 100;
// 电梯每层的运行时间, 单位：豪秒
pub const EVERY_FLOOR_RUN_TIME_IN_MILLISECONDS: u32 = 2 * 100;
// 电梯运行过程中的休眠时间, 单位：豪秒
pub const ELEVATOR_SLEEP_TIME_IN_MILLISECONDS: u32 = 1 * 100;
