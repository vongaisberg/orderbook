use crate::{commands::OrderCommand, event::MatchingEngineEvent};
use redis::{Commands, Connection, Pipeline, PubSub, PubSubCommands, RedisError, RedisResult};
use serde::{Deserialize, Serialize};
use std::{
    error::Error,
    io::{BufRead, BufReader, Read, Write},
    net::TcpStream,
    time::Duration,
};

#[derive(Clone, Debug, Serialize, Deserialize, bitcode::Encode, bitcode::Decode)]
pub struct Message {
    pub channel: String,
    pub payload: Payload,
}

#[derive(Clone, Debug, Serialize, Deserialize, bitcode::Encode, bitcode::Decode)]
pub enum Payload {
    CommandPayload(OrderCommand),
    MatchingPayload(MatchingEngineEvent),
}

impl Message {
    pub fn new(channel: String, payload: Payload) -> Self {
        Message { channel, payload }
    }
}

pub fn connect() -> Connection {
    let client = redis::Client::open("redis://127.0.0.1/").unwrap();
    client.get_connection().unwrap()
}

pub fn connect_tcp() -> TcpStream {
    TcpStream::connect("127.0.0.1:6379").unwrap()
}

pub fn publish_batch(con: &mut TcpStream, msg: &[Message]) {
    let len = msg.len();
    let len_cut = if (len > 1100) { 1000 } else { len };
    let serialized_msg: String = msg[0..len_cut]
        .iter()
        .map(|m| (&m.channel, serde_json::to_string(m).unwrap()))
        .map(|(c, m)| {
            format!(
                "*3\r\n$7\r\nPUBLISH\r\n${}\r\n{}\r\n${}\r\n{}\r\n",
                c.len(),
                c,
                m.len(),
                m
            )
        })
        .collect();
    // println!("serialized: {}", serialized_msg);
    con.write_all(serialized_msg.as_bytes()).unwrap();
    if len_cut != len {
        println!("Second {}", len);
        publish_batch(con, &msg[len_cut..])
    }
}

pub fn publish(con: &mut Connection, msg: Message) -> Result<(), Box<dyn Error>> {
    let json = serde_json::to_string(&msg)?;

    con.publish(msg.channel, json)?;

    Ok(())
}

pub fn publish_pipeline(con: &mut Pipeline, msg: Message) -> Result<(), Box<dyn Error>> {
    let json = serde_json::to_string(&msg)?;

    con.publish(msg.channel, json);

    Ok(())
}

pub fn subscribe(
    con: &mut Connection,
    channels: &[String],
    mut handler: impl FnMut(Message),
) -> Result<(), Box<dyn Error>> {
    let res: Result<(), RedisError> = con.subscribe(channels, |msg| {
        let received: String = msg.get_payload().unwrap();
        let msg_obj = serde_json::from_str::<Message>(&received).unwrap();

        handler(msg_obj);
        redis::ControlFlow::Continue
    });

    match res {
        Ok(()) => todo!(),
        Err(err) => {
            println!("{}", err.is_connection_dropped());
        }
    }
    Ok(())
}

pub fn subscribe_tcp(
    con: &mut TcpStream,
    channels: &[String],
    mut handler: impl FnMut(Message),
) -> Result<(), Box<dyn Error>> {
    con.write_all(format!("SUBSCRIBE {}\n", &channels.join(" ")).as_bytes())?;

    let mut buffer = String::new();
    let mut length = String::new();
    let mut message = Vec::<u8>::new();
    message.resize(100, 0);
    let mut reader = BufReader::new(con.try_clone().unwrap());
    loop {
        reader.read_line(&mut buffer)?;
        if buffer.contains("message") {
            // Skip three lines
            reader.read_line(&mut buffer)?; // channel name length
            reader.read_line(&mut buffer)?; // channel name
            buffer.clear();

            reader.read_line(&mut length)?; //message length
            let msg_length: usize = length.trim()[1..].parse()?;
            length.clear();

            reader.read_exact(&mut message[0..msg_length])?; //message

            let msg_obj = serde_json::from_slice::<Message>(&message[0..msg_length]).unwrap();

            handler(msg_obj);
        } else {
            buffer.clear();
        }
    }
}

