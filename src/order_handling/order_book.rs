use crate::exchange::commands::TradeCommand;
use crate::order_handling::order::*;
use crate::order_handling::order_bucket::*;
extern crate libc;

use linked_hash_map::VacantEntry;
use log::{debug, error, info, trace, warn};
use std::alloc::{alloc, dealloc, Layout};
use std::boxed;
use std::cell::Cell;
use std::cell::RefCell;
use std::cell::RefMut;
use std::cmp::*;
use std::collections::hash_map::Entry::{Occupied, Vacant};
use std::collections::hash_map::OccupiedEntry;
use std::mem;
use std::mem::MaybeUninit;
use std::ops::Drop;
use std::rc::Rc;
use std::result::Result::*;
use std::sync::mpsc::Sender;
use std::{collections::HashMap, ptr::NonNull};

use fxhash::FxBuildHasher;

use crate::order_handling::public_list::*;

use super::order_bucket;

const MAX_NUMBER_OF_ORDERS: usize = 10_000_000;
const MAX_PRICE: usize = 2_000;

pub struct OrderBook {
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

    pub event_sender: Option<Sender<OrderEvent>>,
}

impl Default for OrderBook {
    fn default() -> OrderBook {
        info!(
            "Size of order_array: {}MB",
            mem::size_of::<[Box<OrderBucket>; MAX_PRICE]>() as f32 / 1000000f32
        );

        let mut price = 1;
        //Fill orders_array with OrderBuckets
        let orders_array = {
            let mut data: [MaybeUninit<Box<OrderBucket>>; MAX_PRICE] =
                unsafe { MaybeUninit::uninit().assume_init() };

            // Dropping a `MaybeUninit` does nothing, so if there is a panic during this loop,
            // we have a memory leak, but there is no memory safety issue.
            for elem in &mut data[..] {
                elem.write(Box::new(OrderBucket::new(price)));
            }
            price += 1;

            // Everything is initialized. Transmute the array to the
            // initialized type.
            unsafe { mem::transmute::<_, [Box<OrderBucket>; MAX_PRICE]>(data) }
        };
        assert!(orders_array.len() == MAX_PRICE);

        // let layout = Layout::new::<[Order; MAX_NUMBER_OF_ORDERS]>();

        OrderBook {
            max_price: MAX_PRICE as u64,
            min_ask_price: MAX_PRICE as u64,
            max_bid_price: 0,
            order_map: HashMap::with_capacity_and_hasher(
                MAX_NUMBER_OF_ORDERS,
                FxBuildHasher::default(),
            ),
            highest_id: 0,
            bucket_array: orders_array,
            event_sender: None,
        }
    }
}

impl OrderBook {
    pub fn new() -> OrderBook {
        Default::default()
    }

    /// Try to instantly match an order as it is coming in
    ///
    ///
    /// The incoming order will possibly take liquidity from the orderbook.
    ///
    ///
    fn match_order(&mut self, order: &mut StandingOrder) {
        // Volume that remains in the incoming order
        let mut vol = order.volume;
        // Value of the already matched volume
        let mut val = 0;

        while vol > 0 {
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
                OrderBucket::match_orders(vol, self, best_price)
            {
                //Loop gets entered at least once, so bucket is not empty
                empty = false;
                // println!("Matched volume: {}", matched_volume);
                vol -= matched_volume;
                val += matched_volume * best_price;
                if let Some(canceled_order) = canceled_order {
                    self.cancel_order(canceled_order)
                }
                if vol == 0 {
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

        let new_filled_vol = order.volume - vol;

        order.filled_volume.set(new_filled_vol);
        order.filled_value.set(val);
        if new_filled_vol > 0 {
            order.notify(new_filled_vol, val, &self.event_sender);
        }
    }

    pub fn insert_order(&mut self, trade: &TradeCommand) {
        // println!(
        //     "Insert order: {}, Limit: {}, Side: {:?}, Volume: {:?}",
        //     order.id, order.limit, order.side, order.volume
        // );

        let mut order =
            StandingOrder::new(self.increment_id(), trade.limit, trade.volume, trade.side);
        self.match_order(&mut order);
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
