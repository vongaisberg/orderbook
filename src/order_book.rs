//mod order_bucket;
extern crate bit_vec;

use crate::order::*;
use crate::order_bucket::*;
use crate::primitives::*;

use bit_vec::BitVec;

use std::collections::HashMap;

use std::cell::Cell;
use std::collections::btree_map::BTreeMap;


//use std::collections::btree_map::RangeMut;
//use std::ops::Range;

use std::rc::Rc;
use std::result::Result::*;

pub struct OrderBook {
    /// Store ask orders sorted by price
    cold_ask_map: BTreeMap<Price, OrderBucket>,

    /// Store bid orders sorted by price
    cold_bid_map: BTreeMap<Price, OrderBucket>,
    // ask_depth_fenwick_tree: Vec<u64>,
    // bid_depth_fenwick_tree: Vec<u64>
    order_map: HashMap<u64, Rc<Order>>,
}

impl Default for OrderBook {
    fn default() -> OrderBook {
        OrderBook {
            cold_ask_map: BTreeMap::new(),
            cold_bid_map: BTreeMap::new(),

            order_map: HashMap::with_capacity(20_000_000),
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
        // Match as much as possible from vol with bucket if price is right
        // Return the volume that is left over and the value that was payed for all matched volume
        let lambda = |(vol, val): (Volume, Value), (price, bucket): (&Price, &mut OrderBucket)| {
            if order.matches_with(&price) {
                let vol_matched = bucket.match_orders(&vol);
                if vol_matched == vol.clone() {
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

        //How much volume is left in the order and how much was payed for the already matched volume
        let (vol, val) = (order.volume, Value::ZERO);

        let cold_map = match order.side {
            //incoming ASK orders match with old BID orders
            OrderSide::ASK => &mut self.cold_bid_map,
            OrderSide::BID => &mut self.cold_ask_map,
        };

        let (new_filled_vol, new_filled_val) = match 
                    // Using try_fold for the short_circuiting feature, when the order is already filled
                    // Err(x) and Ok(x) both mean that the order has been filled as much as possible and x volume remains in it
                    cold_map
                        .range_mut(..)
                        .try_rfold((vol, val), lambda)
                   {
                    Ok((vol, val)) => (order.volume - vol, val),
                    Err((vol, val)) => (order.volume - vol, val),
                };

        order.filled_volume = Cell::new(new_filled_vol);
        order.filled_value = Cell::new(new_filled_val);
        if new_filled_vol.get() > 0 {
            order.notify()
        }
    }

    fn get_or_create_cold_orderbucket(&mut self, order: &Order) -> &mut OrderBucket {
        // Handling only for cold orders, hot orders are missing
        let order_map = match order.side {
            OrderSide::ASK => &mut (self.cold_ask_map),
            OrderSide::BID => &mut (self.cold_bid_map),
        };

        order_map
            .entry(order.limit)
            .or_insert_with(|| OrderBucket::new(order.limit))
    }

    pub fn insert_order(&mut self, mut order: Order) {
        // println!("matching");
        self.match_order(&mut order);
        // println!("matched");

        if !order.is_filled() {
            if order.immediate_or_cancel {
                order.cancel();
            } else {
                let order_rc = Rc::new(order);

                self.get_or_create_cold_orderbucket(&order_rc)
                    .insert_order(Rc::downgrade(&order_rc));

                self.order_map.insert(order_rc.id, order_rc);
            }
        }
    }
    pub fn remove_order(&mut self, id: u64) {
        self.order_map.remove(&id);
    }
}

pub fn first_entry(vec: BitVec<u32>) -> Option<u32> {
    vec.blocks()
        .enumerate()
        .filter(|(_n, b)| *b != 0u32)
        .map(|(n, b)| (n as u32) * 32 + b.trailing_zeros())
        .next()
}
