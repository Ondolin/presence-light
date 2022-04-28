use std::time::{Duration, Instant};

use actix::prelude::*;
use actix_web_actors::ws;

use crate::state::{State, RECIEVER_ADDRS, CURRENT_STATE};

use uuid::Uuid;

/// How often heartbeat pings are sent
const HEARTBEAT_INTERVAL: Duration = Duration::from_secs(5);

/// How long before lack of client response causes a timeout
const CLIENT_TIMEOUT: Duration = Duration::from_secs(15);

pub struct MyWebSocket {
    hb: Instant,
    id: Uuid
}

impl MyWebSocket {
    pub fn new() -> Self {
        Self { 
            hb: Instant::now(),
            id: Uuid::new_v4()
        }
    }

    // heart beat messages
    fn hb(&mut self, ctx: &mut <Self as Actor>::Context) {
        ctx.run_interval(HEARTBEAT_INTERVAL, |act, ctx| {
            // check client heartbeats
            if Instant::now().duration_since(act.hb) > CLIENT_TIMEOUT {
                // heartbeat timed out
                log::warn!("Websocket Client heartbeat failed, disconnecting!");

                // stop actor
                ctx.stop();

                // don't try to send a ping
                return;
            }

            ctx.ping(b"");
        });
    }

}

impl Actor for MyWebSocket {
    type Context = ws::WebsocketContext<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {

        log::info!("Client connected.");

        self.hb(ctx);

        // send the current state
        {
            let state = CURRENT_STATE.lock().unwrap();
        
            ctx.text(state.to_str());

        }

        // add the commection to the recievers list
        {
            let mut current = RECIEVER_ADDRS.lock().unwrap();
            current.insert(self.id, ctx.address());
        }

    }

    fn stopped(&mut self, _ctx: &mut Self::Context) {
       
        // remove the client from reciever list when disconnecting
        let mut map = RECIEVER_ADDRS.lock().unwrap();

        match map.remove(&self.id) {
            Some(_) => log::info!("Client disconnected."),
            None => log::error!("Could not remove client form reciever list!")
        }

    }
    
}


impl Message for State {
    type Result = Result<(), ()>;
}

impl Handler<State> for MyWebSocket {

    type Result = Result<(), ()>;

    fn handle(&mut self, msg: State, ctx: &mut Self::Context) -> Self::Result {

        ctx.text(msg.to_str());

        Ok(()) 
    }
}

/// Handler for `ws::Message`
impl StreamHandler<Result<ws::Message, ws::ProtocolError>> for MyWebSocket {
    fn handle(&mut self, msg: Result<ws::Message, ws::ProtocolError>, ctx: &mut Self::Context) {

        match msg {
            Ok(ws::Message::Ping(msg)) => {
                self.hb = Instant::now();
                ctx.pong(&msg);
            }
            Ok(ws::Message::Pong(_)) => {
                self.hb = Instant::now();
            }
            Ok(ws::Message::Text(_)) => ctx.text("Your message was ignored!"),
            Ok(ws::Message::Binary(_)) => ctx.text("Your message was ignored!"),
            Ok(ws::Message::Close(reason)) => {
                ctx.close(reason);
                ctx.stop();
            }
            _ => ctx.stop(),
        }
    }
}
