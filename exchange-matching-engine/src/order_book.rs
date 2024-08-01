use crate::order::*;
use crate::order_bucket::*;
use exchange_lib::commands::TradeCommand;
use exchange_lib::event::DbEvent;
use exchange_lib::event::MatchingEngineEvent;
use exchange_lib::exchange_settings::ExchangeSettings;
use exchange_lib::message_queue::Message;
use exchange_lib::order_side::OrderSide;
use exchange_lib::router::risk_router;
use fxhash::FxBuildHasher;
use log::{debug, error, info, trace, warn};
use redis::Connection;
use redis::Pipeline;
use std::cmp::*;
use std::collections::hash_map::Entry::{Occupied, Vacant};
use std::collections::HashMap;
use std::mem;
use std::mem::MaybeUninit;

const MAX_NUMBER_OF_ORDERS: usize = 10_000_000;
const MAX_PRICE: usize = 2_000;

pub struct OrderBook {
    /// Symbol which is traded here
    symbol_id: usize,
    /// Maximum price allowed on this orderbook
    max_price: u64,

    /// Lowest current ask price
    pub min_ask_price: u64,

    /// Highest current bid price
    pub max_bid_price: u64,

    // ask_depth_fenwick_tree: Vec<u64>,
    // bid_depth_fenwick_tree: Vec<u64>
    order_map: HashMap<u64, Box<StandingOrder>, FxBuildHasher>,

    /// Next Order ID
    highest_id: u64,

    /// Store orders sorted by price
    pub bucket_array: [Box<OrderBucket>; MAX_PRICE],

    settings: ExchangeSettings,
}

impl OrderBook {
    pub fn new(symbol_id: usize, settings: ExchangeSettings) -> OrderBook {
        let mut price = 0;
        //Fill orders_array with OrderBuckets
        let orders_array = {
            let mut data: [MaybeUninit<Box<OrderBucket>>; MAX_PRICE] =
                unsafe { MaybeUninit::uninit().assume_init() };

            // Dropping a `MaybeUninit` does nothing, so if there is a panic during this loop,
            // we have a memory leak, but there is no memory safety issue.
            for elem in &mut data[..] {
                elem.write(Box::new(OrderBucket::new(price)));

                price += 1;
            }

            // Everything is initialized. Transmute the array to the
            // initialized type.
            unsafe { mem::transmute::<_, [Box<OrderBucket>; MAX_PRICE]>(data) }
        };
        assert!(orders_array.len() == MAX_PRICE);

        // let layout = Layout::new::<[Order; MAX_NUMBER_OF_ORDERS]>();

        OrderBook {
            symbol_id,
            max_price: MAX_PRICE as u64,
            min_ask_price: MAX_PRICE as u64,
            max_bid_price: 0,
            order_map: HashMap::with_capacity_and_hasher(
                MAX_NUMBER_OF_ORDERS,
                FxBuildHasher::default(),
            ),
            highest_id: 0,
            bucket_array: orders_array,
            settings,
        }
    }

    /// Try to instantly match an order as it is coming in
    ///
    ///
    /// The incoming order will possibly take liquidity from the orderbook.
    ///
    ///
    fn match_order(
        &mut self,
        con_tx: &mut Vec<Message>,
        order: &mut StandingOrder,
    ) {
        let mut filled_value = 0;
        let original_volume = order.volume;

        while order.volume > 0 {
            let best_price = match order.side {
                OrderSide::ASK => {
                    if self.max_bid_price < order.limit {
                        break;
                    } else {
                        self.max_bid_price
                    }
                }
                OrderSide::BID => {
                    if self.min_ask_price > order.limit {
                        break;
                    } else {
                        self.min_ask_price
                    }
                }
            };
            // println!(
            //     "Best price: {}, Limit: {}, Side: {:?}",
            //     best_price, order.limit, order.side
            // );
            let mut empty = true;
            //Loop until bucket is empty
            while let Some((matched_volume, canceled_order)) =
                OrderBucket::match_orders(&order, self, best_price, con_tx)
            {
                //Loop gets entered at least once, so bucket is not empty
                empty = false;
                // println!("Matched volume: {}", matched_volume);
                order.volume -= matched_volume;
                filled_value += matched_volume * best_price;
                if let Some(canceled_order) = canceled_order {
                    debug!("Canceling order: {}", canceled_order);
                    self.cancel_order(canceled_order)
                }
                if order.volume == 0 {
                    break;
                }
            }
            if empty {
                match order.side {
                    OrderSide::ASK => self.max_bid_price -= 1,
                    OrderSide::BID => self.min_ask_price += 1,
                }
            }
        }

        if filled_value > 0 {
            // Notify the order
            order.notify(original_volume - order.volume, filled_value, con_tx);
        }
    }

    pub fn insert_order(
        &mut self,
        trade: &TradeCommand,
        con_tx: &mut Vec<Message>,
    ) {
        let mut order = StandingOrder::new(
            trade.id,
            trade.participant_id,
            trade.limit,
            trade.volume,
            trade.side,
        );

        // println!(
        //     "Insert order: {}, Limit: {}, Side: {:?}, Volume: {:?}",
        //     order.id, order.limit, order.side, order.volume
        // );
        self.match_order(con_tx, &mut order);
        // println!("Matched, order: {:?}", order);

        if !order.is_filled() {
            match order.side {
                OrderSide::ASK => self.min_ask_price = min(self.min_ask_price, order.limit),
                OrderSide::BID => self.max_bid_price = max(self.max_bid_price, order.limit),
            }
            let id = order.id;
            let limit = order.limit;

            //Get a raw pointer to the order and put it into order_map´

            let boxed_order = Box::new(order);
            let entry = self.order_map.entry(id);

            let occupied_entry = entry.insert_entry(boxed_order);
            self.bucket_array[limit as usize].insert_order(occupied_entry.get().into());

            //Set pointer to the HashMap Entry into the Orde
        }
    }

    pub fn cancel_order(&mut self, id: u64) {
        match self.order_map.entry(id) {
            Vacant(v) => (),
            Occupied(entry) => {
                //Remove from hashmap
                let mut order = entry.remove();

                //Remove from linked list
                order.remove_from_bucket(&mut self.bucket_array[order.limit as usize])
            }
        }
    }

    pub fn increment_id(&mut self) -> u64 {
        self.highest_id += 1;
        self.highest_id - 1
    }
}

/*
fn first_entry(vec: BitVec<u32>) -> Option<u32> {
    vec.blocks()
        .enumerate()
        .filter(|(_n, b)| *b != 0u32)
        .map(|(n, b)| (n as u32) * 32 + b.trailing_zeros())
        .next()
}
*/
