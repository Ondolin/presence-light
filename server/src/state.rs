#![allow(non_camel_case_types)]

use std::{sync:: Mutex, collections::HashMap};

use crate::server::MyWebSocket;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum State {
    BUSY,
    OK_FOR_INTERRUPTIONS,
    FREE,
    OFF
}

impl State {
    pub fn to_str(&self) -> &str {
        match self {
            State::BUSY => "BUSY",
            State::OK_FOR_INTERRUPTIONS => "OK_FOR_INTERRUPTIONS",
            State::FREE => "FREE",
            State::OFF => "OFF"
        }
    }

    pub fn from_str(str: String) -> Result<State, ()> {
        match str.as_str() {
            "BUSY" => Ok(State::BUSY),
            "OK_FOR_INTERRUPTIONS" => Ok(State::OK_FOR_INTERRUPTIONS),
            "FREE" => Ok(State::FREE),
            "OFF" => Ok(State::OFF),
            _ => Err(())
        }
    }
}


lazy_static! {
    pub static ref CURRENT_STATE: Mutex<State> = Mutex::new(State::OFF);
    pub static ref RECIEVER_ADDRS: Mutex<HashMap<uuid::Uuid, actix::Addr<MyWebSocket>>> = Mutex::new(HashMap::new());
}


