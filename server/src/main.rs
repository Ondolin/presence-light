#[macro_use]
extern crate lazy_static;

use warp::{Filter, Rejection};

mod handlers;
mod ws;
mod state;

#[tokio::main]
async fn main() {

    println!("Configuring websocket route");
    let ws_route = warp::path("ws")
        .and(warp::ws())
        .map(|ws: warp::ws::Ws| {
            ws.on_upgrade(move |socket| ws::client_connection(socket))
        });

    let change_state = warp::post()
        .and(warp::path("current"))
        .and(warp::body::bytes())
        .and_then(handlers::post_current);


    let incoming_log = warp::log::custom(|info| {
        eprintln!(
            "{} {} {} {:?}",
            info.method(),
            info.path(),
            info.status(),
            info.elapsed(),
        );
    });

    let routes = change_state.or(ws_route).with(warp::cors().allow_any_origin()).with(incoming_log);


    println!("Starting server");
    warp::serve(routes)
        .tls()
        .cert_path("./cert.crt")
        .key_path("./cert.key")
        .run(([0, 0, 0, 0], 8000)).await;
}
