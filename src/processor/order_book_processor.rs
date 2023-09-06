use crate::exchange::commands::OrderCommand;
use crate::exchange::exchange_settings::ExchangeSettings;
use crate::order_handling::event::MatchingEngineEvent;
use crate::order_handling::order::{self, *};
use crate::order_handling::order_book::OrderBook;
use crate::risk::router::{self, risk_router};
use crossbeam::channel::{Receiver, Sender};
use log::debug;
use tokio::sync::mpsc;

use std::collections::HashMap;
use tokio::sync::RwLock;

#[derive(Default)]
pub struct OrderBookProcessor {
    symbol_id: usize,
    settings: ExchangeSettings,
}

impl OrderBookProcessor {
    pub fn new(symbol_id: usize, settings: ExchangeSettings) -> Self {
        Self { symbol_id,settings }
    }

    pub fn run(
        &mut self,
        receiver: Receiver<OrderCommand>,
        senders: Vec<Sender<MatchingEngineEvent>>,
    ) {
        let mut book = OrderBook::new(self.settings.clone(), senders);

        while let Ok(order_command) = receiver.recv() {
            debug!("Order book received command: {:?}", order_command);
            match order_command {
                OrderCommand::Trade(trade) => {
                    book.insert_order(&trade);
                }
                OrderCommand::Cancel(cancel) => {
                    book.cancel_order(cancel.order_id);
                }
            }
        }
    }
}
