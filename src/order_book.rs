//mod order_bucket;
extern crate bit_vec;

use crate::order::*;
use crate::order_bucket::*;
use crate::primitives::*;

use bit_vec::BitVec;
use intmap::IntMap;
use std::collections::btree_map::BTreeMap;
use std::collections::btree_map::RangeMut;
use std::ops::Range;
use std::cell::Cell;

use std::rc::Rc;
use std::result::Result::*;

const DEFAULT_HOT_PRICE_RANGE: u64 = 1024;
const DEFAULT_HOT_PRICE_MIDPOINT: u64 = 2048;
const DEFAULT_HOT_MAP_CAPACITY: usize = 128;

pub struct OrderBook {
    //hot_set: BitVec,
    //hot_price_midpoint: u64,
    //hot_price_range: u64,

    // hot_map: IntMap<Rc<OrderBucket>>,
    /// Store ask orders sorted by price
    cold_ask_map: BTreeMap<Price, OrderBucket>,

    /// Store bid orders sorted by price
    cold_bid_map: BTreeMap<Price, OrderBucket>,
    // ask_depth_fenwick_tree: Vec<u64>,
    
    // bid_depth_fenwick_tree: Vec<u64>
    order_map: IntMap<Rc<Order>>,
}

impl Default for OrderBook {
    fn default() -> OrderBook {
        OrderBook {
            //hot_ask_set: BitVec::from_elem((DEFAULT_HOT_PRICE_RANGE * 2) as usize, false),
            //hot_bid_set: BitVec::from_elem((DEFAULT_HOT_PRICE_RANGE * 2) as usize, false),

            //hot_map: IntMap::with_capacity(DEFAULT_HOT_MAP_CAPACITY),

            //hot_price_midpoint: DEFAULT_HOT_PRICE_MIDPOINT,
            //hot_price_range: DEFAULT_HOT_PRICE_RANGE,
            cold_ask_map: BTreeMap::new(),
            cold_bid_map: BTreeMap::new(),

            order_map: IntMap::new(),
        }
    }
}

impl OrderBook {
    pub fn new() -> OrderBook {
        Default::default()
    }

    fn is_hot(&self, price: Price) -> bool {
        //[self.hot_price_midpoint - self.hot_price_range..=
        //    self.hot_price_midpoint + self.hot_price_range].contains(price)
        false
    }

    /// Try to instantly match an order as it is coming in
    ///
    ///
    /// The incoming order will possibly take liquidity from the orderbook.
    ///
    ///
    fn match_order(&mut self, order: &mut Order) {
        // Currently only handle cold orders, will handle hot orders in the future

        /// Match as much as possible from vol with bucket if price is right
        /// Return the volume that is left over and the value that was payed for all matched volume
        let lambda = |(vol, val): (Volume, Value), (price, bucket): (&Price, &mut OrderBucket)| {
            if order.matches_with(price) {
                
                let vol_matched = bucket.match_orders(vol);
                if vol_matched == vol {
                    //Everything was matched, shortcut the try_fold
                    Err((Volume::ZERO, (val + (vol_matched * *price))))
                } else {
                    //Not everything was matched, return how much is left
                    Ok((vol - vol_matched, (val + (vol_matched * *price))))
                }
            } else {
                Err((vol, val))
            }
        };

        let (new_filled_vol, new_filled_val) = match match order.side {
            // Using try_fold for the short_circuiting feature, when the order is already filled
            // Err(x) and Ok(x) both mean that the order has been filled as much as possible and x volume remains in it
            OrderSide::ASK => self
                .cold_ask_map
                .range_mut(..)
                .try_rfold((order.volume, Value::ZERO), lambda),
            OrderSide::BID => self
                .cold_bid_map
                .range_mut(..)
                .try_fold((order.volume, Value::ZERO), lambda),
        } {
            Ok((vol, val)) => (order.volume - vol, val),
            Err((vol, val)) => (order.volume - vol, val),
        };

        order.filled_volume = Cell::new(new_filled_vol);
        order.filled_value = Cell::new(new_filled_val);
    }

    /* fn hot_set_index_to_price(&self, index: usize) -> u64 {
            index as u64 + (self.hot_price_midpoint - self.hot_price_range)
        }

        fn hot_set_price_to_index(&self, price: u64) -> usize {
            (price - (self.hot_price_midpoint - self.hot_price_range)) as usize
        }
    */
    fn get_or_create_orderbucket(&mut self, order: &Order) -> &mut OrderBucket {
        // Handling only for cold orders, hot orders are missing
        let order_map = match order.side {
            OrderSide::ASK => &mut (self.cold_ask_map),
            OrderSide::BID => &mut (self.cold_bid_map),
        };

        order_map
            .entry(order.limit)
            .or_insert_with(|| OrderBucket::new(order.limit))
    }

    pub fn insert(&mut self, mut order: Order) {
        self.match_order(&mut order);

        if !order.is_filled() {
            if self.is_hot(order.limit) {
                /*
                if order.side == OrderSide::ASK {
                    self.hot_ask_set
                        .set(self.hot_set_price_to_index(order.limit), true);
                    let rc = Rc::new(order);
                    self.hot_map.insert(self.hot_set_price_to_index(order.limit) as u64, rc);

                } else {
                    self.hot_bid_set
                        .set(self.hot_set_price_to_index(order.limit), true);
                }
                */
            } else {
                self.get_or_create_orderbucket(&order).insert_order(order);
            }
        }
    }
}

pub fn first_entry(vec: BitVec<u32>) -> Option<u32> {
    vec.blocks()
        .enumerate()
        .filter(|(_n, b)| *b != 0u32)
        .map(|(n, b)| (n as u32) * 32 + b.trailing_zeros())
        .next()
}
