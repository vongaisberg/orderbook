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
use std::rc::Rc;
use std::time::SystemTime;
use std::time::{Duration, Instant};

use std::{thread, time};

fn main() {
    /*     let mut vec = BitVec::from_elem(700, true);
    vec.set(0, false);
    vec.set(1, false);
    println!("First element: {}", order_book::first_entry(vec).unwrap()); */

    //test_hot_set_index();
    benchmark_order_book();
    //test_order_book();
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
            bucket.insert_order(Rc::downgrade(&Rc::new(Order::new(
                Price::new(500),
                Volume::new(20),
                OrderSide::ASK,
                Some(callback),
                false,
            ))));
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
            bucket.match_orders(&Volume::new(5));
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

fn test_hot_set_index() {
    for x in 1..10 {
        let book = OrderBook::new();

        println!(
            "x={:?}, result={:?}",
            x,
            book.hot_set_price_to_index(&Order::new(
                book.hot_set_index_to_price(x, &OrderSide::BID),
                Volume::ZERO,
                OrderSide::BID,
                None,
                false
            ))
        );
    }
}

fn benchmark_order_book() {
    let mut rng = rand::thread_rng();
    let mut book = OrderBook::new();

    const loops:usize = 10_000_000;

    let mut r1 = Vec::<u64>::with_capacity(loops);

    let mut r2 = Vec::<bool>::with_capacity(loops);

    let mut r3 = Vec::<bool>::with_capacity(loops);

    let mut r4 = Vec::<bool>::with_capacity(loops);

    let mut r5 = Vec::<u64>::with_capacity(loops);

//let mut orders = <std::vec::Vec<order::Order>>::new();




    for x in 0..loops {
        r1.insert(x, rng.gen_range(1, 750));
        r2.insert(x, rng.gen_range(0u64, 2u64) == 0);
       //r2.insert(x, r1[x] >375);

        //cancel?
        r3.insert(x, rng.gen_range(0, 18) < 10);

        // GTC?
        r4.insert(x, rng.gen_range(0, 12) < 3);
        
        //volume
        r5.insert(x, rng.gen_range(1, 10));

        
    }

    let mut now = Instant::now();

    for x in 0..2000 {
        book.insert_order( Order {
            side: if rng.gen_range(0u64, 2u64) == 0 {
                OrderSide::ASK
            } else {
                OrderSide::BID
            },
            limit: Price::new(rng.gen_range(1, 750)),
            volume: Volume::new(rng.gen_range(1, 10)),
            id: x as u64,
            //callback: Some(callback),
            callback: None,
            filled_volume: Cell::new(Volume::ZERO),
            filled_value: Cell::new(Value::ZERO),
            immediate_or_cancel: rng.gen_range(0, 12) < 3,
        });
    }

   

    println!("Time for order placement: {}", now.elapsed().as_millis());
    now = Instant::now();
    let mut y = 0;
    for x in 0..(loops as u64) {
        //  println!("x={:?}", x);
        if r3[x as usize] {
            // println!("removing");
            book.remove_order((loops as u64) + x-1);
        //println!("removed");
        } else {
            //println!("inserting");
            book.insert_order(Order {
                side: if r2[x as usize] {
                    OrderSide::ASK
                } else {
                    OrderSide::BID
                },
                limit: Price::new(r1[x as usize]),
                volume: Volume::new(r5[x as usize]),
                id:loops as u64 + x as u64,
                //callback: Some(callback),
                callback: None,
                filled_volume: Cell::new(Volume::ZERO),
                filled_value: Cell::new(Value::ZERO),
                immediate_or_cancel: r4[x as usize],
            });
            //println!("inserted");
        }
    }
    println!("Time for orderbook change: {}, Mtps: {}", now.elapsed().as_millis(),  (loops as f32 / 1_000f32)/(now.elapsed().as_millis() as f32));
}

fn benchmark_order_book2() {
    let mut book = OrderBook::new();
/**
    for x in 1..1000 {
        book.insert_order(Order::new(
            Price::new(x),
            Volume::new(10_000),
            OrderSide::ASK,
            None,
            false,
        ));
    }
**/
    let mut now = Instant::now();

    for x in 0..30 {
        thread::sleep(Duration::from_millis(50));
        now = Instant::now();
        for y in 0..1000000 {
            book.insert_order(Order::new(
                Price::new(y%750+1),
                Volume::new(1),
                OrderSide::BID,
                None,
                false,
            ));
        }

        println!("Matching at price={:?} took {:?}ms", x, now.elapsed().as_millis());
        
    }

    let ten_millis = time::Duration::from_millis(100000);

thread::sleep(ten_millis);
}

fn test_order_book() {
    let mut book = OrderBook::new();

    for x in 5..10 {
        book.insert_order(Order::new(
            Price::new(x),
            Volume::new(1),
            OrderSide::ASK,
            Some(callback),
            false,
        ));
    }
    println!("t");
    book.insert_order(Order::new(
        Price::new(8),
        Volume::new(1),
        OrderSide::BID,
        Some(callback),
        false,
    ));
}
