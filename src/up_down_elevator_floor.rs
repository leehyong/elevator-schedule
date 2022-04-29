use std::cmp::Ordering;
use std::fmt::{Display, Formatter};

#[derive(PartialOrd, PartialEq, Debug)]
pub enum FloorType {
    Person,
    Elevator(u8),
}

impl Default for FloorType {
    fn default() -> Self {
        Self::Person
    }
}

#[derive(Debug)]
pub struct UpDownElevatorFloor {
    pub floor: i16,
    pub typ: FloorType,
}

impl UpDownElevatorFloor  {
    fn inner_cmp(&self, other: &Self) ->Ordering{
        if self.floor < other.floor {
            Ordering::Less
        } else if self.floor > other.floor {
            Ordering::Greater
        } else {
            //  保证 楼层 相同的时候，Elevator类型的对象 比 Person 类型的对象 大
            // 从而保证在一组数据在升序时， Person 类型的对象 比 Elevator类型的对象 靠前
            use FloorType::*;
            match self.typ {
                Person => {
                    match other.typ {
                        Person => Ordering::Equal,
                        Elevator(_) => Ordering::Less
                    }
                }
                Elevator(v1) => {
                    match other.typ {
                        Person => Ordering::Greater,
                        Elevator(v2) => v1.partial_cmp(&v2).unwrap()
                    }
                }
            }
        }
    }
}

impl PartialOrd for UpDownElevatorFloor {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.inner_cmp(other))
    }
}

impl PartialEq<Self> for UpDownElevatorFloor {
    fn eq(&self, other: &Self) -> bool {
        self.floor == other.floor && self.typ == other.typ
    }
}

impl Eq for UpDownElevatorFloor {}

impl Ord for UpDownElevatorFloor {
    fn cmp(&self, other: &Self) -> Ordering {
        self.inner_cmp(other)
    }
}


impl Display for FloorType {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f , "{}", match self {
            FloorType::Elevator(v) => format!("Elevator({})", v),
            FloorType::Person => "Person".to_string()
        } )
    }
}


impl Display for UpDownElevatorFloor {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f , "{}-{}", self.floor, self.typ)
    }
}
