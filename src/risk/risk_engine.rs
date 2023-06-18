use std::{
    collections::HashMap,
    sync::{Mutex, MutexGuard},
};

use crate::{
    exchange::{
        asset::{Symbol, SymbolType},
        commands::{OrderCommand, TradeCommand},
        exchange::Exchange,
        exchange_settings::ExchangeSettings,
    },
    order_handling::{event::MatchingEngineEvent, order::OrderSide},
};

use super::{
    participant::{self, Participant},
    risk_order::RiskOrder,
};

pub enum RiskEngineResult {
    ValidForMatchingEngine,
    InsufficientFunds,

    SymbolNotFound,
    UserNotFound,
}

#[derive(Default)]
pub struct RiskEngine {
    participants: HashMap<u64, Mutex<Participant>>,
    settings: ExchangeSettings,
    // participant_id, symbol_id
    orders: HashMap<u64, (u64, u64, Mutex<RiskOrder>)>,
}

impl RiskEngine {
    fn add_participant(&mut self, part: Participant) {
        self.participants.insert(part.id, Mutex::new(part));
    }
    pub fn process_command(&mut self, command: OrderCommand) -> RiskEngineResult {
        match command {
            OrderCommand::Trade(command) => {
                let symbol = &self.settings.symbols[command.symbol as usize];
                let user = self.participants.get_mut(&command.participant_id);
                match user {
                    Some(user) => {
                        let mut user = user.lock().unwrap();
                        let base_asset = symbol.base_asset;
                        let quote_asset = symbol.quote_asset;
                        match symbol.symbol_type {
                            SymbolType::ExchangePair => Self::place_exchange_order(
                                &symbol,
                                command,
                                &mut *user,
                                command,
                                &mut self.orders,
                            ),
                            SymbolType::FuturesContract => todo!(),
                            SymbolType::Option => todo!(),
                        }
                    }
                    None => RiskEngineResult::UserNotFound,
                }
            }
            OrderCommand::Cancel(command) => todo!(),
        }
    }
    fn place_exchange_order(
        symbol: &Symbol,
        command: TradeCommand,
        user: &mut Participant,
        trade_command: TradeCommand,
        orders: &mut HashMap<u64, (u64, u64, Mutex<RiskOrder>)>,
    ) -> RiskEngineResult {
        let (pessimistic_asset, pessimistic_value) = command.pessimistic(symbol);

        match user.assets.get(&pessimistic_asset) {
            Some(user_asset) => {
                if user_asset >= &trade_command.volume {
                    //Pessimistically reduce the assets by the highest possible amount
                    user.assets
                        .insert(pessimistic_asset, user_asset - pessimistic_value);
                    orders.insert(
                        trade_command.id,
                        (
                            trade_command.participant_id,
                            trade_command.symbol,
                            Mutex::new(RiskOrder::from(trade_command)),
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

                let participant = self.participants.get(participant_id).expect(
                    "Order was filled for participant that was not known to the risk engine.",
                );

                let order = order.lock().unwrap();
                let mut participant = participant.lock().unwrap();

                let symbol = &self.settings.symbols[*symbol_id as usize];

                // The pessimistic amount of value that was previously deducted for the filled amount of value
                let (pessimistic_asset, pessimistic_value) =
                    TradeCommand::historic_pessimistic(order.side, order.limit, &symbol, volume);

                let (rising_asset, rising_value) = match order.side {
                    OrderSide::BID => (symbol.quote_asset, volume),
                    OrderSide::ASK => (symbol.base_asset, value),
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
                *asset -= pessimistic_value - value;
            }
            MatchingEngineEvent::Canceled(id) => {
                //Add the pessimistically removed assets back

                let (participant_id, symbol_id, order) = self
                    .orders
                    .get_mut(&id)
                    .expect("Order filled that was not known to the risk engine");

                let participant = self.participants.get(participant_id).expect(
                    "Order was filled for participant that was not known to the risk engine.",
                );

                let order = order.get_mut().unwrap();
                let mut participant = participant.lock().unwrap();

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
                *asset -= pessimistic_value;
            }
        }
    }
}
