use crate::order_handling::order::OrderSide;

pub struct PositionRecord {
    pub symbol: usize,
    pub direction: PositionDirection,
    pub volume: u64,

    // Currency paid to open the position
    pub paid_value: u64,
    pub profit: u64,

    pub pending_buy_volume: u64,
    pub pending_sell_volume: u64,
}

impl PositionRecord {
    pub fn pending_hold(mut self, side: OrderSide, volume: u64) {
        if side == OrderSide::ASK {
            self.pending_sell_volume += volume
        } else {
            self.pending_buy_volume += volume
        }
    }

    pub fn pending_release(mut self, side: OrderSide, volume: u64) {
        if side == OrderSide::ASK {
            self.pending_sell_volume -= volume
        } else {
            self.pending_buy_volume -= volume
        }
    }
}

pub enum PositionDirection {
    Long,
    Short,
}
