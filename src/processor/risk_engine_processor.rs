use crate::exchange::commands::OrderCommand;
use crate::order_handling::event::{self, MatchingEngineEvent};
use crate::order_handling::order::*;
use crate::order_handling::order_book::OrderBook;
use crate::risk::risk_engine::RiskEngine;
use tokio::sync::mpsc;
use tokio::sync::mpsc::{Receiver, Sender};

use std::collections::HashMap;
use tokio::sync::RwLock;

const ORDER_BOOK_COUNT: usize = 1;
#[derive(Default)]
pub struct RiskEngineProcessor {
    risk_engine: RiskEngine,
}

impl RiskEngineProcessor {
    pub fn new() -> Self {
        RiskEngineProcessor::default()
    }

    pub async fn run(
        &mut self,
        mut command_receiver: Receiver<OrderCommand>,
        senders: Vec<Sender<OrderCommand>>,
        mut event_receiver: Receiver<MatchingEngineEvent>,
    ) {
        tokio::select! {
            order_command = command_receiver.recv() => self.run_pre(senders, order_command).await,
            event = event_receiver.recv() => self.run_post(event).await,
        }
    }

    pub async fn run_pre(
        &mut self,
        senders: Vec<Sender<OrderCommand>>,
        order_command: Option<OrderCommand>,
    ) {
        if let Some(order_command) = order_command {
            let result = self.risk_engine.process_command(order_command);
            match result {
                crate::risk::risk_engine::RiskEngineResult::ValidForMatchingEngine => {
                    Self::send_to_matching_engine(order_command, senders.clone())
                }
                crate::risk::risk_engine::RiskEngineResult::InsufficientFunds => todo!(),
                crate::risk::risk_engine::RiskEngineResult::SymbolNotFound => todo!(),
                crate::risk::risk_engine::RiskEngineResult::UserNotFound => todo!(),
            }
        }
    }
    pub async fn run_post(&mut self, event: Option<MatchingEngineEvent>) {
        if let Some(event) = event {
            self.risk_engine.process_matcher_event(event);
        }
    }

    fn send_to_matching_engine(command: OrderCommand, senders: Vec<Sender<OrderCommand>>) {
        let symbol_id = match command {
            OrderCommand::Trade(command) => command.symbol,
            OrderCommand::Cancel(command) => command.symbol,
        };
        senders[symbol_id as usize].send(command);
    }
}
