use serde::{Deserialize, Serialize};

/// Events triggered by the matching engine and sent to the risk engines

#[derive(Clone, Debug, Serialize, Deserialize, bitcode::Encode, bitcode::Decode)]
pub enum MatchingEngineEvent {
    ///How much volume was filled and how much Value was payed for it
    /// order_id, volume, value
    Filled(u64, u64, u64),
    /// id
    Canceled(u64),
}

/// Events triggered by the matching engine and sent to the DB,
/// 
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum DbEvent {
    Trade(Trade),
}

/// If volume is positive, the taker receives volume and pays value,
/// If volume is negative, the taker receives that negative volume and pays that (hopefully negative) value
/// 
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Trade {
    symbol: usize,
    volume: i64,
    value: i64,
    taker_participant: usize,
    maker_participant: usize,
}
