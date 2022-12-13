//extern crate rand;
#![allow(dead_code)]
#![allow(unused_variables)]
#![allow(unused_assignments)]
#![allow(unused_imports)]
#![feature(async_closure)]
#![feature(box_syntax)]
#![feature(dropck_eyepatch)]
#![feature(extend_one)]
#![feature(exact_size_is_empty)]

//mod bit_set;
pub mod exchange;
pub mod order_handling;

use crate::order_handling::order::*;
use crate::order_handling::order_book::*;
use crate::order_handling::order_bucket::OrderBucket;
use rand_distr::{Distribution, Normal};
use std::rc::Rc;

use rand::prelude::*;
use std::cell::Cell;
use std::cmp::max;

use std::{collections::HashMap, ptr::NonNull};

use std::time::{Duration, Instant};

use std::{thread, time};

use crate::exchange::account::*;
use crate::exchange::commands::*;
use crate::exchange::exchange::*;
use crate::order_handling::order_bucket::*;

use order_handling::{order, public_list::*};

fn main() {
    // let ob = OrderBucket::new(1);

    // let ex = Exchange::new();

    test_exchange();
    //println!("test")
    /*     let mut vec = BitVec::from_elem(700, true);
    vec.set(0, false);
    vec.set(1, false);
    println!("First element: {}", order_book::first_entry(vec).unwrap()); */

    //test_hot_set_index();
    // benchmark_order_book();
    //test_order_book();
}

const COUNT: usize = 100_000;
const SECOND_COUNT: usize = 1_000;

fn test_exchange() {
    let mut rng = thread_rng();
    let mut ex = Exchange::new();
    let mut canceled = 1u64;
    for e in 0..1 {
        let queue = unsafe {
            let mut arr: Box<[OrderCommand; COUNT]> = Box::new(std::mem::uninitialized());
            let mut i = 0;
            let normal =
                Normal::new(20f64, 5f64).expect("Normal distribution initialization error");
            for item in &mut arr[..] {
                // println!("OrderCommand: {}", i);

                if rng.gen_range(0u64..3u64) == 0 {
                    canceled += 1;
                    std::ptr::write(
                        item,
                        OrderCommand::Cancel(CancelCommand {
                            ticker: 0,
                            order_id: canceled,
                        }),
                    );
                } else {
                    std::ptr::write(
                        item,
                        OrderCommand::Trade(TradeCommand {
                            ticker: 0,
                            side: if rng.gen_range(0u64..2u64) == 0 {
                                OrderSide::ASK
                            } else {
                                OrderSide::BID
                            },
                            volume: rng.gen_range(1..10),
                            limit: normal.sample(&mut rng) as u64,
                            //immediate_or_cancel: rng.gen_range(0, 12) < 3,
                            immediate_or_cancel: false,
                        }),
                    );
                }
                i += 1;
            }
            arr
        };
        let now = Instant::now();
        for j in 0..SECOND_COUNT {
            for i in 0..COUNT {
                ex.trade(&queue[i]);
                for i in 1..11 {
                    // println!(
                    //     "Price: {}, Volume: {}, Number: {} {}",
                    //     i,
                    //     *ex.orderbooks[0].orders_array[i].total_volume,
                    //     ex.orderbooks[0].orders_array[i].size,
                    //     ex.orderbooks[0].orders_array[i].print_list()
                    // );
                }
                // println!(
                //     "Min ASK: {}, Max BID: {}\n",
                //     ex.orderbooks[0].min_ask_price, ex.orderbooks[0].max_bid_price
                // );
            }
        }
        println!(
            "Number of trades: {}, Milliseconds: {}, MTps: {}",
            COUNT * SECOND_COUNT,
            now.elapsed().as_millis(),
            ((COUNT as f32) * (SECOND_COUNT as f32))
                / ((now.elapsed().as_nanos() as f32) / 1_000_000_000f32)
                / 1_000_000f32
        );
    }

    for i in 1..21 {
        println!(
            "Price: {}, Volume: {}, Number: {}",
            i, ex.orderbooks[0].orders_array[i].total_volume, ex.orderbooks[0].orders_array[i].size
        );
    }
}

