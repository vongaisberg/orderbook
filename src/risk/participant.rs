use crate::exchange::asset::*;
use std::{collections::HashMap, default};

use super::risk_order::RiskOrder;

/// A market participant holding assets and position
///

#[derive(Default)]
pub struct Participant {
    /// A unique id representing the account
    pub id: u64,
    /// How much of each asset an account holds.
    pub assets: HashMap<AssetId, u64>,

    /// Standing orders of this participant
    pub orders: HashMap<(u64, u64), Box<RiskOrder>>,
}
