use crate::exchange_settings::ExchangeSettings;


pub fn risk_router(settings: &ExchangeSettings, participant_id: &u64) -> usize{

    (participant_id % settings.risk_engine_shards) as usize

}
