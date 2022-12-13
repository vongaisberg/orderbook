//use crate::order_handling::deletable_list::*;
use crate::order_handling::order::*;
//use crate::primitives::*;
extern crate libc;

use std::collections::VecDeque;
use std::{cmp::Ordering, time};
//use std::hash::{Hash, Hasher};
use std::rc::{Rc, Weak};

use arr_macro::arr;
use fxhash::FxBuildHasher;
use linked_hash_map::LinkedHashMap;

/// A sorted queue of orders that have all the same limit price

pub struct OrderBucket {
    pub price: u64,
    pub total_volume: u64,
    pub size: usize,

    /// The inner ```Weak``` is for orders that get canceled from the orderbook (manual cancel)
    /// The outer ```Weak``` is for orders that get canceled from the order_bucket (canceled because they were filled)
    //map order_queue: VecDeque<Weak<Weak<Order>>>,
    pub order_queue: LinkedHashMap<u64, Order, FxBuildHasher>,
    //map order_map: HashMap<u64, Rc<Weak<Order>>>,
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
    pub fn new(price: u64) -> OrderBucket {
        OrderBucket {
            price: price,
            total_volume: 0,
            size: 0,
            order_queue: LinkedHashMap::with_hasher(FxBuildHasher::default()),
            //map   order_map: HashMap::with_capacity(DEFAULT_CAPACITY),
        }
    }

    pub fn insert_order(&mut self, order: Order) {
        self.size += 1;
        self.total_volume += order.remaining_volume();

        //map   self.order_queue.push_back(Rc::downgrade(&order_rc));
        self.order_queue.insert(order.id, order);
        //map  self.order_map.insert(order_up.id, order_rc);
        //  println!("Order insertion: Queue: {}, Map: {}", t1.as_nanos(), (now.elapsed()-t1).as_nanos());
    }

    pub fn remove_order(&mut self, id: u64) -> u64 {
        match self.order_queue.remove(&id) {
            Some(order) => {
                order.cancel();
                self.size -= 1;
                self.total_volume -= order.remaining_volume();
                order.remaining_volume()
            }
            None => 0,
        }
    }

    /// Match as many orders as possible with a given amount of volume
    ///
    /// #Returns how much volume was matched
    pub fn match_orders(&mut self, volume: u64) -> u64 {
        let mut unmatched_volume = volume;
        // println!(
        //     "Matching order. Unmatched Volume: {}, Bucket Volume: {}",
        //     unmatched_volume, self.total_volume
        // );
        // std::thread::sleep(time::Duration::from_millis(100));
        while unmatched_volume > 0 && !self.order_queue.is_empty() {
            /*
            let sum: u64 = self
                .order_queue
                .iter()
                .map(|o| *(o.remaining_volume()))
                .sum();

            if sum != *self.total_volume {
                println!("Order queue misaccounting");
                println!("");
            }

            else {
               println!("Unmatched Volume: {}, Bucket Volume: {}", *unmatched_volume, *self.total_volume);
            }
            */
            let (id, order) = self.order_queue.front().unwrap();
            // println!("Matching with: {:?}", order);

            //map match order.upgrade() {
            //map    Some(order) => {
            let filled_volume = order.fill(unmatched_volume, self.price);
            // println!("Matched with: {:?}", order);
            unmatched_volume -= filled_volume;
            self.total_volume -= filled_volume;
            if order.is_filled() {
                // println!("Order is filled, mark it as canceled");
                order.is_canceled.set(true);
                // println!("Marked as canceled, removing from bucket");
                self.order_queue.pop_front();
                self.size -= 1;
            }

            /*map   }
                None => {
                    //Remove elements that were removed from the book
                    self.order_queue.remove(0);
                }
            }
            */
        }

        volume - unmatched_volume
    }

    // pub fn print_list(&self) -> String {
    //     self.order_queue
    //         .iter()
    //         .map(|order| {
    //             format!(
    //                 "{}({}/{}){:?} ->",
    //                 order.id,
    //                 order.remaining_volume().0,
    //                 order.volume.0,
    //                 order.side
    //             )
    //         })
    //         .fold_first(|a, b| a + &b)
    //         .unwrap_or("Empty".to_string())
    // }
}
