use std::time::Duration;

use super::asset::Symbol;

#[derive(PartialEq, Eq, Default, Clone)]
pub struct ExchangeSettings {
    pub symbols: Vec<Symbol>,
    pub risk_engine_shards: u64,

    //Technical parameters
    pub db_sync_speed: Duration,
    pub db_min_recv_timeout: Duration, //To prevent frequent contex switches
}
