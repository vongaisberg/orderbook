#![feature(entry_insert)]

use exchange_lib::exchange_settings::ExchangeSettings;
use order_book_processor::OrderBookProcessor;

pub mod order;
pub mod order_book;
pub mod order_book_processor;
pub mod order_bucket;
pub mod public_list;

fn main() {
    env_logger::init();
    let settings = ExchangeSettings {
        ..Default::default()
    };
    OrderBookProcessor::new(1, settings).run()
}
