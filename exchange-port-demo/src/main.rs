use std::{
    thread::{self, JoinHandle},
    time::Instant,
};

use exchange_lib::{
    commands::{OrderCommand, TradeCommand},
    message_queue::{connect, connect_tcp, publish, subscribe, subscribe_tcp, Message, Payload},
    order_side::OrderSide,
};
use log::debug;
use rand::Rng;
use redis::Connection;

fn main() {
    let sub = bench_sub(Box::new(["risk".to_string()]));
    let a = thread::spawn(move || {
        bench_pipe(500, 1_000, &create_test_messages(1_000_000, OrderSide::BID));
    });
    let b = thread::spawn(move || {
        bench_pipe(500, 1_000, &create_test_messages(1_000_000, OrderSide::ASK));
    });

    a.join();
    b.join();
    sub.join();
}

fn create_test_messages(count: usize, side: OrderSide) -> Vec<Message> {
    let mut v = Vec::new();

    for a in 0..count {
        v.push({
            let payload = Payload::CommandPayload(OrderCommand::Trade(TradeCommand {
                id: (if side == OrderSide::ASK {
                    a as u64
                } else {
                    a as u64 + 10000
                }),
                participant_id: 1,
                symbol: 1,
                side,
                volume: 1,
                limit: 1,
                immediate_or_cancel: false,
            }));
            Message::new("orders".to_string(), payload)
        })
    }
    v
}
fn optimize_batch_size() {
    let tps = 500_000;
    let mut v = create_test_messages(10_000, OrderSide::ASK);
    for i in 1..10 {
        let i = i * 20;
        println!("{}", i);
        let mut avg = 0f32;
        for _ in 0..10 {
            avg += bench_pipe(tps / i, i, &v);
        }
        println!("Batch size: {}, average worst case: {}", i, avg / 10f32);
    }

    // bench_pipe();
    // env_logger::init();
    // bench_1();
}

// #[test]
fn bench_1() {
    let mut con = connect();

    let subscriber = sub(Box::new(["risk".to_string()]));
    let mut rng = rand::thread_rng();

    let start = Instant::now();
    const COUNT: u64 = 10_000;
    for i in 0..COUNT {
        // if rng.gen_bool(0.5) {
        //     ask(&mut con, i, 1, rng.gen_range(70..100))
        // } else {
        //     bid(&mut con, i, 1, rng.gen_range(70..100))
        // }
        ask(&mut con, 1, 1, 1);
    }
    let duration = Instant::now() - start;
    println!(
        "Duration: {}, Mtps: {}",
        duration.as_secs_f32(),
        COUNT as f64 / duration.as_micros() as f64
    );
}

fn test_single() {
    let mut con = connect();

    let subscriber = sub(Box::new(["risk".to_string()]));

    ask(&mut con, 1, 1, 100);

    bid(&mut con, 2, 1, 100);
    debug!("Published");

    subscriber.join().unwrap();
}

fn sub(channels: Box<[String]>) -> JoinHandle<()> {
    let a = thread::spawn(move || {
        let mut con = connect();
        let _ = subscribe(&mut con, &channels, |msg| {
            debug!("{:?}", msg);
        });
    });
    a
}

fn ask(con: &mut Connection, id: u64, vol: u64, limit: u64) {
    let payload = Payload::CommandPayload(OrderCommand::Trade(TradeCommand {
        id,
        participant_id: 1,
        symbol: 1,
        side: OrderSide::ASK,
        volume: vol,
        limit,
        immediate_or_cancel: false,
    }));
    let msg = Message::new("orders".to_string(), payload);
    publish(con, msg);
}

fn bench_pipe(c1: u64, c2: u64, v: &Vec<Message>) -> f32 {
    let start = Instant::now();

    let mut con = connect();
    let mut worst_case_latency = 0f32;
    for i in 0..c1 {
        // println!("i: {}", i);
        // let start2 = Instant::now();
        publish_pipe(&mut con, &v[i as usize*c2 as usize..(i as usize+1)*c2 as usize]);
        let duration2 = Instant::now() - start;
        let latency = duration2.as_secs_f32() - ((i * c2) as f32 / (c1 * c2) as f32);
        if latency > worst_case_latency {
            worst_case_latency = latency;
        }
    }
    let duration = Instant::now() - start;
    // println!(
    //     "Batch size: {} Latency: {}ms, Duration: {}s, Mtps: {}",
    //     c2,
    //     worst_case_latency * 1000f32,
    //     duration.as_secs_f32(),
    //     (c1 * c2) as f64 / duration.as_micros() as f64
    // );
    return worst_case_latency;
}

fn publish_pipe(con: &mut Connection, vec: &[Message]) {
    let mut pipe = redis::pipe();
    for m in vec {
        pipe.publish(&m.channel, serde_json::to_string(m).unwrap());
    }
    pipe.execute(con);
}

fn bench_sub(channels: Box<[String]>) -> JoinHandle<()> {
    let a = thread::spawn(move || {
        let mut con = connect_tcp();
        // let mut con = connect();

        let start = Instant::now();
        let mut count = 0;
        let _ = subscribe_tcp(&mut con, &channels, |msg| {
            count += 1;
            // println!("{}", count);

            if count % 10_000 == 0 {
                println!(
                    "Receive duration {}ms, count {}, msg: {:?}",
                    (Instant::now() - start).as_millis(),
                    count,
                    msg
                );
            }
        });
    });
    a
}

fn bid(con: &mut Connection, id: u64, vol: u64, limit: u64) {
    let payload = Payload::CommandPayload(OrderCommand::Trade(TradeCommand {
        id,
        participant_id: 1,
        symbol: 1,
        side: OrderSide::BID,
        volume: vol,
        limit,
        immediate_or_cancel: false,
    }));
    let msg = Message::new("orders".to_string(), payload);
    publish(con, msg);
}
