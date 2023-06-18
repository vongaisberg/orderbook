use std::cell::{Cell, RefCell, RefMut};
use std::collections::hash_map::{Entry, OccupiedEntry};
use std::ops::Neg;
use std::ptr::NonNull;
use tokio::sync::mpsc::Sender;

use crate::exchange::commands::TradeCommand;
use crate::risk::participant;

use super::event::MatchingEngineEvent;
use super::order_bucket::OrderBucket;

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

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum OrderSide {
    ASK,
    BID,
}

impl Neg for OrderSide {
    type Output = Self;
    fn neg(self) -> Self {
        match self {
            Self::ASK => Self::BID,
            Self::BID => Self::ASK,
        }
    }
}

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
    pub async fn fill(&mut self, volume: u64, price: u64, sender: &Sender<MatchingEngineEvent>) -> u64 {
        //println!("Own volume: {}, Incoming volume: {}", *self.remaining_volume(), *volume);

        if self.remaining_volume() <= volume {
            let old_volume = self.remaining_volume();
            //Fill order completely
            self.volume = 0;
            self.notify(old_volume, old_volume * price, sender).await;

            //Return what did fit in
            old_volume
        } else {
            //Fill as much as possible
            self.volume -= volume;

            self.notify(volume, volume * price, sender).await;
            //Return volume, because everything fit in
            volume
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

    /// Call the callback function
    /// Execute this whenever the order state changes
    pub async fn notify(&self, fill_volume: u64, fill_value: u64, sender: &Sender<MatchingEngineEvent>) {
        let _ = sender.send(MatchingEngineEvent::Filled(
            self.id,
            fill_volume,
            fill_value,
        )).await;
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
