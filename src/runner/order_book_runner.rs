use crate::exchange::account::Account;
use crate::exchange::asset::Asset;
use crate::exchange::commands::OrderCommand;
use crate::order_handling::order::*;
use crate::order_handling::order_book::OrderBook;
use std::sync::mpsc;
use std::sync::mpsc::{Receiver, Sender};

use std::collections::HashMap;
use std::sync::RwLock;

const ORDER_BOOK_COUNT: usize = 1;
#[derive(Default)]
pub struct OrderBookRunner {
    pub orderbook: OrderBook,
}

impl OrderBookRunner {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn run(&mut self, receiver: Receiver<OrderCommand>, sender: Sender<OrderEvent>) {
        while let Ok(order_command) = receiver.recv() {
            match order_command {
                OrderCommand::Trade(trade) => {
                    let book = &mut self.orderbook;
                    book.insert_order(&trade);
                }
                OrderCommand::Cancel(cancel) => {
                    let book = &mut self.orderbook;
                    book.cancel_order(cancel.order_id);
                }
            }
        }
    }
}
