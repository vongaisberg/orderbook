use super::order_bucket::OrderBucket;
use exchange_lib::commands::TradeCommand;
use exchange_lib::event::MatchingEngineEvent;
use exchange_lib::message_queue::{publish, publish_pipeline, Message, Payload};
use exchange_lib::order_side::OrderSide;
use log::debug;
use redis::{Connection, Pipeline};
use core::panic;
use std::ptr::NonNull;

// #[derive(Debug)]
// pub struct NonNull<T>(NonNull<T>);
// unsafe impl<T> Send for NonNull<T> {}
// impl<T> NonNull<T> {
//     pub fn as_mut(mut self) -> &'static mut T {
//         unsafe { self.0.as_mut() }
//     }
//     pub fn as_ref(mut self) -> &'static mut T {
//         unsafe { &mut self.0.as_ref() }
//     }
//     pub fn from_nonnull(value: NonNull<T>) -> Self {
//         Self(value)
//     }
// }

#[derive(Debug)]
pub struct StandingOrder {
    pub limit: u64,
    pub volume: u64,
    pub side: OrderSide,
    pub id: u64,
    pub participant_id: u64,

    pub next: Option<NonNull<Box<StandingOrder>>>,
    pub prev: Option<NonNull<Box<StandingOrder>>>,
}

impl PartialEq for StandingOrder {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl StandingOrder {
    pub fn new(
        id: u64,
        participant_id: u64,
        limit: u64,
        volume: u64,
        side: OrderSide,
    ) -> StandingOrder {
        StandingOrder {
            limit,
            volume,
            side,
            id,
            participant_id,

            next: None,
            prev: None,
        }
    }

    pub fn remove_from_bucket(&mut self, bucket: &mut OrderBucket) {
        match self.next {
            None => bucket.tail = self.prev,
            Some(mut n) => {
                let entry = unsafe { n.as_mut() };
                entry.prev = self.prev;
            }
        }
        match self.prev {
            None => bucket.head = self.next,
            Some(mut p) => {
                let entry = unsafe { p.as_mut() };
                entry.next = self.next;
            }
        }
        bucket.len -= 1;
    }

    pub fn remaining_volume(&self) -> u64 {
        self.volume
    }

    //Will fill the order as much as possible and return how much fit in
    //Price ist just to set the filled_value correctly
    pub fn fill(&mut self, taker: &StandingOrder, price: u64, con_tx: &mut Vec<Message>) -> u64 {
        //println!("Own volume: {}, Incoming volume: {}", *self.remaining_volume(), *volume);

        if self.remaining_volume() <= taker.volume {
            let old_volume = self.remaining_volume();
            //Fill order completely
            self.volume = 0;
            self.notify(old_volume, old_volume * price, con_tx);

            //Return what did fit in
            old_volume
        } else {
            //Fill as much as possible
            self.volume -= taker.volume;

            self.notify(taker.volume, taker.volume * price, con_tx);
            //Return volume, because everything fit in
            taker.volume
        }
    }

    pub fn is_filled(&self) -> bool {
        self.remaining_volume() == 0
    }

    /// Check if an INCOMING order would match at this price
    pub fn matches_with(&self, price: u64) -> bool {
        match self.side {
            // ask orders want a higher price
            OrderSide::ASK => self.limit <= price,

            // bid orders want a lower price
            OrderSide::BID => self.limit >= price,
        }
    }

    /// Notify the risk engine about any change to the order
    /// Execute this whenever the order state changes
    pub fn notify(&self, fill_volume: u64, fill_value: u64, con_tx: &mut Vec<Message>) {
        let event = MatchingEngineEvent::Filled(self.id, fill_volume, fill_value);
        debug!("TX: {:?}", event);
        if self.limit > 100000000 {
            println!{"{:?}", self}
            panic!();
        }
        con_tx.push(Message::new(
            "risk".to_string(),
            Payload::MatchingPayload(event),
        ));
    }
}

impl From<TradeCommand> for StandingOrder {
    fn from(value: TradeCommand) -> Self {
        Self::new(
            value.id,
            value.participant_id,
            value.limit,
            value.volume,
            value.side,
        )
    }
}
