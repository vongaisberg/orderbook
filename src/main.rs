//extern crate rand;
#![allow(dead_code)]
#![allow(unused_variables)]
#![allow(unused_assignments)]
#![allow(unused_imports)]
#![feature(async_closure)]
#![feature(dropck_eyepatch)]
#![feature(extend_one)]
#![feature(exact_size_is_empty)]
#![feature(entry_insert)]

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

    // test_exchange();
    //println!("test")
    /*     let mut vec = BitVec::from_elem(700, true);
    vec.set(0, false);
    vec.set(1, false);
    println!("First element: {}", order_book::first_entry(vec).unwrap()); */

    //test_hot_set_index();
    test_exchange();
    //test_order_book();
}

const COUNT: usize = 100_000;
const SECOND_COUNT: usize = 1_00;
static mut canceled: u64 = 0;

fn build_random_order_command(
    rng: &mut ThreadRng,
    normal_ask: Normal<f64>,
    normal_bid: Normal<f64>,
) -> OrderCommand {
    let side = if rng.gen_bool(0.5) {
        OrderSide::ASK
    } else {
        OrderSide::BID
    };
    if rng.gen_bool(0.4) {
        unsafe { canceled += 1 };

        OrderCommand::Cancel(CancelCommand {
            ticker: 0,
            order_id: unsafe { canceled },
        })
    } else {
        OrderCommand::Trade(TradeCommand {
            ticker: 0,
            side,
            volume: rng.gen_range(1..10),
            limit: if side == OrderSide::ASK {
                normal_ask.sample(rng) as u64
            } else {
                normal_bid.sample(rng) as u64
            },
            //immediate_or_cancel: rng.gen_range(0, 12) < 3,
            immediate_or_cancel: false,
        })
    }
}

fn test_exchange() {
    let mut rng = thread_rng();
    let mut ex = Exchange::new();
    let normal_bid = Normal::new(200f64, 15f64).unwrap();
    let normal_ask = Normal::new(230f64, 15f64).unwrap();

    let queue = unsafe {
        let mut arr: Box<[OrderCommand; COUNT]> = Box::new(std::mem::uninitialized());
        let mut i = 0;

        for item in &mut arr[..] {
            std::ptr::write(
                item,
                build_random_order_command(&mut rng, normal_ask, normal_bid),
            );

            i += 1;
        }
        arr
    };
    let now = Instant::now();
    ex.trade(&queue[0]);
    ex.trade(&queue[1]);
    for j in 0..SECOND_COUNT {
        for i in 0..COUNT {
            ex.trade(&queue[i]);
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

fn test_exchange_matching() {
    let mut ex = Exchange::new();
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
}

fn test_order() {
    /*
    let order = Order::new(Price::new(1), Volume::new(1), OrderSide::ASK, None, false);
    let filled_volume = order.fill(Volume::new(1), Price::new(1));
    assert_eq!(filled_volume, Volume::new(1));
    */
}

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
// #[test]
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
        // println!("inserting {}", x);
        // book.insert_order(Box::new(StandingOrder::new(
        //     x,
        //     if r2[x as usize] {
        //         r1[x as usize] + 600
        //     } else {
        //         r1[x as usize]
        //     },
        //     r5[x as usize],
        //     if r2[x as usize] {
        //         OrderSide::ASK
        //     } else {
        //         OrderSide::BID
        //     },
        // )));

        //println!("inserted");
    }
    println!(
        "Time for orderbook change: {}, Mtps: {}",
        now.elapsed().as_millis(),
        (LOOPS as f32 / 1_000f32) / (now.elapsed().as_millis() as f32)
    );
}

fn benchmark_order_book2() {
    let mut rng = rand::thread_rng();
    let mut book = OrderBook::new();

    const LOOPS: usize = 10_000_000;
    let mut i = 0;

    let mut order_queue: [Box<StandingOrder>; LOOPS] = unsafe {
        let mut arr: [Box<StandingOrder>; LOOPS] = std::mem::uninitialized();
        for item in &mut arr[..] {
            //std::ptr::write(item, build_random_order(&mut rng, i));
            i += 1;
        }
        arr
    };
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
