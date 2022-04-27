#[derive(Clone, Copy)]
pub enum Message {
    // 上到某楼层
    Up(i16),
    // 下到某楼层
    Down(i16),
    // 那台电梯正在输入楼层
    InputtingFloor(u8),
    // 那台电梯完成输入楼层
    InputtedFloor(u8, &'static str, i16),
    // 程序停止消息
    Quit,
}

impl Default for Message {
    fn default() -> Self {
        Self::Quit
    }
}