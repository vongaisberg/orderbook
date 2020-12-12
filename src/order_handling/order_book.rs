use crate::order_handling::order::*;
use crate::order_handling::order_bucket::*;
use crate::primitives::*;
extern crate libc;

use std::collections::HashMap;

use std::cell::Cell;

use std::rc::Rc;
use std::result::Result::*;

use std::alloc::{alloc, dealloc, Layout};
use std::cmp::*;
use std::mem;
use std::ops::Drop;

const MAX_PRICE: usize = 1_000;
const MAX_NUMBER_OF_ORDERS: usize = 1_000_000_00;
pub struct OrderBook {
    /// Maximum price allowed on this orderbook
    max_price: Price,

    /// Lowest current ask price
    min_ask_price: Price,

    /// Highest current bid price
    max_bid_price: Price,

    // ask_depth_fenwick_tree: Vec<u64>,
    // bid_depth_fenwick_tree: Vec<u64>
    order_map: Box<[*mut Order; MAX_NUMBER_OF_ORDERS]>,

    /// Next Order ID
    highest_id: usize,

    /// Store orders sorted by price
    pub orders_array: [OrderBucket; MAX_PRICE],
}

impl Default for OrderBook {
    fn default() -> OrderBook {
        //Fill orders_array with OrderBuckets
        let orders_array = unsafe {
            let mut arr: [OrderBucket; MAX_PRICE] = std::mem::uninitialized();
            let mut i = 0;
            for item in &mut arr[..] {
                std::ptr::write(item, OrderBucket::new(Price(i as u64)));
                i += 1;
            }
            arr
        };
        assert!(orders_array.len() == MAX_PRICE);

        let layout = Layout::new::<[*mut Order; MAX_NUMBER_OF_ORDERS]>();
        println!(
            "Size of order_array: {}MB",
            mem::size_of::<[*mut Order; MAX_NUMBER_OF_ORDERS]>() as f32 / 1000000f32
        );
        let b = unsafe {
            mem::transmute::<*mut u8, Box<[*mut Order; MAX_NUMBER_OF_ORDERS]>>(alloc(layout))
        };

        OrderBook {
            max_price: Price(MAX_PRICE as u64),
            min_ask_price: Price(MAX_PRICE as u64),
            max_bid_price: Price::ZERO,
            order_map: b,
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
        let mut val = Value::ZERO;

        while vol > Volume(0) {
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

            let matched_volume = self.orders_array[*best_price as usize].match_orders(&vol);
            //println!("Matched volume: {}", *matched_volume);
            vol -= matched_volume;
            val += matched_volume * best_price;

            if self.orders_array[best_price.get() as usize].total_volume == Volume::ZERO {
                match order.side {
                    OrderSide::ASK => *self.max_bid_price -= 1,
                    OrderSide::BID => *self.min_ask_price += 1,
                }
            }
        }

        let new_filled_vol = order.volume - vol;

        order.filled_volume.set(new_filled_vol);
        order.filled_value.set(val);
        if new_filled_vol.get() > 0 {
            order.notify();
        }
    }

    pub fn insert_order(&mut self, mut order: Order) {
        //println!("matching");
        self.match_order(&mut order);
        //println!("matched");

        if !order.is_filled() {
            if order.immediate_or_cancel {
                order.cancel();
            } else {
                match order.side {
                    OrderSide::ASK => {
                        self.min_ask_price = Price(min(*self.min_ask_price, *order.limit))
                    }
                    OrderSide::BID => {
                        self.max_bid_price = Price(max(*self.max_bid_price, *order.limit))
                    }
                }
                self.order_map[order.id as usize] = &mut order;
                self.orders_array[order.limit.get() as usize].insert_order(order);
            }
        }
    }
    pub fn remove_order(&mut self, id: u64) {
        unsafe {
            self.order_map[id as usize].as_ref().unwrap().cancel();
        }
        //We don't need to remove the order from the order map, as we are simply going to
        // self.order_map.remove(&id);
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
