use std::cell::{Cell, RefCell, RefMut};
use std::collections::hash_map::{Entry, OccupiedEntry};
use std::ops::Neg;
use std::ptr::NonNull;
use std::sync::mpsc::Sender;

use super::order_bucket::OrderBucket;

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
pub enum OrderEvent {
    ///How much volume was filled and how much Value was payed for it
    //id, volume, value
    Filled(u64, u64, u64),
    //id
    Canceled(u64),
}

#[derive(Debug)]
pub struct StandingOrder {
    pub limit: u64,
    pub volume: u64,
    pub side: OrderSide,
    pub id: u64,

    pub filled_volume: Cell<u64>,
    pub filled_value: Cell<u64>,

    pub next: Option<NonNull<Box<StandingOrder>>>,
    pub prev: Option<NonNull<Box<StandingOrder>>>,
}

impl PartialEq for StandingOrder {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl StandingOrder {
    pub fn new(id: u64, limit: u64, volume: u64, side: OrderSide) -> StandingOrder {
        StandingOrder {
            limit,
            volume,
            side,
            id,
            //event_sender: event_sender,
            filled_volume: Cell::new(0),
            filled_value: Cell::new(0),

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
        self.volume - self.filled_volume.get()
    }

    //Will fill the order as much as possible and return how much fit in
    //Price ist just to set the filled_value correctly
    pub fn fill(&self, volume: u64, price: u64, sender: &Option<Sender<OrderEvent>>) -> u64 {
        //println!("Own volume: {}, Incoming volume: {}", *self.remaining_volume(), *volume);
        if self.remaining_volume() <= volume {
            let old_volume = self.remaining_volume();
            //Fill order completely
            self.filled_volume.set(self.volume);
            self.filled_value
                .set(self.filled_value.get() + old_volume * price);
            self.notify(old_volume, old_volume * price, sender);

            //Return what did fit in
            old_volume
        } else {
            //Fill as much as possible
            self.filled_volume.set(self.filled_volume.get() + volume);
            self.filled_value
                .set(self.filled_value.get() + (volume * price));

            self.notify(volume, volume * price, sender);
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
    pub fn notify(&self, fill_volume: u64, fill_value: u64, sender: &Option<Sender<OrderEvent>>) {
        if let Some(sender) = sender {
            sender.send(OrderEvent::Filled(
                self.id,
                self.filled_volume.get(),
                self.filled_value.get(),
            ));
        }
    }
}
