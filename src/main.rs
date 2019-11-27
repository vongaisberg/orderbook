//extern crate rand;

mod bit_set;
mod order;
mod order_book;
mod order_bucket;
mod primitives;

use crate::order::*;
use crate::order_book::*;
use crate::order_bucket::*;
use crate::primitives::*;
use bit_vec::BitVec;
use rand::Rng;
use std::cell::Cell;
use std::collections::HashSet;
use std::time::SystemTime;
use std::time::{Duration, Instant};

fn main() {
    /*     let mut vec = BitVec::from_elem(700, true);
    vec.set(0, false);
    vec.set(1, false);
    println!("First element: {}", order_book::first_entry(vec).unwrap()); */

    test_order_book();
}

fn test_order_bucket() {
    let mut rng = rand::thread_rng();
    let mut bucket = OrderBucket::new(Price::new(500));
    let mut set_x = [0u64; 1000];
    let mut set_y = [0u64; 1000];
    let mut sort_x = [0usize; 1000];
    let mut sort_y = [0usize; 1000];

    for i in 0..1000 {
        set_x[i] = rng.gen();
        set_y[i] = rng.gen();
        sort_x[i] = rng.gen_range(0, 1000);
        sort_y[i] = rng.gen_range(0, 500);
    }

    let mut now = Instant::now();

    for x in 0..10 {
        for y in 0..10 {
            bucket.insert_order(Order::new(
                Price::new(500),
                Volume::new(20),
                OrderSide::ASK,
                Some(callback),
            ));
        }
    }

    println!("Time for order placement: {}", now.elapsed().as_millis());
    now = Instant::now();
    println!(
        "Bucket size:{}, total_volume:{:?}",
        bucket.size, bucket.total_volume
    );

    for x in 0..10 {
        for y in 0..5 {
            //bucket.remove_order(&(set_x[sort_x[x]] / 2 + set_y[sort_y[y]] / 2));
            bucket.match_orders(Volume::new(5));
            /* bucket.insert_order(Order {
                    side: OrderSide::ASK,
                    limit: Price::new(500),
                    volume: Volume::new(10),
                    id: set_x[sort_x[x]] / 2 + set_y[sort_y[y + 500]] / 2,
                    callback: Some(callback),
                    filled_volume: Cell::new(Volume::ZERO),
            filled_value: Cell::new(Value::ZERO),
                }) */
        }
    }

    println!(
        "Bucket size:{}, total_volume:{:?}",
        bucket.size, bucket.total_volume
    );

    println!("Time for orderbook change: {}", now.elapsed().as_millis());
    /*
       for i in 0..100 {
           bucket.match_orders(Volume::new(100));
       }
    */
    println!(
        "Bucket size:{}, total_volume:{:?}",
        bucket.size, bucket.total_volume
    );

    println!("Time: {}", now.elapsed().as_millis());
}

fn callback(event: OrderEvent) {
    println!("OrderEvent: {:?}", event)
}

fn test_order_book() {
    let mut rng = rand::thread_rng();
    let mut book = OrderBook::new();

    let mut now = Instant::now();

    for x in 0..100000 {
        book.insert(Order {
            side: if rng.gen_range(0u8, 1u8) == 0 {
                OrderSide::ASK
            } else {
                OrderSide::BID
            },
            limit: Price::new(rng.gen_range(1, 750)),
            volume: Volume::new(1),
            id: x,
            //callback: Some(callback),
            callback: None,
            filled_volume: Cell::new(Volume::ZERO),
            filled_value: Cell::new(Value::ZERO),
        })
    }

    println!("Time for order placement: {}", now.elapsed().as_millis());
    now = Instant::now();

    /*  for x in 100..100000 {
    book.insert(Order {
        side: OrderSide::BID,
        limit: Price::new(150),
        volume: Volume::new(10000),
        id: rng.gen(),
        callback: Some(callback),
        filled_volume: Cell::new(Volume::ZERO),
        filled_value: Cell::new(Value::ZERO),
    }); */

    println!("Time for orderbook change: {}", now.elapsed().as_millis());
}
