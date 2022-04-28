use std::cell::RefCell;
use std::collections::{HashSet, HashMap, BinaryHeap};
use std::io::Read;
use std::sync::{Arc, RwLock};
use std::thread;
use std::sync::mpsc::{Sender, channel, Receiver};
use std::thread::{JoinHandle, Thread};
use std::cmp::Reverse;
use std::option::Option::Some;
use crate::conf::*;
use crate::elevator::Elevator;
use crate::message::Message;
use crate::state::State;
use crate::upDownElevatorFloor::*;


// 电梯算法调度器
pub struct Scheduler {
    rxOneToMany: Receiver<Message>,
    // 从Scheduler接收电梯里的消息
    cxOneToMany: Sender<Message>,
    senders: HashMap<u8, Sender<Message>>,
    // 从多个电梯里往Scheduler发送消息
    handles: Vec<JoinHandle<()>>,

}

lazy_static! {
   // pub static ref AllElevatorsMap: Arc<RwLock<HashMap<u8, Elevator>>> = Arc::new(RwLock::new(HashMap::with_capacity(MAX_ELEVATOR_NUM)));
   pub static ref AllElevatorsMap: HashMap<u8, Elevator> = {
       let mut ret = HashMap::with_capacity(MAX_ELEVATOR_NUM);
        for x in 0..MAX_ELEVATOR_NUM{
            ret.insert(x as u8, Elevator::new(x as u8));
        }
        ret
    };
}

impl Scheduler {
    pub fn new() -> Self {
        let (cx, rx) = channel();
        Self {
            rxOneToMany: rx,
            cxOneToMany: cx,
            senders: HashMap::with_capacity(MAX_ELEVATOR_NUM),
            handles: Vec::with_capacity(MAX_ELEVATOR_NUM),
        }
    }
    fn help_hint() {
        println!("请按照以下格式以运行电梯,梯楼层范围:[{}~{}]，不在有效楼层，视为无效输入!", MIN_FLOOR, MAX_FLOOR);
        println!("\t10 上、10 up、10 u, 表示要从10楼上楼");
        println!("\t42 下、42 down、42 d, 表示要从42楼下楼");
        println!("\texit、e、退出, 表示推出当前程序\n");
        println!("\t一行可以有多个输入，用 ','或者'，'分隔, 表示同一时间有多个人想要乘电梯\n");
    }

    fn _run(&self) {
        Self::help_hint();
        loop {
            self.response_elevator_msg();
            self.schedule_elevator();
            Elevator::sleep_run();
        }
        // 等待子线程运行完
        for i in 0..MAX_ELEVATOR_NUM {
            self.handles[i].join();
        }
    }

    fn schedule_elevator(&self) {
        // 调度电梯去接居民
        let (upstairs, downstairs) = Self::parse_input();
        if upstairs.is_empty() && downstairs.is_empty() {
            // 无事可做
            return;
        }
        println!("上 {:?}\n 下 {:?}", &upstairs, &downstairs);
        // 存放上行接人的电梯
        let mut ups = Vec::with_capacity(4);
        // 存放下行接人的电梯
        let mut downs = Vec::with_capacity(4);
        // 判定哪些电梯能去接人
        for elevator in AllElevatorsMap.values() {
            // 能上行接人的电梯就不能下行接人；反之亦然.
            // 所以这里是 if ... else if,而不是 if ... if
            if Self::can_elevator_up(&upstairs, elevator) {
                ups.push(elevator);
            } else if Self::cam_elevator_down(&downstairs, elevator) {
                downs.push(elevator);
            }
        }
        println!("电梯上行 {:?}\n 电梯下行 {:?}", &ups, &downs);

        // 调度上行电梯
        self.arrange_up_elevator(&upstairs, &ups);
        // 调度下行电梯
        self.arrange_down_elevator(&downstairs, &downs);
    }

    fn arrange_up_elevator(&self, stairs: &[i16], elevators: &[&Elevator]) {
        let mut bh = BinaryHeap::with_capacity(stairs.len() + elevators.len());
        // 使用 Reverse 构造大顶堆
        for stair in stairs {
            bh.push(UpDownElevatorFloor {
                floor: *stair,
                typ: FloorType::Person,
            })
        }
        for elevator in elevators {
            let floor = elevator.meta.read().unwrap().cur_floor;
            bh.push(UpDownElevatorFloor {
                floor,
                typ: FloorType::Elevator(elevator.no),
            })
        }
        let mut ups = vec![];
        while let Some(item) = bh.pop() {
            match item.typ {
                FloorType::Person => {
                    ups.push(item.floor);
                }
                FloorType::Elevator(no) => {
                    let cx = self.senders.get(&no).unwrap();
                    // 一次 发送全部
                    cx.send(Message::Ups(ups.clone())).unwrap();
                }
            }
        }
    }

    fn arrange_down_elevator(&self, stairs: &[i16], elevators: &[&Elevator]) {
        let mut bh = BinaryHeap::with_capacity(stairs.len() + elevators.len());
        // 使用 Reverse 构造小顶堆
        for stair in stairs {
            bh.push(Reverse(UpDownElevatorFloor {
                floor: *stair,
                typ: FloorType::Person,
            }))
        }
        for elevator in elevators {
            let floor = elevator.meta.read().unwrap().cur_floor;
            bh.push(Reverse(UpDownElevatorFloor {
                floor,
                typ: FloorType::Elevator(elevator.no),
            }))
        }
        let mut ups = vec![];
        while let Some(item) = bh.pop() {
            match item.0.typ {
                FloorType::Person => {
                    ups.push(item.0.floor);
                }
                FloorType::Elevator(no) => {
                    let cx = self.senders.get(&no).unwrap();
                    // 一次 发送全部
                    cx.send(Message::Downs(ups.clone())).unwrap();
                }
            }
        }
    }


