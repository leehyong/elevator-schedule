use std::cell::RefCell;
use std::collections::HashMap;
use std::io::Read;
use std::sync::{Arc, RwLock};
use std::thread;
use std::sync::mpsc::{Sender, channel, Receiver};
use std::thread::{JoinHandle, Thread};
use crate::conf::*;
use crate::elevator::Elevator;
use crate::message::Message;

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
   pub static ref AllElevatorsMap: Arc<RwLock<HashMap<u8, Elevator>>> = Arc::new(RwLock::new(HashMap::with_capacity(MAX_ELEVATOR_NUM)));
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
        println!("请按照以下格式以运行电梯,梯楼层范围:{}~{}，不在有效楼层，视为无效输入!", MIN_FLOOR, MAX_FLOOR);
        println!("\t10 上、10 up、10 u, 表示要从10楼上楼");
        println!("\t42 下、42 down、42 d, 表示要从42楼下楼");
        println!("\texit、e、退出, 表示推出当前程序\n");
        println!("\t一行可以有多个输入，用 ',', ','分隔, 表示同一时间有多个人想要乘电梯\n");

    }

    fn run_schedule(&self) {
        Self::help_hint();
        loop {
            self.response_elevator_msg();
            let (upstairs, downstairs) = Self::parse_input();
            Elevator::sleep_run();
        }
        // 等待子线程运行完
        for i in 0..MAX_ELEVATOR_NUM {
            self.handles[i].join();
        }
    }


    fn response_elevator_msg(&self) {
        use Message::*;
        if let Ok(msg) = self.rxOneToMany.try_recv() {
            match msg {
                InputtedFloor(i, direction, floor) => {
                    println!("电梯{}，有人输入了{}楼:{}", i, direction, floor)
                }
                InputtingFloor(i) => {
                    println!("电梯{}，有人正在输入楼层", i)
                }
                _ => {}
            }
        }
    }

    fn parse_input() -> (Vec<i16>, Vec<i16>) {
        let mut input = String::new();
        {
            std::io::stdin().read_line(&mut input).unwrap();
        }
        let mut upstairs = vec![];
        let mut downstairs = vec![];
        for item in input.split(|x| x == ',' || x == '，') {
            let mut end = 0usize;
            // 去掉首尾空白
            let s1 = item.trim();
            if s1.is_empty() {
                continue;
            }
            let mut s = String::new();
            let mut op = String::new();
            let mut digit_done = false;
            for ch in s1.chars(){
                if !digit_done && ch.is_digit(10){
                    s .push(ch);
                }else{
                    digit_done = true;
                    if !ch.is_whitespace(){
                        op .push(ch);
                    }
                }
            }
            println!("{} - {}", s.parse::<i16>().unwrap_or(-10), op);
            match s.parse() {
                Ok(v) => {
                    match op.as_str(){
                        "u" | "U" | "up" | "上" => {
                            upstairs.push(v);
                        }
                        "d" | "D" | "down" | "下" => {
                            downstairs.push(v);
                        }
                        _ => {
                            println!("不支持的操作：{}", s1)
                        }
                    }
                }
                Err(e) => {
                    println!("输入格式错误：{}", e)
                }
            }
            // let floor = s1.chars()[..end];
        }
        (upstairs, downstairs)
    }

    pub fn run(&mut self) {
        for i in (0..MAX_ELEVATOR_NUM).rev() {
            let cx = self.cxOneToMany.clone();
            let (sender, receiver) = channel();
            let u = i as u8;
            self.senders.insert(u, sender);
            self.handles.push(thread::spawn(
                move || {
                    let u = i as u8;
                    let ele = Elevator::new(u);
                    {
                        let mut lock = AllElevatorsMap.write().unwrap();
                        lock.insert(u, ele);
                    }
                    AllElevatorsMap
                        .read()
                        .unwrap()
                        .get(&u)
                        .unwrap()
                        .run(receiver, cx);
                }));
        }
        // 主线程运行调度器程序
        self.run_schedule();
    }
}
