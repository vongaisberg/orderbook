
use crate::order_handling::order::*;
use crate::order_handling::order_book::OrderBook;

use std::sync::mpsc::*;

const CHANNEL_CAPACITY: usize = 100;

pub struct OrderBookRunner {
    orderbook: OrderBook,
    order_receiver: Receiver<Order>,
    event_sender: SyncSender<OrderEvent>,

    pub order_sender: SyncSender<Order>,
    pub event_receiver: Receiver<OrderEvent>,
}

impl OrderBookRunner {
    pub fn new(orderbook: OrderBook) -> OrderBookRunner {
        let (order_sender, order_receiver) = sync_channel(CHANNEL_CAPACITY);
        let (event_sender, event_receiver) = sync_channel(CHANNEL_CAPACITY);

        OrderBookRunner {
            orderbook: orderbook,
            order_receiver: order_receiver,
            event_sender: event_sender,
            order_sender: order_sender,
            event_receiver: event_receiver,
        }
    }

    pub fn run(&mut self) {
        while let Ok(order) = self.order_receiver.recv() {
            self.orderbook.insert_order(order);
        }
    }
}
