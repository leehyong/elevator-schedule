use std::fmt::{Display, Formatter};

// 电梯状态
#[derive(Ord, PartialOrd, Eq, PartialEq, Debug)]
pub enum State {
    // 电梯静止不动
    Stop,
    // 电梯上行
    GoingUp,
    // 电梯上行中在上下人
    GoingUpSuspend,
    // 电梯下行
    GoingDown,
    // 电梯下行中在上下人
    GoingDownSuspend,
    // 维护中
    Maintaining,
}

impl Default for State {
    fn default() -> Self {
        State::Stop
    }
}

impl Display for State {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        use State::*;
        write!(f, "{}", match self {
            GoingUp => "电梯上行",
            GoingDown => "电梯下行",
            GoingUpSuspend => "电梯上行进出人",
            GoingDownSuspend => "电梯下行进出人",
            Maintaining => "维护中",
            Stop => "电梯静止"
        })
    }
}