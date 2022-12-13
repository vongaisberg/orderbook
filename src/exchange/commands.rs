use crate::order_handling::order::OrderSide;

pub trait Command<'a> {
    fn sender(&self) -> &'a str;
}

pub enum OrderCommand {
    Trade(TradeCommand),
    Cancel(CancelCommand),
}

pub struct TradeCommand {
    pub ticker: usize,
    pub side: OrderSide,
    pub volume: u64,
    pub limit: u64,
    pub immediate_or_cancel: bool,
}

pub struct CancelCommand {
   // sender: &'a str,
    pub ticker: usize,
    pub order_id: u64,
}
/*
impl<'a> Command<'a> for OrderCommand<'a> {
    fn sender(&self) -> &'a str {
        match self {
            OrderCommand::Trade(b) => b.sender,
            OrderCommand::Cancel(c) => c.sender,
        }
    }
}
*/
