use std::ops::Neg;

use serde::{Serialize, Deserialize};

#[derive(Debug, PartialEq, Eq, Clone, Copy, Serialize, Deserialize, bitcode::Encode, bitcode::Decode)]
pub enum OrderSide {
    ASK,
    BID,
}

impl Neg for OrderSide {
    type Output = Self;
    fn neg(self) -> Self {
        match self {
            Self::ASK => Self::BID,
            Self::BID => Self::ASK,
        }
    }
}