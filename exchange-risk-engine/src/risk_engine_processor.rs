use crate::participant::Participant;
use crate::risk_engine::{RiskEngine, RiskEngineResult};
use exchange_lib::commands::OrderCommand;
use exchange_lib::event::{self, MatchingEngineEvent};
use exchange_lib::exchange_settings::ExchangeSettings;
use exchange_lib::message_queue::{connect, publish, subscribe, Message, Payload};
use exchange_lib::order_side::*;
use log::debug;
use redis::Connection;

pub struct RiskEngineProcessor {
    risk_engine: RiskEngine,
    con_tx: Connection,
}

impl RiskEngineProcessor {
    
    const CHANNEL_TX: &str = "risk-pass";
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
        RiskEngineProcessor {
            risk_engine,
            con_tx: connect(),
        }
    }

    pub fn run(&mut self) {
        let mut con_rx = connect();
        let channels_rx: [String; 1] = ["port".to_string()];

        let _ = subscribe(&mut con_rx, &channels_rx, |msg| match msg.payload {
            Payload::CommandPayload(order_command) => self.run_pre(order_command),
            Payload::MatchingPayload(matching_event) => self.run_post(matching_event),
        });
    }

    pub fn run_pre(&mut self, order_command: OrderCommand) {
        debug!("Risk on: {:?}", order_command);
        let result = self.risk_engine.process_command(order_command);
        match result {
            RiskEngineResult::ValidForMatchingEngine => {
                debug!("Order is valid");
                self.send_to_matching_engine(order_command)
            }
            RiskEngineResult::InsufficientFunds => {
                debug!("Insufficient funds!")
            }
            RiskEngineResult::SymbolNotFound => todo!(),
            RiskEngineResult::UserNotFound => todo!(),
            RiskEngineResult::OrderNotFound => {
                debug!("Order not Found")
            }
        }
    }
    pub fn run_post(&mut self, event: MatchingEngineEvent) {
        debug!("Risk off:     {:?}", event);
        self.risk_engine.process_matcher_event(event);
    }

    fn send_to_matching_engine(&mut self, command: OrderCommand) {
        // println!("Sending to matching engine: {:?}", command);
        let symbol_id = match command {
            OrderCommand::Trade(command) => command.symbol,
            OrderCommand::Cancel(command) => command.symbol,
        };
        let msg = Message::new(Self::CHANNEL_TX.to_string(), Payload::CommandPayload(command));
        publish(&mut self.con_tx, msg);
    }
}