pub fn pubsub_batching_tcp(
    con: &mut TcpStream,
    con_tx: &mut TcpStream,
    channels: &[String],
    batch_size: usize,
    timeout: Duration,
    mut handler: impl FnMut(Message, &mut Vec<Message>),
) -> Result<(), Box<dyn Error>> {
    // Subscribe to all channels
    con.write_all(format!("SUBSCRIBE {}\n", &channels.join(" ")).as_bytes())?;

    con.set_read_timeout(Some(timeout));

    //Various read buffers
    let mut buffer = String::new();
    let mut length = String::new();
    let mut message = Vec::<u8>::new();

    // Write buffer
    let mut pipe = Vec::<Message>::with_capacity(batch_size*10);
    // let mut pipe = redis::pipe();
    // let mut pipe_count = 0;

    message.resize(200, 0);
    let mut reader = BufReader::new(con.try_clone().unwrap());
    loop {
        match reader.read_line(&mut buffer) {
            Ok(_) => {
                if buffer.contains("message") {
                    // Skip three lines
                    reader.read_line(&mut buffer)?; // channel name length
                    reader.read_line(&mut buffer)?; // channel name
                    buffer.clear();

                    reader.read_line(&mut length)?; //message length
                    let msg_length: usize = length.trim()[1..].parse()?;
                    length.clear();

                    reader.read_exact(&mut message[0..msg_length])?; //message

                    let msg_obj =
                        serde_json::from_slice::<Message>(&message[0..msg_length]).unwrap();

                    handler(msg_obj, &mut pipe);
                    // pipe_count += 1;

                    if pipe.len() >= batch_size {
                        publish_batch(con_tx, &pipe);
                        pipe.clear();
                        // pipe_count = 0;
                    }
                } else {
                    buffer.clear();
                }
            }
            Err(err) => {
                // println!("Possible timeout: {:?}", err);
                if pipe.len() > 1 {
                    publish_batch(con_tx, &pipe);
                    pipe.clear();
                    // pipe_count = 0;
                }
            }
        }
    }
}

pub fn pubsub_batching(
    con_rx: &mut PubSub,
    con_tx: &mut Connection,
    channels: &[String],
    batch_size: u32,
    timeout: Duration,
    mut handler: impl FnMut(Message, &mut Pipeline),
) -> Result<(), Box<dyn Error>> {
    for c in channels {
        // println!("Sub");
        con_rx.subscribe(c)?;
    }
    con_rx.set_read_timeout(Some(timeout))?;
    let mut pipe = redis::pipe();
    let mut recv_count = 0;
    loop {
        match con_rx.get_message() {
            Ok(msg) => {
                recv_count += 1;
                let payload = msg.get_payload_bytes();

                let msg_obj = serde_json::from_slice(payload)?;

                handler(msg_obj, &mut pipe);
                if recv_count >= batch_size {
                    pipe.execute(con_tx);
                    recv_count = 0;
                    pipe = redis::pipe();
                }
            }
            Err(err) => {
                if err.is_timeout() && recv_count > 0 {
                    pipe.execute(con_tx);
                    recv_count = 0;
                    pipe = redis::pipe();
                } else {
                    // RedisResult::<()>::Err(err).unwrap();
                }
            }
        }
    }
    // let _: () = con
    //     .subscribe(channels, |msg| {
    //         let received: String = msg.get_payload().unwrap();
    //         let msg_obj = serde_json::from_str::<Message>(&received).unwrap();

    //         handler(msg_obj);
    //         redis::ControlFlow::Continue
    //     })
    //     .unwrap();
    Ok(())
}
