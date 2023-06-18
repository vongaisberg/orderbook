use std::collections::HashMap;

use super::asset::Symbol;

#[derive(PartialEq, Eq, Default, Clone)]
pub struct ExchangeSettings {
    pub symbols: Vec<Symbol>,
    pub risk_engine_shards: u64,
}
