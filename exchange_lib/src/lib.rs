//extern crate rand;
#![allow(dead_code)]
#![allow(unused_variables)]
#![allow(unused_assignments)]
#![feature(async_closure)]
#![feature(dropck_eyepatch)]
#![feature(extend_one)]
#![feature(exact_size_is_empty)]
#![feature(entry_insert)]

//mod bit_set;
pub mod asset;
pub mod commands;
pub mod order_side;
pub mod exchange_settings;
pub mod router;
pub mod message_queue;
pub mod event;