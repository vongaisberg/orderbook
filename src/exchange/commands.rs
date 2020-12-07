use crate::order_handling::order::OrderSide;
use crate::primitives::*;

pub trait Command<'a> {
    fn sender(&self) -> &'a str;
}

pub enum OrderCommand<'a> {
    Trade(TradeCommand<'a>),
    Cancel(CancelCommand<'a>),
    
}

pub struct TradeCommand<'a> {
    sender: &'a str,
    pub ticker: &'a str,
    pub side: OrderSide,
    pub volume: Volume,
    pub limit: Price,
    pub immediate_or_cancel: bool,
}

pub struct CancelCommand<'a> {
    sender: &'a str,
    pub ticker: &'a str,
    pub order_id: u64,
}
impl<'a> Command<'a> for OrderCommand<'a> {
    fn sender(&self) -> &'a str {
        match self {
            OrderCommand::Trade(b) => b.sender,
            OrderCommand::Cancel(c) => c.sender,
        }
    }
}
