use crate::{order_handling::order::OrderSide, exchange::commands::TradeCommand};

pub struct RiskOrder {
    pub limit: u64,
    pub volume: u64,
    pub side: OrderSide,
    pub id: u64,
}

impl RiskOrder {
    pub fn new(
        id: u64,
        participant_id: u64,
        limit: u64,
        volume: u64,
        side: OrderSide,
    ) -> RiskOrder {
        RiskOrder {
            limit,
            volume,
            side,
            id,
        }
    }
}

impl From<TradeCommand> for RiskOrder {
    fn from(value: TradeCommand) -> Self {
        Self::new(
            value.id,
            value.participant_id,
            value.limit,
            value.volume,
            value.side,
        )
    }
}
