use crate::order::*;
use crate::primitives::*;

use intmap::IntMap;
use std::cmp::Ordering;
use std::collections::{HashMap, VecDeque};
use std::hash::{Hash, Hasher};
use std::rc::{Rc, Weak};


const DEFAULT_CAPACITY: usize =  2^8 - 1;

/// A sorted queue of orders that have all the same limit price

pub struct OrderBucket {
    pub price: Price,
    pub total_volume: Volume,
    pub size: usize,

    order_queue: VecDeque<Weak<Order>>,
    order_map: IntMap<Rc<Order>>,
}

impl PartialOrd for OrderBucket {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.price.cmp(&other.price))
    }
}

impl PartialEq for OrderBucket {
    fn eq(&self, other: &Self) -> bool {
        self.price == other.price
    }
}

impl OrderBucket {
    pub fn new(price: Price) -> OrderBucket {
        OrderBucket {
            price: price,
            total_volume: Volume::new(0),
            size: 0,
            order_queue: VecDeque::with_capacity(DEFAULT_CAPACITY),
            order_map: IntMap::with_capacity(DEFAULT_CAPACITY),
        }
    }

    pub fn insert_order(&mut self, order: Order) {
        //let now = Instant::now();
        let order_rc = Rc::new(order);

        self.size += 1;
        self.total_volume += order_rc.volume;

        self.order_queue.push_back(Rc::downgrade(&order_rc));
        // let t1 = now.elapsed();
        self.order_map.insert(order_rc.id, order_rc);
        // println!("Order insertion: Queue: {}, Map: {}", t1.as_nanos(), (now.elapsed()-t1).as_nanos());
    }

    pub fn remove_order(&mut self, id: &u64) -> Option<Volume> {
        match self.order_map.remove(*id) {
            Some(order) => {
                order.cancel();
                self.size -= 1;
                self.total_volume -= order.remaining_volume();
                Some(order.remaining_volume())
            }
            None => None,
        }
    }
    /// Match as many orders as possible with a given amount of volume
    ///
    /// #Returns how much volume was matched
    pub fn match_orders(&mut self, volume: Volume) -> (Volume) {
        let mut unmatched_volume = volume;
        while unmatched_volume.get() > 0 && !self.order_queue.is_empty() {
            match self.order_queue.front().unwrap().upgrade() {
                Some(order) => {
                    //  println!("Trying to match with {:?}", order);
                    let filled_volume = order.fill(unmatched_volume, self.price);
                    unmatched_volume -= filled_volume;
                    self.total_volume -= filled_volume;
                    if order.is_filled() {
                        self.order_queue.remove(0);
                        self.size -= 1;
                    }
                }

                //Remove elements that no longer exist
                None => {
                    self.order_queue.remove(0);
                }
            }
        }

        volume - unmatched_volume
    }
}
