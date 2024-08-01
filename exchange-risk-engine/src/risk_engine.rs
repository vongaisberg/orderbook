use std::collections::HashMap;

use exchange_lib::{
    asset::Symbol,
    asset::SymbolType,
    commands::{OrderCommand, TradeCommand},
    exchange_settings::ExchangeSettings,
};
use exchange_lib::{event::MatchingEngineEvent, order_side::OrderSide};

use super::{participant::Participant, risk_order::RiskOrder};

pub enum RiskEngineResult {
    ValidForMatchingEngine,
    InsufficientFunds,

    SymbolNotFound,
    UserNotFound,
    OrderNotFound,
}

pub struct RiskEngine {
    participants: HashMap<u64, Participant>,
    settings: ExchangeSettings,
    // participant_id, symbol_id
    orders: HashMap<u64, (u64, u64, RiskOrder)>,
}

impl RiskEngine {
    pub fn new(settings: ExchangeSettings) -> Self {
        Self {
            participants: HashMap::new(),
            settings,
            orders: HashMap::new(),
        }
    }
    pub fn add_participant(&mut self, part: Participant) {
        self.participants.insert(part.id, part);
    }
    pub fn process_command(&mut self, command: OrderCommand) -> RiskEngineResult {
        match command {
            OrderCommand::Trade(command) => {
                let symbol = &self.settings.symbols[command.symbol as usize];
                let user = self.participants.get_mut(&command.participant_id);
                match user {
                    Some(user) => match symbol.symbol_type {
                        SymbolType::ExchangePair => {
                            Self::place_exchange_order(symbol, user, command, &mut self.orders)
                        }
                        SymbolType::FuturesContract => todo!(),
                        SymbolType::Option => todo!(),
                    },
                    None => RiskEngineResult::UserNotFound,
                }
            }
            OrderCommand::Cancel(command) => {
                let symbol = &self.settings.symbols[command.symbol as usize];
                let user = self.participants.get_mut(&command.participant_id);
                match user {
                    Some(user) => match symbol.symbol_type {
                        SymbolType::ExchangePair => RiskEngineResult::ValidForMatchingEngine,
                        SymbolType::FuturesContract => todo!(),
                        SymbolType::Option => todo!(),
                    },
                    None => RiskEngineResult::UserNotFound,
                }
            }
        }
    }

    // fn cancel_exchange_order(
    //     symbol: &Symbol,
    //     user: &mut Participant,
    //     command: CancelCommand,
    //     orders: &mut HashMap<u64, (u64, u64, RiskOrder)>,
    // ) -> RiskEngineResult {
    //     match orders.remove(&command.order_id) {
    //         Some((_, s_ymbol, order)) => {
    //             let (pessimistic_asset, pessimistic_value) = TradeCommand::historic_pessimistic(
    //                 order.side,
    //                 order.limit,
    //                 symbol,
    //                 order.volume,
    //             );

    //             match user.assets.get(&pessimistic_asset) {
    //                 Some(user_asset) => {
    //                     user.assets
    //                         .insert(pessimistic_asset, user_asset + pessimistic_value);
    //                     RiskEngineResult::ValidForMatchingEngine
    //                 }
    //                 None => todo!(),
    //             }
    //         }
    //         None => RiskEngineResult::OrderNotFound,
    //     }
    // }

    fn place_exchange_order(
        symbol: &Symbol,
        user: &mut Participant,
        trade_command: TradeCommand,
        orders: &mut HashMap<u64, (u64, u64, RiskOrder)>,
    ) -> RiskEngineResult {
        let (pessimistic_asset, pessimistic_value) = trade_command.pessimistic(symbol);

        match user.assets.get(&pessimistic_asset) {
            Some(user_asset) => {
                if user_asset >= &pessimistic_value {
                    //Pessimistically reduce the assets by the highest possible amount
                    user.assets
                        .insert(pessimistic_asset, user_asset - pessimistic_value);
                    orders.insert(
                        trade_command.id,
                        (
                            trade_command.participant_id,
                            trade_command.symbol,
                            RiskOrder::from(trade_command),
                        ),
                    );
                    RiskEngineResult::ValidForMatchingEngine
                } else {
                    RiskEngineResult::InsufficientFunds
                }
            }
            None => RiskEngineResult::InsufficientFunds,
        }
    }

    pub fn process_matcher_event(&mut self, event: MatchingEngineEvent) {
        match event {
            MatchingEngineEvent::Filled(id, volume, value) => {
                let (participant_id, symbol_id, order) = self
                    .orders
                    .get_mut(&id)
                    .expect("Order filled that was not known to the risk engine");

                let participant = self.participants.get_mut(participant_id).expect(
                    "Order was filled for participant that was not known to the risk engine.",
                );

                let symbol = &self.settings.symbols[*symbol_id as usize];

                // The pessimistic amount of value that was previously deducted for the filled amount of value
                let (pessimistic_asset, pessimistic_value) =
                    TradeCommand::historic_pessimistic(order.side, order.limit, symbol, volume);

                let (rising_asset, rising_value, falling_value) = match order.side {
                    OrderSide::BID => (symbol.quote_asset, volume, value),
                    OrderSide::ASK => (symbol.base_asset, value, volume),
                };
                let rising_asset = participant
                    .assets
                    .get_mut(&rising_asset)
                    .expect("Order filled for user asset not known to the risk engine.");
                *rising_asset += rising_value;
                let asset = participant
                    .assets
                    .get_mut(&pessimistic_asset)
                    .expect("Order filled for user asset not known to the risk engine.");

                // Update the asset to reflect the value actually paid
                let difference = pessimistic_value - falling_value;
                *asset += difference;
            }
            MatchingEngineEvent::Canceled(id) => {
                //Add the pessimistically removed assets back

                let (participant_id, symbol_id, order) = self
                    .orders
                    .get_mut(&id)
                    .expect("Order filled that was not known to the risk engine");

                let participant = self.participants.get_mut(participant_id).expect(
                    "Order was filled for participant that was not known to the risk engine.",
                );

                let symbol = &self.settings.symbols[*symbol_id as usize];

                let (pessimistic_asset, pessimistic_value) = TradeCommand::historic_pessimistic(
                    order.side,
                    order.limit,
                    symbol,
                    order.volume,
                );

                let asset = (participant)
                    .assets
                    .get_mut(&pessimistic_asset)
                    .expect("Order filled for user asset not known to the risk engine.");

                // Update the asset to reflect the value actually paid
                *asset += pessimistic_value;
            }
        }
    }
}
