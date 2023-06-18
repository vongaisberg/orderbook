use crate::exchange::exchange_settings::ExchangeSettings;

use super::participant::Participant;

pub fn risk_router(settings: &ExchangeSettings, participant_id: &u64) -> usize{

    (participant_id % settings.risk_engine_shards) as usize

}
