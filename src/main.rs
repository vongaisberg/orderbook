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
use futures::executor::block_on;
use log::LevelFilter;
use rand_distr::{Distribution, Normal};
use simple_logger::SimpleLogger;
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

use std::sync::mpsc;

fn main() {
    SimpleLogger::new()
        .with_level(LevelFilter::Info)
        .init()
        .unwrap();

    // benchmark_exchange();
    test();
    std::thread::sleep(Duration::from_secs(3));
}

const COUNT: usize = 10_000_000;
const SECOND_COUNT: usize = 1;
static mut CANCELED: u64 = 0;

fn build_random_order_command(
    rng: &mut ThreadRng,
    normal_ask: Normal<f64>,
    normal_bid: Normal<f64>,
    id: u64,
) -> OrderCommand {
    let side = if rng.gen_bool(0.3) {
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
            symbol: rng.gen_range(0..2),
            participant_id: rng.gen_range(0..4),
            side,
            volume: rng.gen_range(1..5),
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

fn benchmark_exchange() {
    let mut rng = thread_rng();

    let mut symbols = Vec::new();
    symbols.push(Symbol {
        symbol_type: SymbolType::ExchangePair,
        base_asset: 0,
        quote_asset: 1,
    });
    symbols.push(Symbol {
        symbol_type: SymbolType::ExchangePair,
        base_asset: 0,
        quote_asset: 2,
    });

    let settings = ExchangeSettings {
        symbols,
        risk_engine_shards: 4,
        db_sync_speed: Duration::from_micros(500),
        db_min_recv_timeout: Duration::from_micros(100),
    };

    let mut ex = Exchange::new(settings);

    let normal_bid = Normal::new(200f64, 15f64).unwrap();
    let normal_ask = Normal::new(230f64, 15f64).unwrap();

    let now = Instant::now();

    for i in 0..COUNT {
        ex.trade(build_random_order_command(
            &mut rng, normal_ask, normal_bid, i as u64,
        ));
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

fn test() {
    let mut ex = Exchange::new(ExchangeSettings {
        symbols: vec![Symbol {
            symbol_type: SymbolType::ExchangePair,
            base_asset: 0,
            quote_asset: 1,
        }],
        risk_engine_shards: 1,
        db_sync_speed: Duration::from_micros(500),
        db_min_recv_timeout: Duration::from_micros(100),
    });

    let t = OrderCommand::Trade(TradeCommand {
        id: 0,
        participant_id: 0,
        symbol: 0,
        side: OrderSide::BID,
        volume: 10,
        limit: 5,
        immediate_or_cancel: false,
    });
    ex.trade(t);

    let t = OrderCommand::Trade(TradeCommand {
        id: 0,
        participant_id: 0,
        symbol: 0,
        side: OrderSide::ASK,
        volume: 5,
        limit: 3,
        immediate_or_cancel: false,
    });

    ex.trade(t);
}
