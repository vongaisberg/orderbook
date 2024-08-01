use std::ops::*;
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Copy, Clone)]
pub struct Value(pub u64);
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Copy, Clone)]
pub struct Volume(pub u64);

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Copy, Clone)]
pub struct Price(pub u64);

impl Value {
    pub const ZERO: Self = Self(0);

    #[inline(always)]
    pub fn get(&self) -> u64 {
        self.0
    }
}
impl Volume {
    pub const ZERO: Self = Self(0);

    #[inline(always)]
    pub fn get(&self) -> u64 {
        self.0
    }
}
impl Price {
    pub const ZERO: Self = Self(0);

    #[inline(always)]
    pub fn get(&self) -> u64 {
        self.0
    }
}

impl Deref for Volume {
    type Target = u64;
    #[inline(always)]
    fn deref(&self) -> &u64 {
        &self.0
    }
}

impl DerefMut for Volume {
    #[inline(always)]
    fn deref_mut(&mut self) -> &mut u64 {
        &mut self.0
    }
}

impl Deref for Value {
    type Target = u64;
    #[inline(always)]
    fn deref(&self) -> &u64 {
        &self.0
    }
}

impl DerefMut for Value {
    #[inline(always)]
    fn deref_mut(&mut self) -> &mut u64 {
        &mut self.0
    }
}

impl Deref for Price {
    type Target = u64;
    #[inline(always)]
    fn deref(&self) -> &u64 {
        &self.0
    }
}

impl DerefMut for Price {
    #[inline(always)]
    fn deref_mut(&mut self) -> &mut u64 {
        &mut self.0
    }
}

impl Mul<Price> for Volume {
    type Output = Value;
    #[inline(always)]
    fn mul(self, rhs: Price) -> Value {
        Value(self.0 * rhs.0)
    }
}

impl Mul<u64> for Volume {
    type Output = Self;
    #[inline(always)]
    fn mul(self, rhs: u64) -> Self {
        Self(self.0 * rhs)
    }
}
impl Mul<u64> for Value {
    type Output = Self;
    #[inline(always)]
    fn mul(self, rhs: u64) -> Self {
        Self(self.0 * rhs)
    }
}

impl Mul<u64> for Price {
    type Output = Self;
    #[inline(always)]
    fn mul(self, rhs: u64) -> Self {
        Self(self.0 * rhs)
    }
}

impl Mul<Volume> for Price {
    type Output = Value;
    #[inline(always)]
    fn mul(self, rhs: Volume) -> Value {
        Value(self.0 * rhs.0)
    }
}

impl Add<Price> for Price {
    type Output = Price;
    #[inline(always)]
    fn add(self, rhs: Price) -> Price {
        Price(self.0 + rhs.0)
    }
}

impl Add<Volume> for Volume {
    type Output = Volume;
    #[inline(always)]
    fn add(self, rhs: Volume) -> Volume {
        Volume(self.0 + rhs.0)
    }
}

impl Add<Value> for Value {
    type Output = Value;
    #[inline(always)]
    fn add(self, rhs: Value) -> Value {
        Value(self.0 + rhs.0)
    }
}

impl Sub<Price> for Price {
    type Output = Price;
    #[inline(always)]
    fn sub(self, rhs: Price) -> Price {
        Price(self.0 - rhs.0)
    }
}

impl Sub<Volume> for Volume {
    type Output = Volume;
    #[inline(always)]
    fn sub(self, rhs: Volume) -> Volume {
        Volume(self.0 - rhs.0)
    }
}

impl Sub<Value> for Value {
    type Output = Value;
    #[inline(always)]
    fn sub(self, rhs: Value) -> Value {
        Value(self.0 - rhs.0)
    }
}

impl AddAssign for Price {
    #[inline(always)]
    fn add_assign(&mut self, other: Self) {
        *self = Self(self.0 + other.0);
    }
}

impl AddAssign for Volume {
    #[inline(always)]
    fn add_assign(&mut self, other: Self) {
        *self = Self(self.0 + other.0);
    }
}

impl AddAssign for Value {
    #[inline(always)]
    fn add_assign(&mut self, other: Self) {
        *self = Self(self.0 + other.0);
    }
}

impl SubAssign for Price {
    #[inline(always)]
    fn sub_assign(&mut self, other: Self) {
        *self = Self(self.0 - other.0);
    }
}

impl SubAssign for Volume {
    #[inline(always)]
    fn sub_assign(&mut self, other: Self) {
        *self = Self(self.0 - other.0);
    }
}

impl SubAssign for Value {
    #[inline(always)]
    fn sub_assign(&mut self, other: Self) {
        *self = Self(self.0 - other.0);
    }
}
