use crate::order::*;
use crate::primitives::*;

use std::cmp::Ordering;
use std::collections::VecDeque;
//use std::hash::{Hash, Hasher};
use std::rc::{Rc, Weak};

const DEFAULT_CAPACITY: usize = 2 ^ 8 - 1;

/// A sorted queue of orders that have all the same limit price

pub struct OrderBucket {
    pub price: Price,
    pub total_volume: Volume,
    pub size: usize,

    /// The inner ```Weak``` is for orders that get canceled from the orderbook (manual cancel)
    /// The outer ```Weak``` is for orders that get canceled from the order_bucket (canceled because they were filled)
    //map order_queue: VecDeque<Weak<Weak<Order>>>,
    order_queue: VecDeque<Weak<Order>>,
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
    pub fn new(price: Price) -> OrderBucket {
        OrderBucket {
            price: price,
            total_volume: Volume::new(0),
            size: 0,
            order_queue: VecDeque::with_capacity(DEFAULT_CAPACITY),
            //map   order_map: HashMap::with_capacity(DEFAULT_CAPACITY),
        }
    }

    pub fn insert_order(&mut self, order: &Rc<Order>) {
        self.size += 1;
        self.total_volume += order.volume;

        //map   self.order_queue.push_back(Rc::downgrade(&order_rc));
        self.order_queue.push_back(Rc::downgrade(order));
        //map  self.order_map.insert(order_up.id, order_rc);
        //  println!("Order insertion: Queue: {}, Map: {}", t1.as_nanos(), (now.elapsed()-t1).as_nanos());
    }

    /*map
    pub fn remove_order(&mut self, id: &u64) -> Option<Volume> {
        match self.order_map.remove(id) {
            Some(order) => match order.upgrade() {
                Some(order) => {
                    order.cancel();
                    self.size -= 1;
                    self.total_volume -= order.remaining_volume();
                    Some(order.remaining_volume())
                }
                None => None,
            },
            None => None,
        }
    }

    */
    /// Match as many orders as possible with a given amount of volume
    ///
    /// #Returns how much volume was matched
    pub fn match_orders(&mut self, volume: &Volume) -> Volume {
        let mut unmatched_volume = volume.clone();
        while unmatched_volume.get() > 0 && !self.order_queue.is_empty() {
            println!("Found something to match with");
            match self.order_queue.front().unwrap().upgrade() {
                Some(order) => {
                    //map match order.upgrade() {
                    //map    Some(order) => {
                    println!("Trying to match with {:?}", order);
                    let filled_volume = order.fill(unmatched_volume, self.price);
                    unmatched_volume -= filled_volume;
                    self.total_volume -= filled_volume;
                    if order.is_filled() {
                        self.order_queue.remove(0);
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

                //Remove elements that were removed from the bucket using ```remove_order```
                None => {
                    self.order_queue.remove(0);
                }
            }
        }

        *volume - unmatched_volume
    }
}
