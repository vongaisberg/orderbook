//use crate::order_handling::deletable_list::*;
use crate::order_handling::order::*;
//use crate::primitives::*;
extern crate libc;

use std::borrow::{Borrow, BorrowMut};
use std::collections::hash_map::OccupiedEntry;
use std::collections::{LinkedList, VecDeque};
use std::ptr::NonNull;
use tokio::sync::mpsc::Sender;
use std::{cmp::Ordering, time};
//use std::hash::{Hash, Hasher};
use std::rc::{Rc, Weak};

use arr_macro::arr;
use fxhash::FxBuildHasher;
use linked_hash_map::LinkedHashMap;

use super::order_book::OrderBook;

/// A sorted queue of orders that have all the same limit price

pub struct OrderBucket {
    pub price: u64,
    pub len: usize,

    pub head: Option<NonNull<Box<StandingOrder>>>,
    pub tail: Option<NonNull<Box<StandingOrder>>>,
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
            price,
            len: 0,
            head: None,
            tail: None, //map   order_map: HashMap::with_capacity(DEFAULT_CAPACITY),
        }
    }

    fn push_back(&mut self, mut order: NonNull<Box<StandingOrder>>) {
        unsafe {
            order.as_mut().next = None;
            order.as_mut().prev = self.tail;
            let node = Some(order);

            match self.tail {
                None => self.head = node,
                // Not creating new mutable (unique!) references overlapping `element`.
                Some(mut tail) => tail.as_mut().next = node,
            }

            self.tail = node;
            self.len += 1;
        }
    }

    fn pop_front(&mut self) {
        self.head.map(|node| unsafe {
            let node = node.as_ref();
            self.head = node.next;

            match self.head {
                None => self.tail = None,
                // Not creating new mutable (unique!) references overlapping `element`.
                Some(mut head) => head.as_mut().prev = None,
            }

            self.len -= 1;
            node
        });
    }

    pub fn is_empty(&self) -> bool {
        assert!(self.head.is_none() == self.tail.is_none());
        self.head.is_none()
    }

    pub fn insert_order(&mut self, order: NonNull<Box<StandingOrder>>) {
        self.push_back(order);

        //  println!("Order insertion: Queue: {}, Map: {}", t1.as_nanos(), (now.elapsed()-t1).as_nanos());
    }

    /// Match as many orders as possible with a given amount of volume
    ///
    /// #Returns how much volume was matched
    pub async fn match_orders(
        volume: u64,
        book: &mut OrderBook,
        best_price: u64,
    ) -> Option<(u64, Option<u64>)> {
        let bucket = &mut book.bucket_array[best_price as usize];

        if bucket.is_empty() {
            return None;
        }
        // std::thread::sleep(time::Duration::from_millis(100));
        let order = unsafe { bucket.head.unwrap().as_mut() };
        // println!("Matching with: {:?}", order);

        let filled_volume = order.fill(volume, bucket.price, book.get_sender(order.participant_id)).await;
        // println!("Matched with: {:?}", order);
        let canceled_order = if order.is_filled() {
            // println!("Marked as filled, removing from bucket");
            Some(order.id)
        } else {
            None
        };

        Some((filled_volume, canceled_order))
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
