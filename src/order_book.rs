//mod order_bucket;
extern crate bit_vec;

use crate::order::*;
use crate::order_bucket::*;
use crate::primitives::*;

use bit_vec::BitVec;

use std::collections::HashMap;

use intmap::IntMap;
use std::cell::{Cell, RefCell};
use std::collections::btree_map::BTreeMap;
use std::collections::hash_set::HashSet;
use std::collections::binary_heap::BinaryHeap;
//use std::collections::btree_map::RangeMut;
//use std::ops::Range;
use std::ops::Neg;

use std::rc::Rc;
use std::result::Result::*;

const DEFAULT_HOT_PRICE_RANGE: Price = Price { val: 9750 };
const DEFAULT_HOT_PRICE_BASE: Price = Price { val: 500 };
const DEFAULT_HOT_MAP_CAPACITY: usize = 128;

pub struct OrderBook {
    hot_ask_set: RefCell<BitVec>,
    hot_bid_set: RefCell<BitVec>,

    hot_price_base: Price,
    hot_price_range: Price,

    // The first (closest to the midpoint) filled bucket of both of the hot sets
    hot_ask_set_start: RefCell<usize>,
    hot_bid_set_start: RefCell<usize>,

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
    order_map: HashMap<u64, Rc<Order>>,
}

