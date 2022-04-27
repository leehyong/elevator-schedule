use std::cmp::Ordering;

pub enum Floor {
    // 在某楼层上楼
    Up(i16),
    // 在某楼层下楼
    Down(i16),
}

impl Floor {
    pub fn value(&self) -> i16 {
        use Floor::*;
        match *self {
            Up(v) => v,
            Down(v) => v,
        }
    }
    fn inner_cmp(&self, other: &Self) -> Ordering {
        use Floor::*;
        match self {
            Up(v) => {
                match other {
                    Up(o) => std::cmp::Reverse(v).cmp(&std::cmp::Reverse(o)),
                    _ => unreachable!()
                }
            }
            Down(v) => {
                match other {
                    Down(o) => v.cmp(o),
                    _ => unreachable!()
                }
            }
        }
    }
}

impl Eq for Floor {}

impl PartialEq<Self> for Floor {
    fn eq(&self, other: &Self) -> bool {
        use Floor::*;
        self.value() == other.value()
    }
}

impl PartialOrd<Self> for Floor {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.inner_cmp(other))
    }
}

impl Ord for Floor {
    fn cmp(&self, other: &Self) -> Ordering {
        self.inner_cmp(other)
    }
}
