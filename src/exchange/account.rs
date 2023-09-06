use crate::exchange::asset::*;
use crate::primitives::*;
use std::collections::HashMap;

/// A market participant holding assets and cash
pub struct Account {
    /// A unique id representing the account
    pub id: u64,
    /// How much of each asset an account holds.
    /// Not every Asset that exists on the exchange must be present in this HashMap, it will be dynamically added once the Account receives it.
    pub assets: HashMap<Asset, Volume>,
    /// How much cash the account holds
    pub cash: Value,
}
