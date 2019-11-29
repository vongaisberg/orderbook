//mod order_bucket;
extern crate bit_vec;

use crate::order::*;
use crate::order_bucket::*;
use crate::primitives::*;

use bit_vec::BitVec;

use std::collections::HashMap;

use intmap::IntMap;
use std::cell::Cell;
use std::collections::btree_map::BTreeMap;
use std::collections::btree_map::RangeMut;
use std::ops::Range;

use std::rc::Rc;
use std::result::Result::*;

const DEFAULT_HOT_PRICE_RANGE: Price = Price { val: 1024 };
const DEFAULT_HOT_PRICE_BASE: Price = Price { val: 2048 };
const DEFAULT_HOT_MAP_CAPACITY: usize = 128;

pub struct OrderBook {
    hot_ask_set: BitVec,
    hot_bid_set: BitVec,

    hot_price_base: Price,
    hot_price_range: Price,

    /// Store hot orders indexed by hot_set_id
    /// TODO: Switch to a more efficient implementation (IntMap)
    hot_ask_map: HashMap<u64, OrderBucket>,
    hot_bid_map: HashMap<u64, OrderBucket>,

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
            hot_ask_set: BitVec::from_elem((DEFAULT_HOT_PRICE_RANGE).get() as usize, false),
            hot_bid_set: BitVec::from_elem((DEFAULT_HOT_PRICE_RANGE).get() as usize, false),

            hot_ask_map: HashMap::with_capacity(DEFAULT_HOT_MAP_CAPACITY),
            hot_bid_map: HashMap::with_capacity(DEFAULT_HOT_MAP_CAPACITY),

            hot_price_base: DEFAULT_HOT_PRICE_BASE,
            hot_price_range: DEFAULT_HOT_PRICE_RANGE,
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
        price >= self.hot_price_base && price < self.hot_price_base + self.hot_price_range
    }

    /// Try to instantly match an order as it is coming in
    ///
    ///
    /// The incoming order will possibly take liquidity from the orderbook.
    ///
    ///
    fn match_order(&mut self, order: &mut Order) {
        // First try to match the order with existing hot orders

        let hot_set = match order.side {
            //ASK orders match with bid orders
            OrderSide::ASK => &self.hot_bid_set,

            //ASK orders match with bid orders
            OrderSide::BID => &self.hot_ask_set,
        };

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
                .cold_bid_map
                .range_mut(..)
                .try_rfold((order.volume, Value::ZERO), lambda),
            OrderSide::BID => self
                .cold_ask_map
                .range_mut(..)
                .try_fold((order.volume, Value::ZERO), lambda),
        } {
            Ok((vol, val)) => (order.volume - vol, val),
            Err((vol, val)) => (order.volume - vol, val),
        };

        order.filled_volume = Cell::new(new_filled_vol);
        order.filled_value = Cell::new(new_filled_val);
        if new_filled_vol.get() > 0 {
            order.notify()
        }
    }

    fn hot_set_index_to_price(&self, index: usize, side: OrderSide) -> Price {
        match side {
            //For existing ask orders, we are interested in the lowest offer
            OrderSide::ASK => Price::new(index as u64) + (self.hot_price_base),
            //For existing bid orders, we are interested in the highest offer
            OrderSide::BID => {
                self.hot_price_range - (Price::new(index as u64) + (self.hot_price_base))
            }
        }
    }

    fn hot_set_price_to_index(&self, order: Order) -> usize {
        match order.side {
            OrderSide::ASK => (order.limit - self.hot_price_base).get() as usize,
            OrderSide::BID => {
                (self.hot_price_base - (order.limit - self.hot_price_base)).get() as usize
            }
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

    fn get_or_create_hot_orderbucket(&mut self, order: &Order) -> &mut OrderBucket {
        // Handling only for cold orders, hot orders are missing
        let order_map = match order.side {
            OrderSide::ASK => {
                self.hot_ask_set
                    .set((order.limit - self.hot_price_base).get() as usize, true);
                &mut (self.hot_ask_map)
            }
            OrderSide::BID => {
                self.hot_bid_set
                    .set((order.limit - self.hot_price_base).get() as usize, true);
                &mut (self.hot_bid_map)
            }
        };

        order_map
            .entry(order.limit.get())
            .or_insert_with(|| OrderBucket::new(order.limit))
    }

    pub fn insert_order(&mut self, mut order: Order) {
        self.match_order(&mut order);

        if !order.is_filled() {
            if order.immediate_or_cancel {
                order.cancel();
            } else {
                let order_rc = Rc::new(order);

                if self.is_hot(order_rc.limit) {
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
                    self.get_or_create_cold_orderbucket(&order_rc)
                        .insert_order(Rc::downgrade(&order_rc));
                }
                self.order_map.insert(order_rc.id, order_rc);
            }
        }
    }
    pub fn remove_order(&mut self, id: u64) {
        self.order_map.remove(id);
    }
}

pub fn first_entry(vec: BitVec<u32>) -> Option<u32> {
    vec.blocks()
        .enumerate()
        .filter(|(_n, b)| *b != 0u32)
        .map(|(n, b)| (n as u32) * 32 + b.trailing_zeros())
        .next()
}
