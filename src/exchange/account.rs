use crate::exchange::asset::*;
use std::collections::HashMap;

/// A market participant holding assets and cash
pub struct Account {
    /// A unique id representing the account
    pub id: u64,
    /// How much of each asset an account holds.
    pub assets: Box<[Asset]>,
    /// How much cash the account holds
    pub cash: u64,
}
