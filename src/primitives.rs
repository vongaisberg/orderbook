use std::ops::*; #[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Copy, Clone)] pub struct Value {
    val: u64,
}
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Copy, Clone)]
pub struct Volume {
    val: u64,
}
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Copy, Clone)]
pub struct Price {
    val: u64,
}

impl Value {
    pub const ZERO: Self = Self{val:0};

    pub fn new(value: u64) -> Value {
        Value { val: value }
    }
    pub fn get(&self) -> u64 {
        self.val
    }
}
impl Volume {
    pub const ZERO: Self = Self{val:0};

    pub fn new(value: u64) -> Volume {
        Volume { val: value }
    }
    pub fn get(&self) -> u64 {
        self.val
    }
}
impl Price {
    pub const ZERO: Self = Self{val:0};

    pub fn new(value: u64) -> Price {
        Price { val: value }
    }
    pub fn get(&self) -> u64 {
        self.val
    }
}

impl Mul<Price> for Volume {
    type Output = Value;

    fn mul(self, rhs: Price) -> Value {
        Value::new(self.val * rhs.val)
    }
}

impl Mul<Volume> for Price {
    type Output = Value;

    fn mul(self, rhs: Volume) -> Value {
        Value::new(self.val * rhs.val)
    }
}

impl Add<Price> for Price {
    type Output = Price;

    fn add(self, rhs: Price) -> Price {
        Price::new(self.val + rhs.val)
    }
}

impl Add<Volume> for Volume {
    type Output = Volume;

    fn add(self, rhs: Volume) -> Volume {
        Volume::new(self.val + rhs.val)
    }
}

impl Add<Value> for Value {
    type Output = Value;

    fn add(self, rhs: Value) -> Value {
        Value::new(self.val + rhs.val)
    }
}

impl Sub<Price> for Price {
    type Output = Price;

    fn sub(self, rhs: Price) -> Price {
        Price::new(self.val - rhs.val)
    }
}

impl Sub<Volume> for Volume {
    type Output = Volume;

    fn sub(self, rhs: Volume) -> Volume {
        Volume::new(self.val - rhs.val)
    }
}

impl Sub<Value> for Value {
    type Output = Value;

    fn sub(self, rhs: Value) -> Value {
        Value::new(self.val - rhs.val)
    }
}

impl AddAssign for Price {
    fn add_assign(&mut self, other: Self) {
        *self = Self {
            val: self.val + other.val,
        };
    }
}

impl AddAssign for Volume {
    fn add_assign(&mut self, other: Self) {
        *self = Self {
            val: self.val + other.val,
        };
    }
}

impl AddAssign for Value {
    fn add_assign(&mut self, other: Self) {
        *self = Self {
            val: self.val + other.val,
        };
    }
}

impl SubAssign for Price {
    fn sub_assign(&mut self, other: Self) {
        *self = Self {
            val: self.val - other.val,
        };
    }
}

impl SubAssign for Volume {
    fn sub_assign(&mut self, other: Self) {
        *self = Self {
            val: self.val - other.val,
        };
    }
}

impl SubAssign for Value {
    fn sub_assign(&mut self, other: Self) {
        *self = Self {
            val: self.val - other.val,
        };
    }
}
