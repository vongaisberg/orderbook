use crate::order_handling::order::OrderSide;

use super::asset::Symbol;

pub trait Command {
    fn sender(&self) -> &str;
}

#[derive(Copy, Clone, Debug)]
pub enum OrderCommand {
    Trade(TradeCommand),
    Cancel(CancelCommand),
}

#[derive(Copy, Clone, Debug)]
pub struct TradeCommand {
    pub id: u64,
    pub participant_id: u64,
    pub symbol: u64,
    pub side: OrderSide,
    pub volume: u64,
    pub limit: u64,
    pub immediate_or_cancel: bool,
}

#[derive(Copy, Clone, Debug)]
pub struct CancelCommand {
    pub symbol: u64,
    pub order_id: u64,
    pub participant_id: u64,
}

impl TradeCommand {
    /// Highest amount of value that could be spent (asset_id, value)
    pub fn pessimistic(&self, symbol: &Symbol) -> (usize, u64) {
        match self.side {
            OrderSide::BID => (symbol.base_asset, self.volume * self.limit),
            OrderSide::ASK => (symbol.quote_asset, self.volume),
        }
    }
    /// Highest amount of value that could be spent (asset_id, value)
    pub fn historic_pessimistic(side: OrderSide, limit: u64, symbol: &Symbol, volume: u64) -> (usize, u64) {
        match side {
            OrderSide::BID => (symbol.base_asset, volume * limit),
            OrderSide::ASK => (symbol.quote_asset, volume),
        }
    }
}
