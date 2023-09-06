use crate::exchange::commands::OrderCommand;
use crate::order_handling::event::MatchingEngineEvent;
use crate::order_handling::order::*;
use crate::order_handling::order_book::OrderBook;
use crate::processor::order_book_processor::OrderBookProcessor;
use crate::processor::risk_engine_processor::RiskEngineProcessor;
use crate::risk::router::risk_router;

use futures::executor::block_on;
use log::{info, debug};
use std::collections::HashMap;
use std::thread;
use crossbeam::channel::{bounded, Receiver, Sender};
use tokio::sync::RwLock;

use super::asset::Symbol;
use super::exchange_settings::ExchangeSettings;

const ORDER_BOOK_COUNT: usize = 1;
#[derive(Default)]
pub struct Exchange {
    //pub accounts: RwLock<HashMap<u64, Account>>,
    //pub assets: RwLock<HashMap<&'a str, Asset>>,
    pub settings: ExchangeSettings,
    order_senders: Vec<Sender<OrderCommand>>,
}

impl Exchange {
    pub fn new(settings: ExchangeSettings) -> Self {
        let mut stage_1_senders = Vec::new();
        let mut stage_1_receivers = Vec::new();
        let mut stage_2_senders = Vec::new();
        let mut stage_2_receivers = Vec::new();
        let mut stage_3_senders = Vec::new();
        let mut stage_3_receivers = Vec::new();
        //Create Channels
        // Stage 1: One per risk engine shard:
        for i in 0..settings.risk_engine_shards {
            let (tx, rx) = bounded::<OrderCommand>(1000);
            stage_1_senders.push(tx);
            stage_1_receivers.push(rx);
        }

        // Stage 2: One per book
        for i in 0..settings.symbols.len() {
            let (tx, rx) = bounded::<OrderCommand>(1000);
            stage_2_senders.push(tx);
            stage_2_receivers.push(rx);
        }
        // Stage 3: One per risk engine shard
        for i in 0..settings.risk_engine_shards {
            let (tx, rx) = bounded::<MatchingEngineEvent>(1000);
            stage_3_senders.push(tx);
            stage_3_receivers.push(rx);
        }

        //Create risk engines
        for i in 0..settings.risk_engine_shards {
            let rec = stage_1_receivers.remove(0);
            let ev_rec = stage_3_receivers.remove(0);
            let senders = stage_2_senders.clone();
            let set = settings.clone();
            thread::spawn(move || {
                info!("Starting Risk Engine {:?}", i);
                let mut risk_engine = RiskEngineProcessor::new(set);
               risk_engine.run(rec, senders, ev_rec);
            });
        }

        // Create Order Book Processors for each symbol
        for (i, symbol) in settings.symbols.iter().enumerate() {
            let rev = stage_2_receivers.remove(0);
            let send = stage_3_senders.clone();
            let set = settings.clone();
            thread::spawn(move || {
                info!("Starting Order Book {:?}", i);
                OrderBookProcessor::new(set).run(rev, send);
            });
        }

        Self {
            settings,
            order_senders: stage_1_senders,
        }
    }
    /*
            pub fn add_account(&self, acc: Account) {
                self.accounts.write().unwrap().insert(acc.id, acc);
            }

            pub fn add_asset(&self, asset: Asset) {
                self.assets.write().unwrap().insert(asset.ticker, asset);
            }

        pub fn add_orderbook(&self, asset: &str, book: OrderBook) -> Result<(), &str> {
            match self.assets.read().unwrap().get(asset) {
                Some(ass) => match self
                    .orderbooks
                    .write()
                    .unwrap()
                    .insert(ass.ticker, RwLock::new(book))
                {
                    None => Ok(()),
                    Some(_) => Err("There is already an orderbook for this asset."),
                },
                None => Err("This asset does not exist."),
            }
        }
    */
    pub fn trade(&mut self, order_command: OrderCommand) {
        debug!("Sending TradeOrderCommand {:?}", order_command);
        let participant_id = match order_command {
            OrderCommand::Trade(trade) => trade.participant_id,
            OrderCommand::Cancel(cancel) => cancel.participant_id,
        };
        let shard = risk_router(&self.settings, &participant_id);
        let s = self.order_senders[shard].send(order_command);
        // println!("{:?}", s);
    }
    /*
    /// Check if an assets with that ticker exists on this exchange
    fn check_asset_existance<'b>(&'b self, asset: &str) -> Result<(), &str> {
        if self.assets.read().unwrap().contains_key(asset) {
            Ok(())
        } else {
            Err("This asset does not exist.")
        }
    }
    */
}