fn test_exchange_matching() {
    let mut ex = Exchange::new();
    let mut canceled = 1u64;
    let queue = [OrderCommand::Trade(TradeCommand {
        ticker: 0,
        side: OrderSide::BID,
        volume: 3,
        limit: 5,
        immediate_or_cancel: false,
    })];

    let now = Instant::now();
    for j in 0..SECOND_COUNT {
        for i in 0..COUNT {
            ex.trade(&queue[i]);
            for i in 1..11 {
                // println!(
                //     "Price: {}, Volume: {}, Number: {} {}",
                //     i,
                //     *ex.orderbooks[0].orders_array[i].total_volume,
                //     ex.orderbooks[0].orders_array[i].size,
                //     ex.orderbooks[0].orders_array[i].print_list()
                // );
            }
            // println!(
            //     "Min ASK: {}, Max BID: {}\n",
            //     ex.orderbooks[0].min_ask_price.0, ex.orderbooks[0].max_bid_price.0
            // );
            // println!("j: {}", j);
        }
    }
    println!(
        "Number of trades: {}, Milliseconds: {}, MTps: {}",
        COUNT * SECOND_COUNT,
        now.elapsed().as_millis(),
        ((COUNT as f32) * (SECOND_COUNT as f32))
            / ((now.elapsed().as_nanos() as f32) / 1_000_000_000f32)
            / 1_000_000f32
    );

    for i in 1..20 {
        println!(
            "Price: {}, Volume: {}, Number: {}",
            i, ex.orderbooks[0].orders_array[i].total_volume, ex.orderbooks[0].orders_array[i].size
        );
    }
}

#[test]
fn test_order() {
    /*
    let order = Order::new(Price::new(1), Volume::new(1), OrderSide::ASK, None, false);
    let filled_volume = order.fill(Volume::new(1), Price::new(1));
    assert_eq!(filled_volume, Volume::new(1));
    */
}

#[test]
fn test_order_bucket() {
    /*
    let mut rng = rand::thread_rng();
    let mut bucket = OrderBucket::new(Price::new(500));

    let mut now = Instant::now();

    let mut order_vec = Vec::new();

    for _y in 0..100 {
        let order = Rc::new(Order::new(
            Price::new(500),
            Volume::new(20),
            OrderSide::ASK,
            None,
            false,
        ));
        bucket.insert_order(&order);
        order_vec.push(order);
    }
    println!("Time for order placement: {}", now.elapsed().as_millis());
    now = Instant::now();
    println!(
        "Bucket size:{}, total_volume:{:?}",
        bucket.size, bucket.total_volume
    );

    for _x in 0..50 {
        //bucket.remove_order(&(set_x[sort_x[x]] / 2 + set_y[sort_y[y]] / 2));
        assert_eq!(bucket.match_orders(&Volume::new(5)), Volume::new(5));
        /* bucket.insert_order(Order {
                side: OrderSide::ASK,
                limit: Price::new(500),
                volume: Volume::new(10),
                id: set_x[sort_x[x]] / 2 + set_y[sort_y[y + 500]] / 2,
                callback: Some(callback),
                filled_volume: Cell::new(0),
        filled_value: Cell::new(Value::ZERO),
            }) */
    }

    assert_eq!(bucket.total_volume, Volume::new(1750));
    println!(
        "Bucket size:{}, total_volume:{:?}",
        bucket.size, bucket.total_volume
    );

    println!("Time for orderbook change: {}", now.elapsed().as_millis());
    println!(
        "Bucket size:{}, total_volume:{:?}",
        bucket.size, bucket.total_volume
    );
    println!("Time: {}", now.elapsed().as_millis());
    */
}

