use std::cell::{Cell, RefCell};
use std::ops::Neg;

use crate::primitives::*;

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
    Filled(u64, Volume, Value),
    Canceled(u64),
}

pub struct Order {
    pub limit: Price,
    pub volume: Volume,
    pub side: OrderSide,
    pub id: u64,
    pub immediate_or_cancel: bool,

    //pub event_sender: Option<RefCell<Sender<OrderEvent>>>,
    pub filled_volume: Cell<Volume>,
    pub filled_value: Cell<Value>,
}

impl PartialEq for Order {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl Order {
    pub fn new(
        id: u64,
        limit: Price,
        volume: Volume,
        side: OrderSide,
        //event_sender: Option<RefCell<Sender<OrderEvent>>>,
        immediate_or_cancel: bool,
    ) -> Order {
        Order {
            limit: limit,
            volume: volume,
            side: side,
            id: id,
            //event_sender: event_sender,
            filled_volume: Cell::new(Volume::ZERO),
            filled_value: Cell::new(Value::ZERO),
            immediate_or_cancel: immediate_or_cancel,
        }
    }

    pub fn remaining_volume(&self) -> Volume {
        self.volume - self.filled_volume.get()
    }

    //Will fill the order as much as possible and return how much fit in
    //Price ist just to set the filled_value correctly
    pub async fn fill(&self, volume: Volume, price: Price) -> Volume {
        if self.remaining_volume() <= volume {
            let old_volume = self.remaining_volume();

            //Fill order completely
            self.filled_volume.set(self.volume);
            self.filled_value.set(self.volume * price);
            self.notify();

            //Return what did fit in
            old_volume
        } else {
            //Fill as much as possible
            self.filled_volume.set(self.filled_volume.get() + volume);
            self.filled_value
                .set(self.filled_value.get() + (volume * price));

            self.notify();
            //Return volume, because everything fit in
            volume
        }
    }

    pub fn is_filled(&self) -> bool {
        self.remaining_volume().get() == 0
    }

    /// Check if an INCOMING order would match at this price
    pub fn matches_with(&self, price: &Price) -> bool {
        match self.side {
            // ask orders want a higher price
            OrderSide::ASK => (self.limit <= *price),

            // bid orders want a lower price
            OrderSide::BID => (self.limit >= *price),
        }
    }

    /// Call the callback function
    /// Execute this whenever the order state changes
    pub async fn notify(&self) {
        /*
        match &self.event_sender {
            Some(event_sender) => {
                event_sender
                    .borrow_mut()
                    .send(OrderEvent::Filled(
                        self.id,
                        self.filled_volume.get(),
                        self.filled_value.get(),
                    ))

            }
            None => Ok(()),
        }
        */
    }

    /// Call the callback function
    /// Execute this when the order gets removed from the orderbook without being completely filled
    pub async fn cancel(&self) {
        /*
        assert!(!self.is_filled());
        match &self.event_sender {
            Some(event_sender) => {
                event_sender
                    .borrow_mut()
                    .send(OrderEvent::Canceled(self.id))
                    .await
            }
            None => Ok(()),
        }
        */
    }
}
