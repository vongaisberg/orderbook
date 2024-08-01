use crate::exchange::exchange_settings::ExchangeSettings;
use crate::exchange::{asset::AssetId, commands::OrderCommand};
use crate::order_handling::event::MatchingEngineEvent;
use crate::order_handling::order::*;
use crate::order_handling::order_book::OrderBook;
use crossbeam::channel::{Receiver, Sender};
use tokio::sync::mpsc;

use redis;
use std::cmp::max;
use std::collections::{HashMap, VecDeque};
use std::time::{Duration, Instant};
use tokio::sync::RwLock;

fn connect() -> redis::RedisResult<redis::Connection> {
    let client = redis::Client::open("redis://127.0.0.1/")?;
    client.get_connection()
}

// Keep score of all of the positions of each market participant
pub struct DatabaseProcessor {
    connection: redis::Connection,
    last_sync: Instant,
    settings: ExchangeSettings,
    event_queue: VecDeque<MatchingEngineEvent>,
}

impl DatabaseProcessor {
    fn new(settings: ExchangeSettings) -> Self {
        Self {
            connection: connect().unwrap(),
            last_sync: Instant::now(),
            settings,
            event_queue: VecDeque::new(),
        }
    }

    pub fn run(&mut self, receiver: Receiver<MatchingEngineEvent>) {
        let conn = connect();

        loop {
            if let Ok(order_event) = receiver.recv_timeout(max(
                self.settings.db_min_recv_timeout,
                self.settings.db_sync_speed - (Instant::now() - self.last_sync),
            )) {
                self.event_queue.push_back(order_event);
                //If enough time has passed, send all of the event to the DB
                if Instant::now() - self.last_sync > self.settings.db_sync_speed {
                    self.sync_database(self);
                }
            } else {
                // Timeout reached
            }
        }
    }
    fn sync_database(&mut self) {
        let pipe = redis.pipe();
        for event in 
    }
}
