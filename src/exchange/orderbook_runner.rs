use crate::order_handling::order::*;
use crate::order_handling::order::*;
use crate::order_handling::order_book::OrderBook;

use tokio::sync::mpsc::*;

use std::cell::RefCell;

const CHANNEL_CAPACITY: usize = 100;

pub struct OrderBookRunner {
    orderbook: OrderBook,
    event_sender: Sender<OrderEvent>,

    pub event_receiver: Receiver<OrderEvent>,
}

impl OrderBookRunner {
    pub fn new(orderbook: OrderBook) -> OrderBookRunner {
        let (event_sender, event_receiver) = channel(CHANNEL_CAPACITY);

        OrderBookRunner {
            orderbook: orderbook,
            event_sender: event_sender,
            event_receiver: event_receiver,
        }
    }

    pub async fn insert_order(&mut self, mut order: Order) {
        //order.event_sender = Some(RefCell::new(self.event_sender.clone()));
        self.orderbook.insert_order(order);
    }
}
