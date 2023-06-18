use crate::exchange::commands::OrderCommand;
use crate::exchange::exchange_settings::ExchangeSettings;
use crate::order_handling::event::MatchingEngineEvent;
use crate::order_handling::order::*;
use crate::order_handling::order_book::OrderBook;
use crate::risk::router::{self, risk_router};
use tokio::sync::mpsc;
use tokio::sync::mpsc::{Receiver, Sender};

use std::collections::HashMap;
use tokio::sync::RwLock;

#[derive(Default)]
pub struct OrderBookProcessor {
    settings: ExchangeSettings,
}

impl OrderBookProcessor {
    pub fn new(settings: ExchangeSettings) -> Self {
        Self { settings }
    }

    pub async fn run(
        &mut self,
        mut receiver: Receiver<OrderCommand>,
        senders: Vec<Sender<MatchingEngineEvent>>,
    ) {
        let mut book = OrderBook::new(self.settings.clone(), senders);

        while let Some(order_command) = receiver.recv().await {
            match order_command {
                OrderCommand::Trade(trade) => {
                    book.insert_order(&trade).await;
                }
                OrderCommand::Cancel(cancel) => {
                    book.cancel_order(cancel.order_id);
                }
            }
        }
    }
}