fn callback(event: OrderEvent) {
    println!("OrderEvent: {:?}", event)
}
#[test]
fn benchmark_order_book() {
    let mut rng = rand::thread_rng();
    let mut book = OrderBook::new();

    const LOOPS: usize = 10_000_000;

    let mut r1 = Vec::<u64>::with_capacity(LOOPS);

    let mut r2 = Vec::<bool>::with_capacity(LOOPS);

    let mut r3 = Vec::<bool>::with_capacity(LOOPS);

    let mut r4 = Vec::<bool>::with_capacity(LOOPS);

    let mut r5 = Vec::<u64>::with_capacity(LOOPS);

    //let mut orders = <std::vec::Vec<order::Order>>::new();

    for x in 0..LOOPS {
        r1.insert(x, rng.gen_range(1..750));
        r2.insert(x, rng.gen_range(0u64..2u64) == 0);
        //r2.insert(x, r1[x] >375);

        //cancel?
        r3.insert(x, rng.gen_range(0..18) < 10);

        // GTC?
        r4.insert(x, rng.gen_range(0..12) < 3);
        //volume
        r5.insert(x, rng.gen_range(1..10));
    }

    let mut now = Instant::now();
    /*
    for x in 0..2000 {
        book.insert_order(Order {
            side: if rng.gen_range(0u64, 2u64) == 0 {
                OrderSide::ASK
            } else {
                OrderSide::BID
            },
            limit: Price(rng.gen_range(1, 750)),
            volume: Volume(rng.gen_range(1, 10)),
            id: x as u64,
            //callback: Some(callback),
            //event_sender: None,
            filled_volume: Cell::new(0),
            filled_value: Cell::new(Value::ZERO),
            immediate_or_cancel: rng.gen_range(0, 12) < 3,
        });
    }
    */

    println!("Time for order placement: {}", now.elapsed().as_millis());
    now = Instant::now();
    let _y = 0;
    for x in 0..(LOOPS as u64) {
        //  println!("x={:?}", x);
        if r3[x as usize] {
            // println!("removing");
            book.cancel_order((LOOPS as u64) + x - 1);
        //println!("removed");
        } else {
            /*
            //println!("inserting");
            book.insert_order(Order {
                side: if r2[x as usize] {
                    OrderSide::ASK
                } else {
                    OrderSide::BID
                },
                limit: Price(r1[x as usize]),
                volume: Volume(r5[x as usize]),
                id: LOOPS as u64 + x as u64,
                //callback: Some(callback),
                //event_sender: None,
                filled_volume: Cell::new(0),
                filled_value: Cell::new(Value::ZERO),
                immediate_or_cancel: r4[x as usize],
            });
            //println!("inserted");
            */
        }
    }
    println!(
        "Time for orderbook change: {}, Mtps: {}",
        now.elapsed().as_millis(),
        (LOOPS as f32 / 1_000f32) / (now.elapsed().as_millis() as f32)
    );
}

fn benchmark_order_book2() {
    /*
    let mut book = OrderBook::new();
    /*
        for x in 1..1000 {
            book.insert_order(Order::new(
                Price::new(x),
                Volume::new(10_000),
                OrderSide::ASK,
                None,
                false,
            ));
        }
    */
    let mut now = Instant::now();

    for x in 0..30 {
        thread::sleep(Duration::from_millis(50));
        now = Instant::now();
        for y in 0..1000000 {
            book.insert_order(Order::new(
                Price::new(y % 750 + 1),
                Volume::new(1),
                OrderSide::BID,
                None,
                false,
            ));
        }

        println!(
            "Matching at price={:?} took {:?}ms",
            x,
            now.elapsed().as_millis()
        );
    }

    let ten_millis = time::Duration::from_millis(100000);

    thread::sleep(ten_millis);
    */
}

fn test_order_book() {
    /*
    let mut book = OrderBook::new();

    for x in 5..10 {
        book.insert_order(Order::new(
            Price::new(x),
            Volume::new(1),
            OrderSide::ASK,
            None,
            false,
        ));
    }
    println!("t");
    book.insert_order(Order::new(
        Price::new(8),
        Volume::new(1),
        OrderSide::BID,
        None,
        false,
    ));
    */
}