impl Default for OrderBook {
    fn default() -> OrderBook {
        OrderBook {
            // The size has to be hot_price_range + 1, because the hot_price_base has to be included xD
            hot_ask_set: RefCell::new(BitVec::from_elem(
                (DEFAULT_HOT_PRICE_RANGE + Price::new(1)).get() as usize,
                false,
            )),
            hot_bid_set: RefCell::new(BitVec::from_elem(
                (DEFAULT_HOT_PRICE_RANGE + Price::new(1)).get() as usize,
                false,
            )),

            hot_ask_set_start: RefCell::new(0),
            hot_bid_set_start: RefCell::new(0),

            hot_ask_map: HashMap::with_capacity(DEFAULT_HOT_MAP_CAPACITY),
            hot_bid_map: HashMap::with_capacity(DEFAULT_HOT_MAP_CAPACITY),

            hot_price_base: DEFAULT_HOT_PRICE_BASE,
            hot_price_range: DEFAULT_HOT_PRICE_RANGE,
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

    fn is_hot(&self, price: Price) -> bool {
        false
      //  price >= self.hot_price_base && price < self.hot_price_base + self.hot_price_range
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

        // First try to match the order with existing hot orders

        /*
        let (hot_set, hot_map, mut hot_set_start) = match order.side {
            //ASK orders match with bid orders
            OrderSide::ASK => (
                &self.hot_bid_set,
                &mut self.hot_bid_map,
                self.hot_bid_set_start.borrow_mut(),
            ),

            //BID orders match with ask orders
            OrderSide::BID => (
                &self.hot_ask_set,
                &mut self.hot_ask_map,
                self.hot_ask_set_start.borrow_mut(),
            ),
        };
*/
        //How much volume is left in the order and how much was payed for the already matched volume
        let (mut vol, mut val) = (order.volume, Value::ZERO);

        //Base price and range of the hot set
        let (hot_base, hot_range) = (self.hot_price_base.clone(), self.hot_price_range.clone());

        //List of hot bucket indices which had something in them but are now empty
        let mut empty_buckets = HashSet::<usize>::new();

// The commented stuff is used for hot orders

        let cold_map = match order.side {

            //incoming ASK orders match with old BID orders
            OrderSide::ASK => self.cold_bid_map,
            OrderSide::BID => self.cold_ask_map,
        };







        /* let (new_filled_vol, new_filled_val) = match hot_set

        
            .borrow_mut()
            .iter()
            .enumerate()
            .skip(*hot_set_start)
            .filter(|(_n, b)| *b)
            .map(|(n, _b)| n)
            .try_for_each(|n| {
                // println!("Looking at hot_set entry {:?}", n);
                *hot_set_start = n;
                //  println!("index={:?}", Self::hot_set_index_to_price_by_range(n, &order.side.clone().neg(), hot_base, hot_range));
                let bucket = hot_map
                    .get_mut(
                        &Self::hot_set_index_to_price_by_range(
                            n,
                            &order.side.clone().neg(),
                            hot_base,
                            hot_range,
                        )
                        .get(),
                    )
                    .unwrap();

                if order.matches_with(&bucket.price) {
                    let vol_matched = bucket.match_orders(&vol);

                    if bucket.total_volume.get() == 0 {
                        empty_buckets.insert(n);
                    }

                    val += vol_matched * bucket.price;
                    vol -= vol_matched;
                    if vol_matched == vol.clone() {
                        //Everything was matched, shortcut the try_for_each
                        Err(())
                    } else {
                        //Not everything was matched, continue
                        Ok(())
                    }
                } else {
                    //Can't match any more, shortcircuit
                    Err(())
                }
            }) {
            Err(_) => (order.volume - vol, val),
            Ok(_) => { 
                match match order.side {
                    // Using try_fold for the short_circuiting feature, when the order is already filled
                    // Err(x) and Ok(x) both mean that the order has been filled as much as possible and x volume remains in it
                    OrderSide::ASK => self
                        .cold_bid_map
                        .range_mut(..)
                        .try_rfold((vol, val), lambda),
                    OrderSide::BID => self.cold_ask_map.range_mut(..).try_fold((vol, val), lambda),
                } {
                    Ok((vol, val)) => (order.volume - vol, val),
                    Err((vol, val)) => (order.volume - vol, val),
                };

        // This is needed when we use hot orders again
        
        
            }
        };



        //Mark all newly empty buckets
        for n in empty_buckets.iter() {
            hot_set.borrow_mut().set(*n, false);
        }

         */

        order.filled_volume = Cell::new(new_filled_vol);
        order.filled_value = Cell::new(new_filled_val);
        if new_filled_vol.get() > 0 {
            order.notify()
        }
    }

    pub fn hot_set_index_to_price(&self, index: usize, side: &OrderSide) -> Price {
        match side {
            //For existing ask orders, we are interested in the lowest offer
            //Index shifted up by hot_price_base
            OrderSide::ASK => Price::new(index as u64) + (self.hot_price_base),
            //For existing bid orders, we are interested in the highest offer
            OrderSide::BID => {
                //(Upper limit minus index) shift up by hot_price_base
                (self.hot_price_range - Price::new(index as u64)) + self.hot_price_base
            }
        }
    }

    pub fn hot_set_index_to_price_by_range(
        index: usize,
        side: &OrderSide,
        hot_price_base: Price,
        hot_price_range: Price,
    ) -> Price {
        match side {
            //For existing ask orders, we are interested in the lowest offer
            OrderSide::ASK => Price::new(index as u64) + (hot_price_base),
            //For existing bid orders, we are interested in the highest offer
            OrderSide::BID => (hot_price_range - Price::new(index as u64)) + hot_price_base,
        }
    }

    pub fn hot_set_price_to_index(&self, order: &Order) -> usize {
        match order.side {
            OrderSide::ASK => (order.limit - self.hot_price_base).get() as usize,
            OrderSide::BID => {
                (self.hot_price_range - (order.limit - self.hot_price_base)).get() as usize
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
        let index = self.hot_set_price_to_index(order);
        let order_map = match order.side {
            OrderSide::ASK => {
                self.hot_ask_set.borrow_mut().set(index, true);
                if *self.hot_ask_set_start.borrow() > index
                    && index <= self.hot_price_range.get() as usize
                    && false
                {
                    *self.hot_ask_set_start.borrow_mut() = index;
                }
                &mut (self.hot_ask_map)
            }
            OrderSide::BID => {
                self.hot_bid_set.borrow_mut().set(index, true);
                if *self.hot_bid_set_start.borrow() > index
                    && index <= self.hot_price_range.get() as usize
                    && false
                {
                    *self.hot_bid_set_start.borrow_mut() = index;
                }
                &mut (self.hot_bid_map)
            }
        };

        order_map
            .entry(order.limit.get())
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

                if self.is_hot(order_rc.limit) {
                    self.get_or_create_hot_orderbucket(&order_rc)
                        .insert_order(Rc::downgrade(&order_rc));
                } else {
                    self.get_or_create_cold_orderbucket(&order_rc)
                        .insert_order(Rc::downgrade(&order_rc));
                }
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
