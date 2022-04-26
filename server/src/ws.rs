use futures::SinkExt;
use futures::StreamExt;
use futures::TryFutureExt;
use warp::ws::Message;
use warp::ws::WebSocket;
use crate::state::{CHANNEL, CURRENT_STATE};

pub async fn client_connection(ws: WebSocket) {
    println!("establishing client connection... {:?}", ws);

    let (mut tx, _rx) = ws.split();

    tokio::task::spawn(async move {

   
        let current_state = {
            CURRENT_STATE.clone().lock().unwrap().clone()
        };

        tx.send(Message::text(current_state.to_str())).unwrap_or_else(|_| {println!("EEEEROOOR")}).await; 

        let recv = CHANNEL.subscribe();
        
        loop {

            let update = recv.recv(); 

            tx.send(Message::text(update.unwrap().to_str())).unwrap_or_else(|x| {println!("EEEEROOOR, {:?}", x)}).await; 
            
        }

    });
}
