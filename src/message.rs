use crate::conf::TFloor;

#[derive(Clone)]
pub enum Message {
    // 上到某楼层
    Up(TFloor),
    // 上到某些楼层
    Ups(Vec<TFloor>),
    // 下到某楼层
    Down(TFloor),
    // 下到某些楼层
    Downs(Vec<TFloor>),
    // 哪台电梯正在输入楼层
    InputtingFloor(TFloor),
    // 哪台电梯完成输入楼层
    InputtedFloor(usize, TFloor),
    // 程序停止消息
    Quit,
}

#[derive(Clone, Copy, Debug, PartialOrd, PartialEq)]
pub enum UiMessage {
    Noop,
    SliderChange(TFloor),
    SliderRelease(TFloor),
    ClickedBtnPlus,
    ClickedBtnSubtract,
    ClickedBtnUp,
    ClickedBtnDown,
    ClickedBtnFloor(usize, TFloor),
}

impl Default for Message {
    fn default() -> Self {
        Self::Quit
    }
}