use std::sync::{Arc, Mutex};

use pub_sub::PubSub;

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
}

impl TryFrom<warp::hyper::body::Bytes> for State {
    
    type Error = ();
    
    fn try_from(value: warp::hyper::body::Bytes) -> Result<Self, Self::Error> {

        println!("{:?}, {}", value, String::from_utf8(value.to_ascii_uppercase()).unwrap().as_str() );

        println!("{:?}", value == "BUSY");

        match String::from_utf8(value.to_ascii_uppercase()).unwrap().as_str() {
             "BUSY" => Ok(State::BUSY),
             "OK_FOR_INTERRUPTIONS" => Ok(State::OK_FOR_INTERRUPTIONS),
             "FREE" => Ok(State::FREE),
             "OFF" => Ok(State::OFF),
             _ => Err(())
         }
        
        
    }
}

lazy_static! {
    pub static ref CURRENT_STATE: Arc<Mutex<State>> = Arc::new(Mutex::new(State::OFF));
    pub static ref CHANNEL: PubSub<State> = PubSub::new();
}


