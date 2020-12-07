//mod order_bucket;
//extern crate bit_vec;

use crate::order_handling::order::*;
use crate::order_handling::order_bucket::*;
use crate::primitives::*;

//use bit_vec::BitVec;

use std::collections::HashMap;

use std::cell::Cell;
//use std::collections::btree_map::BTreeMap;
//use tokio::sync::mpsc::error::SendError;

//use std::collections::btree_map::RangeMut;
//use std::ops::Range;

use std::rc::Rc;
use std::result::Result::*;

const MAX_PRICE: usize = 1_000;
pub struct OrderBook {
    /// Maximum price allowed on this orderbook
    max_price: Price,

    /// Lowest current ask price
    min_ask_price: Price,

    /// Highest current bid price
    max_bid_price: Price,

    // ask_depth_fenwick_tree: Vec<u64>,
    // bid_depth_fenwick_tree: Vec<u64>
    order_map: HashMap<u64, *mut Order>,

    /// Next Order ID
    highest_id: usize,

    /// Store orders sorted by price
    orders_array: [OrderBucket; MAX_PRICE],
}

impl Default for OrderBook {
    fn default() -> OrderBook {
        let mut array = unsafe {
            let mut arr: [OrderBucket; MAX_PRICE] = std::mem::uninitialized();
            let mut i = 0;
            for item in &mut arr[..] {
                std::ptr::write(item, OrderBucket::new(Price::new(i as u64)));
                i += 1;
            }
            arr
        };
        assert!(array.len() == MAX_PRICE);
        for i in 0..MAX_PRICE {
            array[i] = OrderBucket::new(Price::new(i as u64));
        }

        OrderBook {
            max_price: Price::new(MAX_PRICE as u64),
            min_ask_price: Price::new(MAX_PRICE as u64),
            max_bid_price: Price::ZERO,
            order_map: HashMap::with_capacity(MAX_PRICE),
            highest_id: 0,
            orders_array: array,
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
    async fn match_order(&mut self, order: &mut Order) {
        // Volume that remains in the incoming order
        let mut vol = order.volume;
        // Value of the already matched volume
        let mut val = Value::ZERO;

        while vol > Volume::new(0) {
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

            let matched_volume = self.orders_array[best_price.val as usize].match_orders(&vol);
            vol -= matched_volume;
            val += matched_volume * best_price;

            if self.orders_array[best_price.val as usize].total_volume == Volume::ZERO {
                match order.side {
                    OrderSide::ASK => self.max_bid_price.val -= 1,
                    OrderSide::BID => self.min_ask_price.val += 1,
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
        // println!("matching");
        self.match_order(&mut order);
        // println!("matched");

        if !order.is_filled() {
            if order.immediate_or_cancel {
                order.cancel();
            } else {
                self.order_map.insert(order.id, &mut order);
                self.orders_array[order.limit.val as usize].insert_order(order);
            }
        }
    }
    pub fn remove_order(&mut self, id: u64) -> Result<(), String> {
        unsafe {
            self.order_map
                .get(&id)
                .ok_or("This order does not exist")?
                .as_ref()
                .unwrap()
                .cancel();
        }
        self.order_map.remove(&id);
        Ok(())
    }

    pub fn inecrement_id(&mut self) -> usize {
        self.highest_id += 1;
        self.highest_id
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
