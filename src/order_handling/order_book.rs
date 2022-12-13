use crate::order_handling::order::*;
use crate::order_handling::order_bucket::*;
extern crate libc;

use std::mem::MaybeUninit;
use std::{collections::HashMap, ptr::NonNull};

use std::cell::Cell;

use std::rc::Rc;
use std::result::Result::*;

use std::alloc::{alloc, dealloc, Layout};
use std::cmp::*;
use std::mem;
use std::ops::Drop;

use crate::order_handling::public_list::*;

const MAX_NUMBER_OF_ORDERS: usize = 10_000_000;
const MAX_PRICE: usize = 1_000;

pub struct OrderBook {
    /// Maximum price allowed on this orderbook
    max_price: u64,

    /// Lowest current ask price
    pub min_ask_price: u64,

    /// Highest current bid price
    pub max_bid_price: u64,

    // ask_depth_fenwick_tree: Vec<u64>,
    // bid_depth_fenwick_tree: Vec<u64>
    order_map: Vec<u64>,

    /// Next Order ID
    highest_id: usize,

    /// Store orders sorted by price
    pub orders_array: [Box<OrderBucket>; MAX_PRICE],
}

impl Default for OrderBook {
    fn default() -> OrderBook {
        println!(
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
            order_map: vec![0; 100_000_000],
            highest_id: 0,
            orders_array: orders_array,
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
    fn match_order(&mut self, order: &mut Order) {
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
            //println!("Best price: {}, Limit: {}, Side: {:?}, Bucket Volume: {:?}", *best_price, *order.limit, order.side, self.orders_array[best_price.get() as usize].total_volume);

            let matched_volume = self.orders_array[best_price as usize].match_orders(vol);
            //println!("Matched volume: {}", *matched_volume);
            vol -= matched_volume;
            val += matched_volume * best_price;

            if self.orders_array[best_price as usize].total_volume == 0 {
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
            order.notify();
        }
    }

    pub fn insert_order(&mut self, mut order: Order) {
        // println!(
        //     "Insert order: {}, Limit: {}, Side: {:?}, Volume: {:?}",
        //     order.id, order.limit.0, order.side, order.volume
        // );
        self.match_order(&mut order);
        // println!("Matched, order: {:?}", order);

        if !order.is_filled() {
            if order.immediate_or_cancel {
                order.cancel();
            } else {
                match order.side {
                    OrderSide::ASK => self.min_ask_price = min(self.min_ask_price, order.limit),
                    OrderSide::BID => self.max_bid_price = max(self.max_bid_price, order.limit),
                }
                let id = order.id;
                self.order_map[id as usize] = order.limit;
                self.orders_array[order.limit as usize].insert_order(order);
            }
        }
    }
    pub fn cancel_order(&mut self, id: u64) {
        let price = self.order_map[id as usize];

        // println!("Removing element: {}", id);

        self.orders_array[price as usize].remove_order(id);
    }

    pub fn increment_id(&mut self) -> usize {
        self.highest_id += 1;
        self.highest_id - 1usize
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
