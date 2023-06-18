
use crate::exchange::commands::OrderCommand;
use crate::order_handling::event::MatchingEngineEvent;
use crate::order_handling::order::*;
use crate::order_handling::order_book::OrderBook;
use tokio::sync::mpsc;
use tokio::sync::mpsc::{Receiver, Sender};

use std::collections::HashMap;
use tokio::sync::RwLock;

const ORDER_BOOK_COUNT: usize = 1;
#[derive(Default)]
pub struct AccountRunner {
    receivers: Vec<Receiver<MatchingEngineEvent>>
}

impl AccountRunner {
    pub fn new() -> Self {
        Self::default()
    }

    pub async fn run(&mut self, mut receiver: Receiver<MatchingEngineEvent>) {
        while let Some(order_event) = receiver.recv().await {
            match order_event {
                MatchingEngineEvent::Filled(id, vol, val) => {}
                MatchingEngineEvent::Canceled(cancel) => {}
            }
        }
    }
}
