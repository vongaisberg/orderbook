use crate::exchange::account::Account;
use crate::exchange::asset::Asset;
use crate::exchange::commands::OrderCommand;
use crate::order_handling::order::*;
use crate::order_handling::order_book::OrderBook;

use std::collections::HashMap;
use std::sync::RwLock;

const ORDER_BOOK_COUNT: usize = 1;
#[derive(Default)]
pub struct Exchange {
    //pub accounts: RwLock<HashMap<u64, Account>>,
    //pub assets: RwLock<HashMap<&'a str, Asset>>,
    pub orderbooks: [OrderBook; ORDER_BOOK_COUNT],
}

impl<'a> Exchange {
    pub fn new() -> Self {
        Self::default()
    }
    /*
            pub fn add_account(&self, acc: Account) {
                self.accounts.write().unwrap().insert(acc.id, acc);
            }

            pub fn add_asset(&self, asset: Asset) {
                self.assets.write().unwrap().insert(asset.ticker, asset);
            }

        pub fn add_orderbook(&self, asset: &str, book: OrderBook) -> Result<(), &str> {
            match self.assets.read().unwrap().get(asset) {
                Some(ass) => match self
                    .orderbooks
                    .write()
                    .unwrap()
                    .insert(ass.ticker, RwLock::new(book))
                {
                    None => Ok(()),
                    Some(_) => Err("There is already an orderbook for this asset."),
                },
                None => Err("This asset does not exist."),
            }
        }
    */
    pub fn trade(&mut self, order_command: &OrderCommand) -> Result<(), String> {
        match order_command {
            OrderCommand::Trade(trade) => {
                //self.check_asset_existance(trade.ticker)?;
                let book = &mut (self.orderbooks)[trade.ticker];
                let id = book.increment_id();
                let order = Order::new(
                    id as u64,
                    trade.limit,
                    trade.volume,
                    trade.side,
                    //None,
                    trade.immediate_or_cancel,
                );
                book.insert_order(order);
                Ok(())
            }
            OrderCommand::Cancel(cancel) => {
                let book = &mut self.orderbooks[cancel.ticker];
                book.remove_order(cancel.order_id);
                Ok(())
            }
        }
    }
    /*
    /// Check if an assets with that ticker exists on this exchange
    fn check_asset_existance<'b>(&'b self, asset: &str) -> Result<(), &str> {
        if self.assets.read().unwrap().contains_key(asset) {
            Ok(())
        } else {
            Err("This asset does not exist.")
        }
    }
    */
}
