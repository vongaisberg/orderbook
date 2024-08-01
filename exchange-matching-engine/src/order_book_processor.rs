use std::time::Duration;

use crate::order_book::OrderBook;
use exchange_lib::commands::OrderCommand;
use exchange_lib::exchange_settings::ExchangeSettings;
use exchange_lib::message_queue::{
    connect, connect_tcp, pubsub_batching, pubsub_batching_tcp, Payload,
};
use log::debug;

#[derive(Default)]
pub struct OrderBookProcessor {
    symbol_id: usize,
    settings: ExchangeSettings,
}

impl OrderBookProcessor {
    pub fn new(symbol_id: usize, settings: ExchangeSettings) -> Self {
        Self {
            symbol_id,
            settings,
        }
    }

    pub fn run(&mut self) {
        let mut con_rx = connect();
        let mut con_tx = connect();
        let mut book = OrderBook::new(self.symbol_id, self.settings.clone());
        let channels_sub = ["orders".to_string()];
        let channel_pub = "risk".to_string();

        //Subscribe to channels
        // let _ = subscribe(&mut con_rx, &channels_sub, |msg| {
        //     println!("Event");
        //     if let Payload::CommandPayload(order_command) = msg.payload {
        //         debug!("RX: {:?}", order_command);
        //         match order_command {
        //             OrderCommand::Trade(trade) => {
        //                 // Insert order and provide callback which publishes the matching engine event
        //                 book.insert_order(&trade, &mut con_tx);
        //             }
        //             OrderCommand::Cancel(cancel) => {
        //                 book.cancel_order(cancel.order_id);
        //             }
        //         }
        //     }
        // });

        // pubsub_batching(
        //     &mut con_rx.as_pubsub(),
        //     &mut con_tx,
        //     &channels_sub,
        //     1000,
        //     Duration::from_millis(1),
        //     |msg, pipe| {
        //         if let Payload::CommandPayload(order_command) = msg.payload {
        //             // debug!("RX: {:?}", order_command);
        //             match order_command {
        //                 OrderCommand::Trade(trade) => {
        //                     // Insert order and provide callback which publishes the matching engine event
        //                     book.insert_order(&trade, pipe);
        //                 }
        //                 OrderCommand::Cancel(cancel) => {
        //                     book.cancel_order(cancel.order_id);
        //                 }
        //             }
        //         }
        //     },
        // )
        // .unwrap();

        pubsub_batching_tcp(
            &mut connect_tcp(),
            &mut connect_tcp(),
            &channels_sub,
            1000,
            Duration::from_millis(1),
            |msg, pipe| {
                if let Payload::CommandPayload(order_command) = msg.payload {
                    // debug!("RX: {:?}", order_command);
                    match order_command {
                        OrderCommand::Trade(trade) => {
                            // Insert order and provide callback which publishes the matching engine event

                            book.insert_order(&trade, pipe);
                        }
                        OrderCommand::Cancel(cancel) => {
                            book.cancel_order(cancel.order_id);
                        }
                    }
                }
            },
        )
        .unwrap();
    
    }
}
