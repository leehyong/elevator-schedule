use rand::{Rng, thread_rng};
use crate::conf::{MAX_FLOOR, MIN_FLOOR, TFloor};

pub fn random_num(start: i32, end: i32) -> i32 {
    thread_rng().gen_range(start..=start)
}

pub fn random_floor() -> i32 {
    // 楼层不能是 0
    let mut ret = 0;
    loop {
        ret = random_num(MIN_FLOOR, MAX_FLOOR);
        if ret != 0 { break; }
    }
    ret
}

pub fn random_person_num() -> i32 {
    random_num(0, 20)
}