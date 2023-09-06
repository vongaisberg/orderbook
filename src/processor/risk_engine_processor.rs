use crate::exchange::commands::OrderCommand;
use crate::exchange::exchange_settings::ExchangeSettings;
use crate::order_handling::event::{self, MatchingEngineEvent};
use crate::order_handling::order::*;
use crate::order_handling::order_book::OrderBook;
use crate::risk::participant::Participant;
use crate::risk::risk_engine::RiskEngine;
use log::debug;
use tokio::sync::mpsc;
use crossbeam::channel::{Receiver, Sender, RecvError, Select};

use std::collections::HashMap;
use tokio::sync::RwLock;

pub struct RiskEngineProcessor {
    risk_engine: RiskEngine,
}

impl RiskEngineProcessor {
    pub fn new(settings: ExchangeSettings) -> Self {
        let mut risk_engine = RiskEngine::new(settings);
        let mut part = Participant::default();
        part.assets.insert(0, 1000);
        part.assets.insert(1, 1000);
        part.assets.insert(2, 1000);
        risk_engine.add_participant(part);
        let mut part2 = Participant::default();
        part2.assets.insert(0, 1000);
        part2.assets.insert(1, 1000);
        part2.assets.insert(2, 1000);
        part2.id = 1;
        risk_engine.add_participant(part2);
        let mut part3 = Participant::default();
        part3.assets.insert(0, 1000);
        part3.assets.insert(1, 1000);
        part3.assets.insert(2, 1000);
        part3.id = 2;
        risk_engine.add_participant(part3);
        let mut part4 = Participant::default();
        part4.assets.insert(0, 1000);
        part4.assets.insert(1, 1000);
        part4.assets.insert(2, 1000);
        part4.id = 3;
        risk_engine.add_participant(part4);
        RiskEngineProcessor { risk_engine }
    }

    pub fn run(
        &mut self,
        command_receiver: Receiver<OrderCommand>,
        senders: Vec<Sender<OrderCommand>>,
        event_receiver: Receiver<MatchingEngineEvent>,
    ) {

        let select = Select::new();
        loop {
            crossbeam::channel::select! {
                recv(command_receiver) -> order_command => self.run_pre(&senders, order_command),
                recv(event_receiver) -> event => self.run_post(event),
            }
        }
    }

    pub fn run_pre(
        &mut self,
        senders: &[Sender<OrderCommand>],
        order_command: Result<OrderCommand, RecvError>,
    ) {
        if let Ok(order_command) = order_command {
            debug!("Risk on: {:?}", order_command);
            let result = self.risk_engine.process_command(order_command);
            match result {
                crate::risk::risk_engine::RiskEngineResult::ValidForMatchingEngine => {
                    debug!("Order is valid");
                  Self::send_to_matching_engine(order_command, senders)
                }
                crate::risk::risk_engine::RiskEngineResult::InsufficientFunds => {
                    debug!("Insufficient funds!")
                }
                crate::risk::risk_engine::RiskEngineResult::SymbolNotFound => todo!(),
                crate::risk::risk_engine::RiskEngineResult::UserNotFound => todo!(),
                crate::risk::risk_engine::RiskEngineResult::OrderNotFound => {
                    debug!("Order not Found")
                }
            }
        } 
    }
    pub fn run_post(&mut self, event: Result<MatchingEngineEvent, RecvError>) {
        if let Ok(event) = event {
            debug!("Risk off:     {:?}", event);
            self.risk_engine.process_matcher_event(event);
        }
    }

    fn send_to_matching_engine(command: OrderCommand, senders: &[Sender<OrderCommand>]) {
        // println!("Sending to matching engine: {:?}", command);
        let symbol_id = match command {
            OrderCommand::Trade(command) => command.symbol,
            OrderCommand::Cancel(command) => command.symbol,
        };
        let _ = senders[symbol_id as usize].send(command);
    }
}
