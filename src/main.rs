#[macro_use]
extern crate lazy_static;
pub mod floor;
pub mod elevator;
pub mod state;
pub mod scheduler;
pub mod conf;
pub mod message;
pub mod up_down_elevator_floor;
pub mod app;
pub mod style;
pub mod floor_btn;
pub mod icon;


use std::io::{Read, Write};
use scheduler::Scheduler;

fn main() {
   // let mut schedule = Scheduler::new();
   //  schedule.run();
    app::run_window()
    // for x in "尾是发发发fy̆发发发".chars(){
    //     println!("{}", x);
    // }
    //     println!("{}", "尾是发发发fy̆发发发".chars().take(2).next().unwrap());
    // let four: u32 = "4gg".parse().unwrap();
    // println!("{}", four);
    // print!("请输入姓名:>");
    // std::io::stdout().flush().unwrap();
    // let mut input = String::new();
    // std::io::stdin().read_line(&mut input).unwrap();
    // println!("输入的是：{}", input)
}

