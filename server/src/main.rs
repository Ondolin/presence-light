#[macro_use]
extern crate lazy_static;

#[macro_use]
extern crate diesel;
extern crate dotenv;

mod server;
mod state;

mod db;
mod db_schema;

use std::sync::Mutex;

use diesel::prelude::*;

use self::server::MyWebSocket;
use self::state::{State, CURRENT_STATE, RECIEVER_ADDRS};

use actix_web::{
    middleware, post, web, App, Error, HttpRequest, HttpResponse, HttpServer, Responder,
};
use actix_web_actors::ws;
use actix_cors::Cors;

use openssl::ssl::{SslAcceptor, SslFiletype, SslMethod};

async fn echo_ws(req: HttpRequest, stream: web::Payload) -> Result<HttpResponse, Error> {
    ws::start(MyWebSocket::new(), &req, stream)
}

#[post("/current")]
async fn set_state(db_connection: web::Data<Mutex<SqliteConnection>>, req_body: String) -> impl Responder {
    let lock = RECIEVER_ADDRS.lock().unwrap();

    if let Ok(state) = State::from_str(req_body) {
        {
            let mut current = CURRENT_STATE.lock().unwrap();

            *current = state;
        }

        {
            let db_lock = db_connection.lock().unwrap();

            db::insert_state_log(&db_lock, state);
        }

        for (_uuid, addr) in &mut lock.iter() {
            match addr.try_send(state) {
                Ok(_) => {}
                Err(e) => log::error!("Could not send message. Got error: {}", e),
            }
        }

        HttpResponse::Ok()
    } else {
        HttpResponse::NotFound()
    }
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));

    log::info!("starting HTTP server at http://localhost:8080");

    let mut builder = SslAcceptor::mozilla_intermediate(SslMethod::tls()).unwrap();
    builder
        .set_private_key_file("cert.key", SslFiletype::PEM)
        .unwrap();
    builder.set_certificate_chain_file("cert.crt").unwrap();

    let db_connection = web::Data::new(Mutex::new(db::establish_connection()));

    HttpServer::new(move || {
        let cors = Cors::default()
              .allow_any_origin();

        App::new()
            .service(web::resource("/ws").route(web::get().to(echo_ws)))
            .app_data(db_connection.clone())
            .service(set_state)
            .wrap(cors)
            .wrap(middleware::Logger::default())
    })
    .bind_openssl("0.0.0.0:443", builder)?
    .run()
    .await
}
