#[macro_use]
extern crate lazy_static;
pub mod floor;
pub mod elevator;
pub mod state;
pub mod scheduler;
pub mod conf;
pub mod message;


use scheduler::Scheduler;

fn main() {
   let mut schedule = Scheduler::new();
    schedule.run();
    // for x in "尾是发发发fy̆发发发".chars(){
    //     println!("{}", x);
    // }
    //     println!("{}", "尾是发发发fy̆发发发".chars().take(2).next().unwrap());
    // let four: u32 = "4gg".parse().unwrap();
    // println!("{}", four);
}

