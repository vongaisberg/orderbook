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
pub mod processor;
pub mod risk;
use crate::exchange::asset::Symbol;
use crate::exchange::asset::SymbolType;
use crate::exchange::exchange_settings::ExchangeSettings;
use crate::order_handling::event::MatchingEngineEvent;
use crate::processor::order_book_processor::*;

use crate::order_handling::order::*;
use crate::order_handling::order_book::*;
use crate::order_handling::order_bucket::OrderBucket;
use crate::risk::*;
use rand_distr::{Distribution, Normal};
use std::rc::Rc;

use rand::prelude::*;
use std::cell::Cell;
use std::cmp::max;

use std::{collections::HashMap, ptr::NonNull};

use std::time::{Duration, Instant};

use std::{thread, time};

use crate::exchange::commands::*;
use crate::exchange::exchange::*;
use crate::order_handling::order_bucket::*;

use order_handling::{order, public_list::*};

use tokio::sync::mpsc::channel;


#[tokio::main]
async fn main() {

    test_exchange().await;
}

const COUNT: usize = 10000;
const SECOND_COUNT: usize = 1;
static mut CANCELED: u64 = 0;

fn build_random_order_command(
    rng: &mut ThreadRng,
    normal_ask: Normal<f64>,
    normal_bid: Normal<f64>,
    id: u64,
) -> OrderCommand {
    let side = if rng.gen_bool(0.5) {
        OrderSide::ASK
    } else {
        OrderSide::BID
    };
    if rng.gen_bool(0.4) {
        unsafe { CANCELED += 1 };

        OrderCommand::Cancel(CancelCommand {
            symbol: 0,
            order_id: unsafe { CANCELED },
            participant_id: 0,
        })
    } else {
        OrderCommand::Trade(TradeCommand {
            symbol: 0,
            participant_id: 0,
            side,
            volume: rng.gen_range(1..2),
            limit: if side == OrderSide::ASK {
                normal_ask.sample(rng) as u64
            } else {
                normal_bid.sample(rng) as u64
            },
            //immediate_or_cancel: rng.gen_range(0, 12) < 3,
            immediate_or_cancel: false,
            id,
        })
    }
}

async fn test_exchange() {
    let mut rng = thread_rng();

    let mut symbols = Vec::new();
    symbols.push(Symbol {
        symbol_type: SymbolType::ExchangePair,
        base_asset: 0,
        quote_asset: 1,
    });

    let settings = ExchangeSettings {
        symbols,
        risk_engine_shards: 1,
    };

    let mut ex = Exchange::new(settings);

    let normal_bid = Normal::new(200f64, 15f64).unwrap();
    let normal_ask = Normal::new(230f64, 15f64).unwrap();

    let queue = unsafe {
        let mut arr: Box<[OrderCommand; COUNT]> = Box::new(std::mem::uninitialized());

        for (i, item) in arr[..].iter_mut().enumerate() {
            std::ptr::write(
                item,
                build_random_order_command(&mut rng, normal_ask, normal_bid, i as u64),
            );
        }
        arr
    };
    let now = Instant::now();

    for i in 0..COUNT {
        ex.trade(queue[i]);
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
