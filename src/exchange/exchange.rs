use crate::exchange::account::Account;
use crate::exchange::asset::Asset;
use crate::exchange::commands::OrderCommand;
use crate::order_handling::order::*;
use crate::order_handling::order_book::OrderBook;

use std::collections::HashMap;
use std::sync::RwLock;

#[derive(Default)]
pub struct Exchange<'a> {
    pub accounts: RwLock<HashMap<u64, Account>>,
    pub assets: RwLock<HashMap<&'a str, Asset>>,
    pub orderbooks: RwLock<HashMap<&'a str, RwLock<OrderBook>>>,
}

impl<'a> Exchange<'a> {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add_account(&self, acc: Account) {
        self.accounts.write().unwrap().insert(acc.id, acc);
    }

    pub fn add_asset(&self, asset: Asset) {
        self.assets.write().unwrap().insert(asset.ticker, asset);
    }

    pub fn add_orderbook(&self, asset: &str, book: OrderBook) -> Result<(), &str> {
        match self.assets.read().unwrap().get(asset) {
            Some(ass) => match self.orderbooks.write().unwrap().insert(ass.ticker, RwLock::new(book)) {
                None => Ok(()),
                Some(_) => Err("There is already an orderbook for this asset."),
            },
            None => Err("This asset does not exist."),
        }
    }

    pub fn trade(&self, order_command: OrderCommand) -> Result<(), String> {
        match order_command {
            OrderCommand::Trade(trade) => {
                self.check_asset_existance(trade.ticker)?;
                let orderbooks = self.orderbooks.read().unwrap();
                let book = orderbooks.get(trade.ticker).ok_or("This orderbook does not exist.")?;
                let order = Order::new(trade.limit, trade.volume, trade.side, None, trade.immediate_or_cancel);
                book.write().unwrap().insert_order(order);
                Ok(())
            }
            OrderCommand::Cancel(cancel) => {
                let orderbooks = self.orderbooks.read().unwrap();
                let book = orderbooks.get(cancel.ticker).ok_or("This orderbook does not exist.")?;
                let mut book = book.write().unwrap();
                book.remove_order(cancel.order_id)
            }
        }
    }
    /// Check if an assets with that ticker exists on this exchange
    fn check_asset_existance<'b>(&'b self, asset: &str) -> Result<(), &str> {
        if self.assets.read().unwrap().contains_key(asset) {
            Ok(())
        } else {
            Err("This asset does not exist.")
        }
    }
}
