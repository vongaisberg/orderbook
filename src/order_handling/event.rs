#[derive(Debug, Copy, Clone)]
pub enum MatchingEngineEvent {
    ///How much volume was filled and how much Value was payed for it
    /// id, volume, value
    Filled(u64, u64, u64),
    /// id
    Canceled(u64),
}