    fn can_elevator_up(stairs: &[i16], elevator: &Elevator) -> bool {
        // 返回某电梯是否能上去接人
        let meta = elevator.meta.read().unwrap();
        use State::*;
        match meta.state {
            // 维护中、下行的、下行中在上下人的肯定不能上去接人
            Maintaining | GoingDown | GoingDownSuspend => return false,
            Stop | GoingUp | GoingUpSuspend => {
                meta.persons + 1 < MAX_PERSON_CAPACITY && // 再上一人不超员
                    stairs // 能上行肯定是，居民所在楼层不比电梯楼层低
                        .iter()
                        .filter(|item| **item >= meta.cur_floor)
                        .count() > 0
            }
        }
    }
    fn cam_elevator_down(stairs: &[i16], elevator: &Elevator) -> bool {
        // 返回某电梯是否能下去接人
        let meta = elevator.meta.read().unwrap();
        use State::*;
        match meta.state {
            // 维护中、上行的、上行中在上下人的肯定不能下去接人
            Maintaining | GoingUp | GoingUpSuspend => return false,
            Stop | GoingDown | GoingDownSuspend => {
                meta.persons + 1 < MAX_PERSON_CAPACITY &&                // 再上一人不超员
                    stairs   // 能下行肯定是，居民所在楼层不比电梯楼层高
                        .iter()
                        .filter(|item| **item <= meta.cur_floor)
                        .count() > 0
            }
        }
    }

    fn response_elevator_msg(&self) {
        use Message::*;
        if let Ok(msg) = self.rxOneToMany.try_recv() {
            match msg {
                InputtingFloor(i) => {
                    println!("电梯{}，有人正在输入楼层...", i);
                    let mut floor = 0i16;
                    let mut input = String::with_capacity(10);
                    loop {
                        std::io::stdin().read_line(&mut input).unwrap();
                        if let Ok(v) = input.parse() {
                            if v >= MIN_FLOOR && v <= MAX_FLOOR {
                                floor = v;
                                break;
                            }
                        }
                        println!("请重新输入楼层,范围:{}~{}！", MIN_FLOOR, MAX_FLOOR);
                    }
                    let cx = self.senders.get(&i).unwrap();
                    cx.send(InputtedFloor(i, floor)).unwrap();
                }
                _ => {}
            }
        }
    }

    const fn stair_capacity() -> usize {
        // 根据经验每部同时乘电梯的楼层数一般不会超过半数，
        // 所有乘电梯的楼层的容量
        ((((MAX_FLOOR - MIN_FLOOR) >> 1 )as usize) * MAX_ELEVATOR_NUM) as usize
    }

    fn parse_input() -> (Vec<i16>, Vec<i16>) {
        let mut input = String::new();
        {
            std::io::stdin().read_line(&mut input).unwrap();
        }
        // HashSet 去重
        let mut upstairs = HashSet::with_capacity(Self::stair_capacity());
        let mut downstairs = HashSet::with_capacity(Self::stair_capacity());
        for item in input.split(|x| x == ',' || x == '，') {
            // 去掉首尾空白
            let s1 = item.trim();
            if s1.is_empty() {
                continue;
            }
            // 避免一次性分配过多内存
            let mut s = String::with_capacity(4);
            let mut op = String::with_capacity(4); // down 是最多的字符数
            let mut digit_done = false;
            for ch in s1.chars() {
                if !digit_done && ch.is_digit(10) {
                    s.push(ch);
                } else {
                    digit_done = true;
                    if !ch.is_whitespace() {
                        op.push(ch);
                    }
                }
            }
            match s.parse() {
                Ok(v) => {
                    if v < MIN_FLOOR || v > MAX_FLOOR {
                        println!("楼层范围不对，请检查：[{}~{}]", MIN_FLOOR, MAX_FLOOR);
                    } else {
                        match op.as_str() {
                            "u" | "U" | "up" | "上" => {
                                upstairs.insert(v);
                            }
                            "d" | "D" | "down" | "下" => {
                                downstairs.insert(v);
                            }
                            _ => {
                                println!("不支持的操作：{}", s1)
                            }
                        }
                    }
                }
                Err(e) => {
                    println!("输入格式错误：{}", e)
                }
            }
            // let floor = s1.chars()[..end];
        }
        (upstairs.iter().map(|o|*o).collect(),
         downstairs.iter().map(|o|*o).collect())
    }

    pub fn run(&mut self) {
        for i in (0..MAX_ELEVATOR_NUM).rev() {
            let cx = self.cxOneToMany.clone();
            let (sender, receiver) = channel();
            let u = i as u8;
            self.senders.insert(u, sender);
            self.handles.push(thread::spawn(
                move || {
                    AllElevatorsMap.get(&u)
                        .unwrap()
                        .run(receiver, cx);
                }));
        }
        // 主线程运行调度器程序
        self._run();
    }
}
