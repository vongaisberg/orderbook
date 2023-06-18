use crate::exchange::commands::OrderCommand;
use crate::exchange::exchange_settings::ExchangeSettings;
use crate::order_handling::event::{self, MatchingEngineEvent};
use crate::order_handling::order::*;
use crate::order_handling::order_book::OrderBook;
use crate::risk::participant::Participant;
use crate::risk::risk_engine::RiskEngine;
use tokio::sync::mpsc;
use tokio::sync::mpsc::{Receiver, Sender};

use std::collections::HashMap;
use tokio::sync::RwLock;

const ORDER_BOOK_COUNT: usize = 1;
pub struct RiskEngineProcessor {
    risk_engine: RiskEngine,
}

impl RiskEngineProcessor {
    pub fn new(settings: ExchangeSettings) -> Self {
        let mut risk_engine = RiskEngine::new(settings);
        let mut part = Participant::default();
        part.assets.insert(0, 1000);
        part.assets.insert(1, 1000);
        risk_engine.add_participant(part);
        RiskEngineProcessor { risk_engine }
    }

    pub async fn run(
        &mut self,
        mut command_receiver: Receiver<OrderCommand>,
        senders: Vec<Sender<OrderCommand>>,
        mut event_receiver: Receiver<MatchingEngineEvent>,
    ) {
        loop {
            tokio::select! {
                order_command = command_receiver.recv() => self.run_pre(&senders, order_command).await,
                event = event_receiver.recv() => self.run_post(event).await,
            }
        }
    }

    pub async fn run_pre(
        &mut self,
        senders: &[Sender<OrderCommand>],
        order_command: Option<OrderCommand>,
    ) {
        if let Some(order_command) = order_command {
            println!("Risk received command: {:?}", order_command);
            let result = self.risk_engine.process_command(order_command);
            match result {
                crate::risk::risk_engine::RiskEngineResult::ValidForMatchingEngine => {
                    Self::send_to_matching_engine(order_command, senders).await
                }
                crate::risk::risk_engine::RiskEngineResult::InsufficientFunds => {
                    println!("Insufficient funds!")
                }
                crate::risk::risk_engine::RiskEngineResult::SymbolNotFound => todo!(),
                crate::risk::risk_engine::RiskEngineResult::UserNotFound => todo!(),
                crate::risk::risk_engine::RiskEngineResult::OrderNotFound => {
                    println!("Order not Found")
                }
            }
        }
    }
    pub async fn run_post(&mut self, event: Option<MatchingEngineEvent>) {
        if let Some(event) = event {
            println!("{:?}", event);
            self.risk_engine.process_matcher_event(event);
        }
    }

    async fn send_to_matching_engine(command: OrderCommand, senders: &[Sender<OrderCommand>]) {
        println!("Sending to matching engine: {:?}", command);
        let symbol_id = match command {
            OrderCommand::Trade(command) => command.symbol,
            OrderCommand::Cancel(command) => command.symbol,
        };
        let _ = senders[symbol_id as usize].send(command).await;
    }
}
