#[derive(Clone)]
pub enum Message {
    // 上到某楼层
    Up(i16),
    // 上到某些楼层
    Ups(Vec<i16>),
    // 下到某楼层
    Down(i16),
    // 下到某些楼层
    Downs(Vec<i16>),
    // 哪台电梯正在输入楼层
    InputtingFloor(u8),
    // 哪台电梯完成输入楼层
    InputtedFloor(u8, i16),
    // 程序停止消息
    Quit,
}
#[derive(Clone, Copy, Debug, PartialOrd, PartialEq)]
pub enum UiMessage{
    Noop,
    SliderChange(i16),
    ClickedBtnPlus,
    ClickedBtnSubtract,
    ClickedBtnUp,
    ClickedBtnDown,
    ClickedBtnFloor(u8,i16)
}

impl Default for Message {
    fn default() -> Self {
        Self::Quit
    }
}